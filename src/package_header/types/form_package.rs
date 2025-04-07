use crate::HiiPackageType;
use crate::package_header::{EfiHiiPackageHeader, PackageHeader, UndefineHiiPackageHdr};
use crate::HiiPackage;

extern crate alloc;
use alloc::vec::Vec;

use core::{
    ops::Sub,
    ptr::{self},
};

pub struct HiiFormPackageHdr {
    parts: UndefineHiiPackageHdr,
}

impl PackageHeader for HiiFormPackageHdr {
    const PACKAGE_TYPE: HiiPackageType = HiiPackageType::Forms;

    fn from_undef(pack_head: &UndefineHiiPackageHdr) -> Self {
        Self {
            parts: pack_head.clone(),
        }
    }

    fn header(&self) -> EfiHiiPackageHeader {
        self.parts.header
    }
}

impl HiiFormPackageHdr {
    pub fn as_data(&self) -> &[u8] {
        unsafe {
            let data = self.parts.location.byte_offset(4 as isize);
            let size_data = self.parts.len() - 4;
            &*ptr::slice_from_raw_parts(data, size_data as usize)
        }
    }

    pub fn count_op_codes(self) -> usize {
        unsafe { EfiOpHeaderIter::from_slice(self.as_data()) }.count()
    }

    /// Creates a new HII (Human Interface Infrastructure) form package.
    ///
    /// # Example usage:
    /// ```
    /// use hii_database::package_header::HiiFormPackageHdr;
    /// 
    /// let vfr_data: &[u8] = &[
    ///     0x0E, 0xA7, 0x10, 0x66, 0xC6, 0x32, 0xDF, 0x94, 0x6D, 0x4D, 0x98, 0x4F, 0x8C, 0xBE, 0x44, 0x51,
    ///     0x9B, 0x87, 0x02, 0x00, 0x03, 0x00, 0x01, 0x71, 0x99, 0x03, 0x93, 0x45, 0x85, 0x04, 0x4B, 0xB4,
    ///     0x5E, 0x32, 0xEB, 0x83, 0x26, 0x04, 0x0E, 0x5C, 0x06, 0x00, 0x00, 0x00, 0x00, 0x5C, 0x06, 0x00,
    ///     0x00, 0x01, 0x00, 0x26, 0x27, 0x0B, 0x00, 0x90, 0xAB, 0x3A, 0x3A, 0x86, 0x78, 0x2E, 0x4F, 0x88,
    ///     0xF8, 0x59, 0x7A, 0x95, 0x1B, 0x78, 0xBC, 0x07, 0x00, 0x00, 0x00, 0x03, 0x00, 0x53, 0x79, 0x73,
    ///     0x74, 0x65, 0x6D, 0x41, 0x63, 0x63, 0x65, 0x73, 0x73, 0x00, 0x01, 0x86, 0x01, 0x00, 0x04, 0x00,
    ///     0x05, 0x91, 0x05, 0x00, 0x06, 0x00, 0x01, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x04, 0x10, 0x00, 0x01,
    ///     0x00, 0x09, 0x07, 0x08, 0x00, 0x00, 0x00, 0x00, 0x09, 0x07, 0x07, 0x00, 0x10, 0x00, 0x01, 0x29,
    ///     0x02, 0x02, 0x87, 0x04, 0x00, 0x00, 0x00, 0x00, 0x29, 0x02, 0x29, 0x02, 0x29, 0x02
    /// ];
    /// 
    /// let hii_form_pack = hii_database::package_header::HiiFormPackageHdr::create(vfr_data);
    /// ```
    pub fn create(pack_data: &[u8]) -> HiiPackage {
        // Create header for 'HiiFormPackageHdr'
        let head = EfiHiiPackageHeader::new((pack_data.len() + 4) as u32, Self::PACKAGE_TYPE);

        //
        let mut form_pack = Vec::with_capacity(head.length() as usize);
        form_pack.extend_from_slice(&head.to_bytes());
        form_pack.extend_from_slice(&pack_data);
        HiiPackage::new(form_pack)
    }
}

/// Iterator for events in  [`EfiOpHeader`].
pub struct EfiOpHeader {
    location: *const u8,
}

impl EfiOpHeader {
    fn new(ptr: *const u8) -> Self {
        Self { location: ptr }
    }

    pub fn op_code(&self) -> u8 {
        unsafe { *self.location }
    }

    pub fn len(&self) -> usize {
        unsafe { (*self.location.offset(1) & 0x7f) as usize }
    }

    pub fn scope(&self) -> u8 {
        unsafe { (*self.location.offset(1) & 0x80) as u8 }
    }

    pub fn get_data(&self) -> Option<&[u8]> {
        if self.len() == 2 {
            return None;
        }
        unsafe {
            Some(&*ptr::slice_from_raw_parts(
                self.location.byte_offset(2),
                self.len().sub(2) as usize,
            ))
        }
    }
}

/// Iterator for events in  [`EfiOpHeaderIter`].
pub struct EfiOpHeaderIter {
    location: *const u8,
    size: usize,
    count: usize,
}

impl EfiOpHeaderIter {
    pub unsafe fn from_slice(start: &[u8]) -> Self {
        Self {
            location: start.as_ptr(),
            size: start.len(),
            count: 0,
        }
    }
}

impl Iterator for EfiOpHeaderIter {
    type Item = EfiOpHeader;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            // Читаем заголовок
            let header = EfiOpHeader::new(self.location);
            if self.size == self.count {
                return None;
            }

            self.location = self.location.byte_add(header.len());
            self.count += header.len();
            return Some(header);
        }
    }
}

impl IntoIterator for HiiFormPackageHdr {
    type Item = EfiOpHeader;
    type IntoIter = EfiOpHeaderIter;

    fn into_iter(self) -> EfiOpHeaderIter {
        unsafe { EfiOpHeaderIter::from_slice(self.as_data()) }
    }
}

pub mod ifr_parse {
    use super::*;
    use core::mem::transmute;

    #[repr(u8)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum EfiIfrOpCode {
        FormOp = 0x01,
        SubtitleOp = 0x02,
        TextOp = 0x03,
        ImageOp = 0x04,
        OneOfOp = 0x05,
        CheckboxOp = 0x06,
        NumericOp = 0x07,
        PasswordOp = 0x08,
        OneOfOptionOp = 0x09,
        SuppressIfOp = 0x0A,
        LockedOp = 0x0B,
        ActionOp = 0x0C,
        ResetButtonOp = 0x0D,
        FormSetOp = 0x0E,
        RefOp = 0x0F,
        NoSubmitIfOp = 0x10,
        InconsistentIfOp = 0x11,
        EqIdValOp = 0x12,
        EqIdIdOp = 0x13,
        EqIdValListOp = 0x14,
        AndOp = 0x15,
        OrOp = 0x16,
        NotOp = 0x17,
        RuleOp = 0x18,
        GrayOutIfOp = 0x19,
        DateOp = 0x1A,
        TimeOp = 0x1B,
        StringOp = 0x1C,
        RefreshOp = 0x1D,
        DisableIfOp = 0x1E,
        AnimationOp = 0x1F,
        ToLowerOp = 0x20,
        ToUpperOp = 0x21,
        MapOp = 0x22,
        OrderedListOp = 0x23,
        VarstoreOp = 0x24,
        VarstoreNameValueOp = 0x25,
        VarstoreEfiOp = 0x26,
        VarstoreDeviceOp = 0x27,
        VersionOp = 0x28,
        EndOp = 0x29,
        MatchOp = 0x2A,
        GetOp = 0x2B,
        SetOp = 0x2C,
        ReadOp = 0x2D,
        WriteOp = 0x2E,
        EqualOp = 0x2F,
        NotEqualOp = 0x30,
        GreaterThanOp = 0x31,
        GreaterEqualOp = 0x32,
        LessThanOp = 0x33,
        LessEqualOp = 0x34,
        BitwiseAndOp = 0x35,
        BitwiseOrOp = 0x36,
        BitwiseNotOp = 0x37,
        ShiftLeftOp = 0x38,
        ShiftRightOp = 0x39,
        AddOp = 0x3A,
        SubtractOp = 0x3B,
        MultiplyOp = 0x3C,
        DivideOp = 0x3D,
        ModuloOp = 0x3E,
        RuleRefOp = 0x3F,
        QuestionRef1Op = 0x40,
        QuestionRef2Op = 0x41,
        Uint8Op = 0x42,
        Uint16Op = 0x43,
        Uint32Op = 0x44,
        Uint64Op = 0x45,
        TrueOp = 0x46,
        FalseOp = 0x47,
        ToUintOp = 0x48,
        ToStringOp = 0x49,
        ToBooleanOp = 0x4A,
        MidOp = 0x4B,
        FindOp = 0x4C,
        TokenOp = 0x4D,
        StringRef1Op = 0x4E,
        StringRef2Op = 0x4F,
        ConditionalOp = 0x50,
        QuestionRef3Op = 0x51,
        ZeroOp = 0x52,
        OneOp = 0x53,
        OnesOp = 0x54,
        UndefinedOp = 0x55,
        LengthOp = 0x56,
        DupOp = 0x57,
        ThisOp = 0x58,
        SpanOp = 0x59,
        ValueOp = 0x5A,
        DefaultOp = 0x5B,
        DefaultStoreOp = 0x5C,
        FormMapOp = 0x5D,
        CatenateOp = 0x5E,
        GuidOp = 0x5F,
        SecurityOp = 0x60,
        ModalTagOp = 0x61,
        RefreshIdOp = 0x62,
        WarningIfOp = 0x63,
        Match2Op = 0x64,
    }

    fn is_expression_op_code(operand: u8) -> bool {
        if (operand >= EfiIfrOpCode::EqIdValOp as u8) && (operand <= EfiIfrOpCode::NotOp as u8)
            || (operand >= EfiIfrOpCode::MatchOp as u8) && (operand <= EfiIfrOpCode::SetOp as u8)
            || (operand >= EfiIfrOpCode::EqualOp as u8) && (operand <= EfiIfrOpCode::SpanOp as u8)
            || (operand == EfiIfrOpCode::CatenateOp as u8)
            || (operand == EfiIfrOpCode::ToLowerOp as u8)
            || (operand == EfiIfrOpCode::ToUpperOp as u8)
            || (operand == EfiIfrOpCode::MapOp as u8)
            || (operand == EfiIfrOpCode::VersionOp as u8)
            || (operand == EfiIfrOpCode::SecurityOp as u8)
            || (operand == EfiIfrOpCode::Match2Op as u8)
        {
            return true;
        }
        false
    }

    fn is_know_op_code(operand: u8) -> bool {
        if operand > EfiIfrOpCode::Match2Op as u8 {
            true
        } else {
            false
        }
    }

    /// Calculate number of Statemens(Questions) and Expression OpCodes.
    pub fn count_op_codes(package: &HiiFormPackageHdr) -> (usize, usize) {
        let mut count: (usize, usize) = (0, 0);

        for opcode in unsafe { EfiOpHeaderIter::from_slice(package.as_data()) } {
            if is_expression_op_code(opcode.op_code()) {
                count.0 += 1;
            } else {
                count.1 += 1;
            }
        }
        return count;
    }

    impl From<u8> for EfiIfrOpCode {
        fn from(value: u8) -> Self {
            unsafe { transmute(value) }
        }
    }
}
