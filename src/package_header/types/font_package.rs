use crate::HiiPackageType;
use crate::package_header::{EfiHiiPackageHeader, PackageHeader, UndefineHiiPackageHdr};
use core::ptr::{self};

pub struct HiiFontPackageHdr {
    parts: UndefineHiiPackageHdr,
}

impl PackageHeader for HiiFontPackageHdr {
    const PACKAGE_TYPE: HiiPackageType = HiiPackageType::Fonts;

    fn from_undef(pack_head: &UndefineHiiPackageHdr) -> Self {
        Self {
            parts: pack_head.clone(),
        }
    }

    fn header(&self) -> EfiHiiPackageHeader {
        self.parts.header
    }
}

impl HiiFontPackageHdr {
    pub fn number_of_narrow_glyphs(&self) -> u16 {
        unsafe { ptr::read_unaligned(self.parts.as_ptr().byte_offset(4) as *const u16) }
    }

    pub fn number_of_wide_glyphs(&self) -> u16 {
        unsafe { ptr::read_unaligned(self.parts.as_ptr().byte_offset(6) as *const u16) }
    }
}
