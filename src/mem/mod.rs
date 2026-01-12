//! メモリ管理モジュール
//!
//! GDT、TSS、ページング

pub mod gdt;
pub mod paging;
pub mod tss;

pub fn init(physical_memory_offset: u64) {
    log::info!("Initializing memory...");

    gdt::init();
    paging::init(physical_memory_offset);

    log::info!("Memory initialized");
}
