use crate::HiiPackageType;
use crate::package_header::{EfiHiiPackageHeader, PackageHeader, UndefineHiiPackageHdr};
use crate::HiiPackage;

use core::mem;

pub struct HiiEndPackageHdr {
    parts: UndefineHiiPackageHdr,
}

impl PackageHeader for HiiEndPackageHdr {
    const PACKAGE_TYPE: HiiPackageType = HiiPackageType::End;

    fn from_undef(pack_head: &UndefineHiiPackageHdr) -> Self {
        Self {
            parts: pack_head.clone(),
        }
    }

    fn header(&self) -> EfiHiiPackageHeader {
        self.parts.header
    }
}

impl HiiEndPackageHdr {
    pub fn create() -> HiiPackage {
        let head = EfiHiiPackageHeader::new(
            mem::size_of::<EfiHiiPackageHeader>() as u32,
            Self::PACKAGE_TYPE,
        );

        HiiPackage::new(head.to_bytes().to_vec())
    }
}
