//! カーネルエントリーポイント

#![no_std]

use crate::{mem, sprintln, util, BootInfo};

/// カーネルエントリーポイント
#[no_mangle]
pub extern "C" fn kmain(boot_info: &'static BootInfo) -> ! {
    // シリアルポートを初期化
    util::serial::init();
    sprintln!("=== SwiftCore Kernel v0.1.0 ===");
    sprintln!("Serial output initialized");
    sprintln!(
        "Physical memory offset: {:#x}",
        boot_info.physical_memory_offset
    );

    // メモリ管理初期化
    mem::init(boot_info.physical_memory_offset);

    sprintln!("Kernel ready");

    // 割り込みを無効にしてメインループへ
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("cli"); // 割り込み無効化
    }

    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
