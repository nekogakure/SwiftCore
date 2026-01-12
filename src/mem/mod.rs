//! メモリ管理モジュール
//!
//! ページング

pub mod paging;

pub fn init(physical_memory_offset: u64) {
    log::info!("Initializing memory...");

    paging::init(physical_memory_offset);

    log::info!("Memory initialized");
}
