//! TSS管理モジュール
//!
//! TSSを管理

use crate::println;
use spin::Once;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

/// ダブルフォルト用ISTインデックス
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

static TSS: Once<TaskStateSegment> = Once::new();

/// TSSを初期化して返す
#[allow(unused_unsafe)]
pub fn init() -> &'static TaskStateSegment {
    println!("Initializing TSS...");

    TSS.call_once(|| {
        let mut tss = TaskStateSegment::new();

        // ダブルフォルト用の専用スタックを設定
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &raw const STACK });
            let stack_end = stack_start + STACK_SIZE as u64;
            stack_end
        };

        println!("TSS configured with IST[{}] stack", DOUBLE_FAULT_IST_INDEX);
        tss
    })
}
