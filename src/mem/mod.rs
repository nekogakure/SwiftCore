//! メモリ管理モジュール
//!
//! GDT、TSS、ページング

use crate::{println};

pub mod gdt;
pub mod paging;
pub mod tss;

pub fn init(physical_memory_offset: u64) {
    println!("Initializing memory...");

    paging::init(physical_memory_offset);
    gdt::init();

    println!("Memory initialized");
}
