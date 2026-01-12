#![no_std]
#![no_main]

extern crate alloc;

use swiftcore::{kmain, BootInfo};
use uefi::prelude::*;

#[global_allocator]
static ALLOCATOR: uefi::allocator::Allocator = uefi::allocator::Allocator;

#[path = "../handler.rs"]
mod handler;

static BOOT_INFO: BootInfo = BootInfo {
    physical_memory_offset: 0,
};

/// UEFIエントリーポイント
#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi::helpers::init(&mut system_table).expect("Failed to initialize UEFI services");

    let rev = system_table.uefi_revision();

    log::debug!("SwiftBoot launched");
    log::debug!("UEFI Version: {}.{}", rev.major(), rev.minor());
    log::debug!("Firmware Vendor: {}", system_table.firmware_vendor());
    log::debug!("Firmware Revision: {}", system_table.firmware_revision());
    log::debug!("Boot services initialized");

    log::debug!("Transferring control to kernel...");
    log::debug!("");

    kmain(&BOOT_INFO);
}
