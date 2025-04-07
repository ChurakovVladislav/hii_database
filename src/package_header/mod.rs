use core::fmt;
use core::fmt::{Display, Formatter};
use core::ptr::{self};
use core::mem;
use uefi::{CStr8};

mod types;
pub use types::*;

mod package_list;
pub use package_list::*;

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct EfiHiiPackageHeader {
    length: [u8; 3],
    r#type: u8,
}

impl EfiHiiPackageHeader {
    pub fn new(length: u32, pack_type: HiiPackageType) -> Self {
        let bytes = length.to_le_bytes();
        Self {
            length: [bytes[0], bytes[1], bytes[2]],
            r#type: pack_type as u8,
        }
    }

    pub fn length(&self) -> u32 {
        (self.length[2] as u32) << 16 | (self.length[1] as u32) << 8 | (self.length[0] as u32)
    }

    pub fn get_type(&self) -> Result<HiiPackageType, ()> {
        HiiPackageType::try_from(self.r#type)
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        let byte_array: [u8; 4] = [self.length[0], self.length[1], self.length[2], self.r#type];
        byte_array
    }
}

impl Display for EfiHiiPackageHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "0x{:02x} 0x{:02x} 0x{:02x} 0x{:02x}",
            self.length[0], self.length[1], self.length[2], self.r#type
        )
    }
}

// Value of HII package type
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum HiiPackageType {
    TypeAll = 0x00,
    TypeGuid = 0x01,
    Forms = 0x02,
    Strings = 0x04,
    Fonts = 0x05,
    Images = 0x06,
    SimpleFonts = 0x07,
    DevicePath = 0x08,
    KeyboardLayout = 0x09,
    Animations = 0x0A,
    End = 0xDF,
    TypeSystemBegin = 0x0E,
    TypeSystemEnd = 0xFF,
}

impl HiiPackageType {
    fn try_from(orig: u8) -> Result<Self, ()> {
        match orig {
            0x00 => Ok(HiiPackageType::TypeAll),
            0x01 => Ok(HiiPackageType::TypeGuid),
            0x02 => Ok(HiiPackageType::Forms),
            0x04 => Ok(HiiPackageType::Strings),
            0x05 => Ok(HiiPackageType::Fonts),
            0x06 => Ok(HiiPackageType::Images),
            0x07 => Ok(HiiPackageType::SimpleFonts),
            0x08 => Ok(HiiPackageType::DevicePath),
            0x09 => Ok(HiiPackageType::KeyboardLayout),
            0x0A => Ok(HiiPackageType::Animations),
            0xDF => Ok(HiiPackageType::End),
            0x0E => Ok(HiiPackageType::TypeSystemBegin),
            0xFF => Ok(HiiPackageType::TypeSystemEnd),
            _ => Err(()),
        }
    }
}

/// EFI_HII_PACKAGE_TYPE_x.
pub enum DefinedStruct {
    // EFI_HII_PACKAGE_FORM = 0x02
    FormPackage(HiiFormPackageHdr),
    // EFI_HII_PACKAGE_STRINGS = 0x04
    StringPackage(HiiStringPackageHdr),
    // EFI_HII_PACKAGE_FONTS = 0x05
    FontPackage(HiiFontPackageHdr),
    // EFI_HII_PACKAGE_END = 0xDF
    EndPackage(HiiEndPackageHdr),
    // UNKNOWN = ?
    Undefined(UndefineHiiPackageHdr),
}

pub trait PackageHeader {
    const PACKAGE_TYPE: HiiPackageType;

    fn from_undef(pack_head: &UndefineHiiPackageHdr) -> Self;

    fn header(&self) -> EfiHiiPackageHeader;
}

impl From<&UndefineHiiPackageHdr> for DefinedStruct {
    fn from(item: &UndefineHiiPackageHdr) -> Self {
        match item.header.get_type() {
            Ok(HiiFormPackageHdr::PACKAGE_TYPE) => {
                DefinedStruct::FormPackage(HiiFormPackageHdr::from_undef(item))
            }
            Ok(HiiStringPackageHdr::PACKAGE_TYPE) => {
                DefinedStruct::StringPackage(HiiStringPackageHdr::from_undef(item))
            }
            Ok(HiiFontPackageHdr::PACKAGE_TYPE) => {
                DefinedStruct::FontPackage(HiiFontPackageHdr::from_undef(item))
            }
            Ok(HiiEndPackageHdr::PACKAGE_TYPE) => {
                DefinedStruct::EndPackage(HiiEndPackageHdr::from_undef(item))
            }
            _ => DefinedStruct::Undefined(*item),
        }
    }
}

#[derive(Clone, Copy)]
/// Iterator for events in  [`UndefineHiiPackageIter`].
pub struct UndefineHiiPackageIter {
    pub location: *const u8,
    size: u32,
}

impl UndefineHiiPackageIter {
    pub unsafe fn new(start: *const u8) -> Self {
        // Get package size
        let size_list_pack = unsafe {
            ptr::read_unaligned(start as *const EfiHiiPackageListHeader).horizontal_resolution
        };

        Self {
            // Skip header EfiHiiPackageListHeader to get the location first package in the list
            location: unsafe { start.byte_offset(mem::size_of::<EfiHiiPackageListHeader>() as isize) },
            // Size is computed without EfiHiiPackageListHeader
            size: (size_list_pack - mem::size_of::<EfiHiiPackageListHeader>() as u32),
        }
    }

    pub fn get_string(&self, string_id: u16, language: &CStr8) -> Option<String> {
        if string_id != 0 {
            for sph in self
                .into_iter()
                .filter(|head| head.header.get_type() == Ok(HiiStringPackageHdr::PACKAGE_TYPE))
                .map(|t| HiiStringPackageHdr::from_undef(&t))
            {
                // Возвращаем первую найденную строку
                return sph.get_string(string_id - 1, language);
            }
        }
        None
    }
}

impl Iterator for UndefineHiiPackageIter {
    type Item = UndefineHiiPackageHdr;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let header = ptr::read_unaligned(self.location as *const EfiHiiPackageHeader);
            // Баг, так как скипаем послений пакет
            if self.size == 0 {
                return None;
            }

            let hii_package_header = UndefineHiiPackageHdr {
                header: header,
                location: self.location,
            };

            self.location = self.location.byte_offset(hii_package_header.len() as isize);
            self.size = self.size - hii_package_header.len();
            return Some(hii_package_header);
        }
    }
}

impl IntoIterator for UndefineHiiPackageListHeader {
    type Item = UndefineHiiPackageHdr;
    type IntoIter = UndefineHiiPackageIter;

    fn into_iter(self) -> UndefineHiiPackageIter {
        unsafe { UndefineHiiPackageIter::new(self.as_ptr()) }
    }
}

impl fmt::Display for UndefineHiiPackageIter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, package_list) in self.enumerate() {
            let _ = write!(
                f,
                "    Package[{}]: type={}; size=0x{:02X}\n",
                i,
                package_list.package_type(),
                package_list.len()
            );
        }
        Ok(())
    }
}

//
pub struct HiiPackage(Vec<u8>);

impl HiiPackage {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    pub fn size(&self) -> usize {
        self.0.len()
    }

    pub fn to_vec(&self) -> &Vec<u8> {
        &self.0
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0[..]
    }    
}