//! メモリ管理モジュール
//!
//! GDT、TSS、ページング

use crate::sprintln;

pub mod gdt;
pub mod paging;
pub mod tss;

pub fn init(physical_memory_offset: u64) {
    sprintln!("Initializing memory...");

    paging::init(physical_memory_offset);
    gdt::init();

    sprintln!("Memory initialized");
}
