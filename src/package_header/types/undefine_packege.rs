extern crate alloc;
use alloc::string::String;

use crate::package_header::EfiHiiPackageHeader;
use core::ptr::{self};

#[derive(Clone, Copy)]
pub struct UndefineHiiPackageHdr {
    // Header Hii Package
    pub header: EfiHiiPackageHeader,
    // Pointer to the location
    pub location: *const u8,
}

impl UndefineHiiPackageHdr {
    pub fn package_type(&self) -> String {
        match self.header.r#type {
            0x00 => String::from("ALL"),
            0x01 => String::from("GUID"),
            0x02 => String::from("FORMS"),
            0x04 => String::from("STRINGS"),
            0x05 => String::from("FONTS"),
            0x06 => String::from("IMAGES"),
            0x07 => String::from("SIMPLE_FONTS"),
            0x08 => String::from("DEVICE_PATH"),
            0x09 => String::from("KEYBOARD_LAYOU"),
            0x0A => String::from("ANIMATIONS"),
            0xDF => String::from("END"),
            0xE0 => String::from("SYSTEM_BEGIN"),
            0xFF => String::from("SYSTEM_END"),
            _ => String::from("UNKNOWN"),
        }
    }

    pub fn len(&self) -> u32 {
        self.header.length()
    }

    pub fn as_ptr(self) -> *mut u8 {
        self.location as *mut u8
    }

    pub fn get_slice(self) -> *const [u8] {
        ptr::slice_from_raw_parts(self.location, self.len() as usize)
    }
}
