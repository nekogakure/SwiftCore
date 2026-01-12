#![no_std]
#![feature(abi_x86_interrupt)]

use log;
pub mod mem;

#[repr(C)]
pub struct BootInfo {
    /// 物理メモリオフセット
    pub physical_memory_offset: u64,
}

/// カーネルエントリーポイント
#[no_mangle]
pub extern "C" fn kmain(boot_info: &'static BootInfo) -> ! {
    log::info!("");
    log::info!("=================================");
    log::info!("  SwiftCore Kernel v0.1.0");
    log::info!("=================================");
    log::info!("");
    log::info!("Initializing kernel subsystems...");
    log::info!(
        "Physical memory offset: {:#x}",
        boot_info.physical_memory_offset
    );
    log::info!("");

    // メモリ管理初期化
    mem::init(boot_info.physical_memory_offset);

    log::info!("Kernel ready");

    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
