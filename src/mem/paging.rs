//! ページング管理モジュール
//!
//! 仮想メモリとページテーブル管理

use crate::sprintln;
use spin::Mutex;
use x86_64::{
    structures::paging::{FrameAllocator, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB},
    VirtAddr,
};

static PAGE_TABLE: Mutex<Option<OffsetPageTable<'static>>> = Mutex::new(None);

/// ページングシステムを初期化
pub fn init(_physical_memory_offset: u64) {
    sprintln!("Initializing paging...");
    // TODO: ページテーブルの取得と操作は後で実装
    sprintln!("Paging initialized");
}

#[allow(unused)]
/// アクティブなレベル4ページテーブルへの参照を取得
unsafe fn active_level_4_table(physical_memory_offset: u64) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = VirtAddr::new(phys.as_u64() + physical_memory_offset);
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

/// ページをマップ
pub fn map_page(
    _page: Page,
    _frame: PhysFrame,
    _flags: PageTableFlags,
) -> Result<(), &'static str> {
    let mut _page_table = PAGE_TABLE.lock();
    let _page_table = _page_table.as_mut().ok_or("Page table not initialized")?;

    // TODO: フレームアロケータの実装
    // page_table.map_to(page, frame, flags, &mut DummyFrameAllocator)?;

    Ok(())
}

/// ページフラグ
pub use x86_64::structures::paging::PageTableFlags;

/// ダミーフレームアロケータ（将来的に実装）
pub struct DummyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for DummyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        None
    }
}
