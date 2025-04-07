use uefi::boot::ScopedProtocol;
use uefi::{CStr8, Guid};
use uefi::{print, println};

use crate::*;

fn print_hex_dump(data: &[u8]) {
    for (i, byte) in data.iter().enumerate() {
        if i % 16 == 0 {
            print!("\n{:08x}: ", i);
        }
        print!("{:02x} ", byte);
    }
    println!("");
}

// Print all packages in the system
pub fn show_hii(table: &ScopedProtocol<HiiDatabaseProtocol>) {
    table.get_hii_package_lists().map(|list_pack| {
        for (j, package_list) in list_pack.iter().enumerate() {
            println!(
                "PackageList[{}]: GUID={}; size=0x{:02X}",
                j,
                package_list.header().package_list_guid,
                package_list.header().horizontal_resolution
            );

            for (i, package) in package_list.into_iter().enumerate() {
                println!(
                    "        Package[{}]: type={}; size=0x{:02X}",
                    i,
                    package.package_type(),
                    package.len()
                );
            }
        }
    });
}

// Print all string for package
pub fn hii_strings_uni(table: &ScopedProtocol<HiiDatabaseProtocol>, package_guid: Guid) {
    if let Some(package_string) = table.get_package::<HiiStringPackageHdr>(package_guid) {
        for (index, sph) in package_string.iter().enumerate() {
            println!(" {})'{}' string package", index + 1, sph.str_language());
            let lang_strings = unsafe { CStr8::from_bytes_with_nul_unchecked(sph.language()) };

            for id in 0..sph.count_strings() {
                sph.get_string(id as u16, lang_strings)
                    .map(|string| println!("   ID {} = {:?}", id, string));
            }
        }
    }
}

pub fn show_dump_vfr_form(table: &ScopedProtocol<HiiDatabaseProtocol>, package_guid: Guid) {
    if let Some(package_form) = table.get_package::<HiiFormPackageHdr>(package_guid) {
        println!("Form package Guid: {}\n", package_guid);

        for fph in package_form {
            println!("// PACKAGE HEADER\n");
            println!("{}", fph.header());

            println!("// PACKAGE DATA");
            print_hex_dump(fph.as_data());
        }
    }
}
