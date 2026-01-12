//! パニックハンドラ
//!
//! カーネルパニック時の処理

use crate::sprintln;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    sprintln!("!!! KERNEL PANIC !!!");
    sprintln!("{}", info);
    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
