#![no_std]
#![no_main]

use uefi::prelude::*;

#[global_allocator]
static ALLOCATOR: uefi::allocator::Allocator = uefi::allocator::Allocator;

/// UEFIエントリーポイント
#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi::helpers::init(&mut system_table).expect("Failed to initialize UEFI services");

    // システム情報を表示
    let rev = system_table.uefi_revision();
    log::warn!("UEFI Version: {}.{}", rev.major(), rev.minor());

    // ファームウェアベンダー情報
    log::warn!("Firmware Vendor: {}", system_table.firmware_vendor());
    log::warn!("Firmware Revision: {}", system_table.firmware_revision());
    log::warn!("");
    log::warn!("Boot successful! System is ready.");
    log::warn!("");

    // カーネルを起動する準備
    log::warn!("Preparing to start kernel...");

    // TODO: カーネルに制御を移す
    // kernel_main();

    log::warn!("Bootloader complete. System halted.");

    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("BOOTLOADER PANIC!");
    log::error!("{}", info);
    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
