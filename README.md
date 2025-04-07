# hii_database

This project is an implementation of the HII (Human Interface Infrastructure) support for the UEFI protocol in the Rust programming language. The project is based on the uefi-rs crate, which provides low-level access to the UEFI API.

## How to use?
The base module provides examples of service information output:
```rust
fn main() -> Status {
    uefi::helpers::init().unwrap();

    // Get protocol HiiDatabaseProtocol
    let table = base::locate_protocol::<hii_database::HiiDatabaseProtocol>();    

    hii_database::base::hii_strings_uni(&table, guid!("ce4f5b0c-dc00-4a32-97ed-2966981c7725"));
}
```

## Requirements
* crate uefi-rs(0.34.0)

## Licensing
The project is distributed under the MIT license. See the file for details.
