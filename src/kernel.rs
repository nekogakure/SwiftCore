#![no_std]

use log;

/// カーネルエントリーポイント
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {

    log::info!("");
    log::info!("Hello from SwiftCore Kernel!");
    log::info!("Kernel is now running...");
    log::info!("");

    // カーネルメインループ
    log::warn!("Entering kernel main loop");

    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
