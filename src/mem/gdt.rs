//! GDT管理モジュール
//!
//! Global Descriptor Tableを管理

use crate::mem::tss;
use spin::Once;
use x86_64::instructions::segmentation::{Segment, CS};
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};

static GDT: Once<(GlobalDescriptorTable, Selectors)> = Once::new();

/// GDTセレクタ
struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

/// GDTを初期化
pub fn init() {
    log::info!("Initializing GDT...");

    // TSSを初期化
    let tss = tss::init();

    // GDTを初期化
    let (gdt, selectors) = GDT.call_once(|| {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.append(Descriptor::kernel_code_segment());
        let tss_selector = gdt.append(Descriptor::tss_segment(tss));

        log::debug!("GDT entries created:");
        log::debug!("  Code selector: {:?}", code_selector);
        log::debug!("  TSS selector: {:?}", tss_selector);

        (
            gdt,
            Selectors {
                code_selector,
                tss_selector,
            },
        )
    });

    // GDTをロードしてセグメントレジスタを設定
    unsafe {
        gdt.load();
        CS::set_reg(selectors.code_selector);
        load_tss(selectors.tss_selector);
    }

    log::info!("GDT initialized successfully");
}
