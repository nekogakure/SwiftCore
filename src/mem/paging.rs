//! ページング管理モジュール
//!
//! 仮想メモリとページテーブル管理

use spin::Mutex;
use x86_64::{
    structures::paging::{FrameAllocator, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB},
    VirtAddr,
};

static PAGE_TABLE: Mutex<Option<OffsetPageTable<'static>>> = Mutex::new(None);

/// ページングシステムを初期化
pub fn init(physical_memory_offset: u64) {
    log::info!("Initializing paging...");
    log::info!("Physical memory offset: {:#x}", physical_memory_offset);

    unsafe {
        let level_4_table = active_level_4_table(physical_memory_offset);

        // ページテーブルの概要を出力
        log::debug!("Page table summary:");
        let mut active_entries = 0;
        for (i, entry) in level_4_table.iter().enumerate() {
            if !entry.is_unused() {
                active_entries += 1;
                log::debug!("  L4[{}]: {:?}", i, entry.flags());
            }
        }
        log::debug!("Active L4 entries: {}/512", active_entries);

        let phys_offset = VirtAddr::new(physical_memory_offset);
        let page_table = OffsetPageTable::new(level_4_table, phys_offset);

        *PAGE_TABLE.lock() = Some(page_table);
    }

    log::info!("Paging initialized successfully");
}

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
