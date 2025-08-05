#![no_std]
use uefi::Guid;
use uefi::boot::MemoryType;
use uefi::prelude::*;
use uefi::proto::unsafe_protocol;

extern crate alloc;
use alloc::slice;
use alloc::vec::Vec;

use core::ffi::c_void;
use core::mem;
use core::ptr;

use crate::package_header::*;

pub mod package_header;
pub mod base;

/// EFI_HII_DATABASE_NOTIFY_TYPE.
type EfiHiiDatabaseNotifyType = usize;

/// Functions which are registered to receive notification of
/// database events have this prototype. The actual event is encoded
/// in NotifyType. The following table describes how PackageType,
/// PackageGuid, Handle, and Package are used for each of the
/// notification types.
pub type EfiHiiDatabaseNotify = unsafe extern "efiapi" fn(
    package_type: u8,
    package_guid: *const Guid,
    package: *const EfiHiiPackageHeader,
    handler: Handle,
    notify_type: EfiHiiDatabaseNotifyType,
) -> Status;

#[repr(C)]
struct EfiHiiKeyboardLayout {
    layout_length: u16,
    guid: Guid,
    layout_descriptor_string_offset: u32,
    descriptor_count: u8,
}

/// HII Configuration Processing and Browser Protocol.
#[derive(Debug)]
#[repr(C)]
#[unsafe_protocol("ef9fc172-a1b2-4693-b327-6d32fc416042")]
pub struct HiiDatabaseProtocol {
    new_package_list: unsafe extern "efiapi" fn(
        *const Self,
        package_list: *const EfiHiiPackageListHeader,
        driver_handle: *mut c_void,
        handler: *mut *mut c_void,
    ) -> Status,
    remove_package_list: unsafe extern "efiapi" fn(*const Self, handler: Handle) -> Status,
    update_package_list: unsafe extern "efiapi" fn(
        *const Self,
        handle: Handle,
        package_list: *const EfiHiiPackageListHeader,
    ) -> Status,
    list_package_lists: unsafe extern "efiapi" fn(
        *const Self,
        package_type: u8,
        package_guid: *const Guid,
        handle_buffer_length: *mut usize,
        handle: *mut Handle,
    ) -> Status,
    export_package_lists: unsafe extern "efiapi" fn(
        *const Self,
        handle: *mut c_void,
        buffer_size: *mut usize,
        buffer: *mut EfiHiiPackageListHeader,
    ) -> Status,
    register_package_notify: unsafe extern "efiapi" fn(
        *const Self,
        package_type: u8,
        package_guid: *const Guid,
        package_notify_fn: EfiHiiDatabaseNotify,
        notify_type: EfiHiiDatabaseNotifyType,
        notify_handle: *mut Handle,
    ) -> Status,
    unregister_package_notify:
        unsafe extern "efiapi" fn(*const Self, notification_handle: Handle) -> Status,
    find_keyboard_layouts: unsafe extern "efiapi" fn(
        *const Self,
        key_guid_buffer_length: *mut u16,
        key_guid_buffer: *mut Guid,
    ) -> Status,
    get_keyboard_layout: unsafe extern "efiapi" fn(
        *const Self,
        key_guid: *const Guid,
        keyboard_layout_length: *mut u16,
        keyboard_layout: *mut EfiHiiKeyboardLayout,
    ) -> Status,
    set_keyboard_layout: unsafe extern "efiapi" fn(*const Self, key_guid: *const Guid) -> Status,
    get_package_list_handle: unsafe extern "efiapi" fn(
        *const Self,
        package_list_handle: Handle,
        driver_handle: *mut Handle,
    ) -> Status,
}

impl HiiDatabaseProtocol {
    /// Get all list packages HII
    pub fn get_hii_package_lists(&self) -> Option<Vec<UndefineHiiPackageListHeader>> {
        let mut package_size: usize = 0;
        let mut package_list: EfiHiiPackageListHeader = Default::default();
        let handle: *mut c_void = ptr::null_mut();

        // Find out the size of the table
        let status = unsafe {
            (self.export_package_lists)(self, handle, &mut package_size, &mut package_list)
        };
        if status != Status::BUFFER_TOO_SMALL {
            return None;
        }
        // Allocate memory for Hii packages
        let package_list =
            boot::allocate_pool(MemoryType::BOOT_SERVICES_DATA, package_size).ok()?;

        let status = unsafe {
            let pl = &mut *(package_list.as_ptr() as *mut EfiHiiPackageListHeader);
            (self.export_package_lists)(self, handle, &mut package_size, pl)
        };

        if status.is_success() {
            // Convert a raw pointer to an iterator and traverse, collecting packets into a vector”
            let hii_package_list = unsafe {
                UndefineHiiPackageListIter::new(package_list.as_ptr(), package_size as u32)
            };

            let vec_list: Vec<UndefineHiiPackageListHeader> =
                hii_package_list.into_iter().collect();
            return Some(vec_list);
        }
        None
    }

    /// Get list packages HII
    fn get_hii_handles(&self, package_list_guid: Guid) -> Option<UndefineHiiPackageListHeader> {
        let mut package_size: usize = 0;
        let mut package_list: EfiHiiPackageListHeader = Default::default();
        let handle: *mut c_void = ptr::null_mut();

        // Find out the size of the table
        let status = unsafe {
            (self.export_package_lists)(self, handle, &mut package_size, &mut package_list)
        };
        if status != Status::BUFFER_TOO_SMALL {
            return None;
        }
        // Allocate memory for Hii packages
        let package_list =
            boot::allocate_pool(MemoryType::BOOT_SERVICES_DATA, package_size).ok()?;

        let status = unsafe {
            let pl = &mut *(package_list.as_ptr() as *mut EfiHiiPackageListHeader);
            (self.export_package_lists)(self, handle, &mut package_size, pl)
        };

        if status.is_success() {
            let hii_package_list = unsafe {
                UndefineHiiPackageListIter::new(package_list.as_ptr(), package_size as u32)
            };

            return hii_package_list
                .into_iter()
                .find(|undef_hii| undef_hii.header().package_list_guid == package_list_guid);
        }
        None
    }

    /// Get Hii packeges for handle
    pub fn get_hii_package(&self, handle: Handle) -> Option<UndefineHiiPackageListHeader>  {
        let mut package_size: usize = 0;
        let mut package_list: EfiHiiPackageListHeader = Default::default();

        // Find out the size of the table
        let status = unsafe {
            (self.export_package_lists)(self, handle.as_ptr(), &mut package_size, &mut package_list)
        };
        if status != Status::BUFFER_TOO_SMALL {
            return None;
        }

        // Allocate memory for Hii packages
        let package_list =
            boot::allocate_pool(MemoryType::BOOT_SERVICES_CODE, package_size).ok()?;

        let status = unsafe {
            let pl = &mut *(package_list.as_ptr() as *mut EfiHiiPackageListHeader);
            (self.export_package_lists)(self, handle.as_ptr(), &mut package_size, pl)
        };

        if status.is_success() {
            let hii_package_list = unsafe {
                UndefineHiiPackageListIter::new(package_list.as_ptr(), package_size as u32)
            };

            return hii_package_list.into_iter().next();
        }
        None
    }

    /// Get vector packages for list
    pub fn get_package<T: PackageHeader>(&self, package_list_guid: Guid) -> Option<Vec<T>> {
        self.get_hii_handles(package_list_guid).map(|handles| {
            handles
                .into_iter()
                .filter(|head| head.header.get_type() == Ok(T::PACKAGE_TYPE))
                .map(|t| T::from_undef(&t))
                .collect()
        })
    }

    /// Registers a list of packages in the HII Database and returns the HII Handle
    pub fn add_packages(
        &self,
        package_list_guid: Guid,
        device_hadle: Option<Handle>,
        packegs: Vec<HiiPackage>,
    ) -> (Status, Option<Handle>) {
        let size_packegs: usize = packegs.iter().map(|pack| pack.size()).sum();

        // Fill in the GUIDE and Length of the Package List Header
        let list_header: EfiHiiPackageListHeader = EfiHiiPackageListHeader {
            package_list_guid,
            horizontal_resolution: (size_packegs + mem::size_of::<EfiHiiPackageListHeader>())
                as u32,
        };

        let bytes_list_header = unsafe {
            slice::from_raw_parts(
                (&list_header as *const EfiHiiPackageListHeader) as *const u8,
                mem::size_of::<EfiHiiPackageListHeader>(),
            )
        };

        // Initialize all byte data into one packet
        let mut list_pack = Vec::with_capacity(list_header.horizontal_resolution as usize);
        list_pack.extend_from_slice(bytes_list_header);
        // Copy the data from each package
        for packeg in packegs {
            list_pack.extend_from_slice(packeg.as_slice());
        }

        // Register the package list with the HII Database
        unsafe {
            let driver_handle: *mut c_void = match device_hadle {
                Some(h) => h.as_ptr(),
                None => ptr::null_mut(),
            };
            let mut ptr_handle: *mut c_void = ptr::null_mut();

            let status = (self.new_package_list)(
                self,
                list_pack.as_ptr() as *const EfiHiiPackageListHeader,
                driver_handle,
                &mut ptr_handle,
            );
            let handle = Handle::from_ptr(ptr_handle);

            (status, handle)
        }
    }

    /// Removes a package list from the HII database.
    pub fn remove_packages(&self, hii_handle: Handle) -> Status {
        unsafe { (self.remove_package_list)(self, hii_handle) }
    }
}
 