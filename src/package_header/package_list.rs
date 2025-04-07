use core::fmt;
use core::ptr::{self};
use uefi::Guid;

/// HII package list
#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct EfiHiiPackageListHeader {
    pub package_list_guid: Guid,
    pub horizontal_resolution: u32,
}

#[derive(Clone, Copy)]
pub struct UndefineHiiPackageListHeader {
    // Location in memory
    location: *const u8,
}

impl UndefineHiiPackageListHeader {
    // Get a pointer to the structure
    pub fn as_ptr(&self) -> *mut u8 {
        self.location as *mut u8
    }

    pub fn header(&self) -> EfiHiiPackageListHeader {
        unsafe { ptr::read_unaligned(self.location as *const EfiHiiPackageListHeader) }
    }
}

#[derive(Clone, Copy)]
/// Iterator for events in  [`UndefineHiiPackageListHeader`].
pub struct UndefineHiiPackageListIter {
    // Pointer to the location UndefineHiiPackageListHeader
    pub location: *const u8,

    pub size: u32,
}

impl UndefineHiiPackageListIter {
    pub unsafe fn new(start: *const u8, size: u32) -> Self {
        Self {
            location: start,
            size,
        }
    }
}

impl Iterator for UndefineHiiPackageListIter {
    type Item = UndefineHiiPackageListHeader;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.size == 0 {
                return None;
            }

            let hii_package_list_header = UndefineHiiPackageListHeader {
                location: self.location,
            };

            let header = ptr::read_unaligned(self.location as *const EfiHiiPackageListHeader);
            // Get next table
            self.location = self
                .location
                .byte_offset(header.horizontal_resolution as isize);
            self.size = self.size - (header.horizontal_resolution as u32);
            return Some(hii_package_list_header);
        }
    }
}

impl fmt::Display for UndefineHiiPackageListIter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, package_list) in self.enumerate() {
            let _ = write!(
                f,
                "PackageList[{}]: GUID={}; size=0x{:02X}\n",
                i,
                package_list.header().package_list_guid,
                package_list.header().horizontal_resolution
            );
        }
        Ok(())
    }
}
