#![no_std]
#![no_main]

extern crate alloc;

use swiftcore::{kmain, BootInfo};
use uefi::prelude::*;

#[global_allocator]
static ALLOCATOR: uefi::allocator::Allocator = uefi::allocator::Allocator;

static BOOT_INFO: BootInfo = BootInfo {
    physical_memory_offset: 0,
};

/// UEFIエントリーポイント
#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi::helpers::init(&mut system_table).expect("Failed to initialize UEFI services");

    kmain(&BOOT_INFO);
}
