use crate::HiiPackageType;
use crate::package_header::{EfiHiiPackageHeader, PackageHeader, UndefineHiiPackageHdr};
use crate::HiiPackage;

extern crate alloc;
use alloc::slice;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use core::mem;
use core::ptr::{self};
use uefi::{CStr8, CStr16, CString16, Char16};

pub struct HiiStringPackageHdr {
    parts: UndefineHiiPackageHdr,
}

impl PackageHeader for HiiStringPackageHdr {
    const PACKAGE_TYPE: HiiPackageType = HiiPackageType::Strings;

    fn from_undef(pack_head: &UndefineHiiPackageHdr) -> Self {
        Self {
            parts: pack_head.clone(),
        }
    }

    fn header(&self) -> EfiHiiPackageHeader {
        self.parts.header
    }
}

#[repr(u8)]
#[derive(PartialEq, Eq)]
pub enum EfiHiiSibt {
    End = 0x00,
    StringScsu = 0x10,
    StringScsuFont = 0x11,
    StringsSCSU = 0x12,
    StringsScsuFont = 0x13,
    StringUcs2 = 0x14,
    StringUcs2Font = 0x15,
    StringsUcs2 = 0x16,
    StringsUcs2Font = 0x17,
    Duplicate = 0x20,
    Skip2 = 0x21,
    Skip1 = 0x22,
    Ext1 = 0x30,
    Ext2 = 0x31,
    Ext4 = 0x32,
    Font = 0x40,
}
#[repr(C)]
#[repr(packed)]
struct StringPackage {
    header: EfiHiiPackageHeader,
    hdr_size: u32,
    string_info_offset: u32,
    language_window: [Char16; 16],
    language_name: u16,
}

impl HiiStringPackageHdr {
    /// Creates a new HII (Human Interface Infrastructure) string package with a specified localization and set of strings.
    ///
    /// # Example usage:
    /// ```
    /// use uefi::CString16;
    /// use hii_database::package_header::HiiStringPackageHdr;
    /// 
    /// let string_pack = vec![CString16::try_from("English").unwrap()];
    /// let hii_pack = HiiStringPackageHdr::create("en-US\0".to_string(), string_pack);
    /// ```
    pub fn create(language: String, strings: Vec<CString16>) -> HiiPackage {
        // Form a block with CString16
        let mut str_block: Vec<Vec<u8>> = strings
            .iter()
            .map(|s| {
                let mut b = s.as_bytes().to_vec();
                b.insert(0, EfiHiiSibt::StringUcs2 as u8);
                b
            })
            .collect();
        str_block.push(vec![EfiHiiSibt::End as u8]);

        let bytes_str_block: &[u8] = &str_block
            .into_iter()
            .flat_map(|v| v.into_iter())
            .collect::<Vec<u8>>()[..];

        let header_size = language.len() as u32 + (mem::size_of::<StringPackage>() as u32);
        let package_size = header_size as usize + bytes_str_block.len();

        let str_hdr = StringPackage {
            header: EfiHiiPackageHeader::new(package_size as u32, Self::PACKAGE_TYPE),
            hdr_size: header_size,
            string_info_offset: header_size,
            language_window: [unsafe { Char16::from_u16_unchecked(0) }; 16],
            language_name: 1,
        };

        let head = unsafe {
            slice::from_raw_parts(
                (&str_hdr as *const StringPackage) as *const u8,
                mem::size_of::<StringPackage>(),
            )
        };

        let mut str_pack = Vec::with_capacity(package_size);
        str_pack.extend_from_slice(head);
        str_pack.extend_from_slice(language.as_bytes());
        str_pack.extend_from_slice(bytes_str_block);

        HiiPackage::new(str_pack)
    }
}

impl HiiStringPackageHdr {
    pub fn hdr_size(&self) -> u32 {
        unsafe { ptr::read_unaligned(self.parts.as_ptr().byte_offset(4) as *const u32) }
    }

    pub fn string_info_offset(&self) -> u32 {
        unsafe { ptr::read_unaligned(self.parts.as_ptr().byte_offset(8) as *const u32) }
    }

    pub fn language_window(&self) -> [Char16; 16] {
        unsafe { ptr::read_unaligned(self.parts.as_ptr().byte_offset(12) as *const [Char16; 16]) }
    }

    pub fn language_name(&self) -> u16 {
        unsafe { ptr::read_unaligned(self.parts.as_ptr().byte_offset(44) as *const u16) }
    }

    pub fn language(&self) -> &[u8] {
        unsafe {
            let data = self.parts.as_ptr().byte_offset(46);
            let size_data = self.hdr_size() - 46;
            &*ptr::slice_from_raw_parts(data, size_data as usize)
        }
    }

    pub fn str_language(&self) -> String {
        self.language()
            .iter()
            .filter(|b| **b != 0)
            .map(|byte| *byte as char)
            .collect()
    }

    // fn data(&self) -> &[u8] {
    //     unsafe {
    //         let data = self.parts.location.byte_offset(self.hdr_size() as isize);
    //         let size_data = self.parts.len() - self.hdr_size();
    //         & *ptr::slice_from_raw_parts(data, size_data as usize)
    //     }
    // }

    // Get iterator on block 
    fn blocks(&self) -> HiiStringBlockIter {
        unsafe {
            HiiStringBlockIter::new(self.parts.location.byte_offset(self.hdr_size() as isize))
        }
    }

    // Get string from package 
    pub fn get_string(&self, string_id: u16, language: &CStr8) -> Option<String> {
        if self.language() != language.as_bytes() {
            return None;
        }

        self.blocks()
            .nth(string_id as usize)
            .and_then(|un_str_block| un_str_block.get_string().map(|s| s.to_string()))
    }

    // Get count strings
    pub fn count_strings(&self) -> usize {
        self.blocks().count()
    }
}

type HiiStringBlock = u8;

pub struct UndefineHiiStringBlock {
    location: *const u8,
}

impl UndefineHiiStringBlock {
    pub fn new(ptr: *const u8) -> Self {
        Self { location: ptr }
    }

    pub fn block_type(&self) -> HiiStringBlock {
        unsafe { ptr::read_unaligned(self.location as *const HiiStringBlock) }
    }

    pub fn get_string(&self) -> Option<&CStr16> {
        unsafe {
            if self.block_type() == EfiHiiSibt::StringUcs2 as u8 {
                let ucs2_string = self.location.byte_offset(1) as *const Char16;
                let mut len: usize = 0;
                while *ucs2_string.add(len) != Char16::from_u16_unchecked(0) {
                    len += 1
                }
                // Add newline character
                len += 1;

                let s16 = &*ptr::slice_from_raw_parts(ucs2_string, len as usize);

                return Some(CStr16::from_char16_with_nul_unchecked(s16));
            }
            None
        }
    }
}

pub struct HiiStringBlockIter {
    pub location: *const u8,
}

impl HiiStringBlockIter {
    pub fn new(start: *const u8) -> Self {
        Self { location: start }
    }
}

impl Iterator for HiiStringBlockIter {
    type Item = UndefineHiiStringBlock;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let block = ptr::read_unaligned(self.location as *const HiiStringBlock);
            match block {
                0x14 => {
                    // EFI_HII_SIBT_STRING_UCS2
                    let ucs2_string = self.location.byte_offset(1) as *const Char16;
                    let mut len = 0;
                    while *ucs2_string.add(len) != Char16::from_u16_unchecked(0) {
                        len += 1
                    }
                    len = ((len + 1) * 2) + 1;
                    // Return the current block
                    let block = UndefineHiiStringBlock::new(self.location);
                    // Moving the iterator to the next block
                    self.location = self.location.byte_offset(len as isize);

                    Some(block)
                }
                _ => None,
            }
        }
    }
}
