//! パニックハンドラ
//!
//! カーネルパニック時の処理

use crate::println;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("!!! KERNEL PANIC !!!");
    println!("{}", info);
    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
