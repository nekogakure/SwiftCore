//! カーネルエントリーポイント

use crate::{mem, sprintln, util, vprintln, BootInfo, MemoryRegion};

/// カーネルエントリーポイント
#[no_mangle]
pub extern "C" fn kmain(boot_info: &'static BootInfo) -> ! {
    util::console::init();

    // フレームバッファ初期化
    util::vga::init(
        boot_info.framebuffer_addr,
        boot_info.screen_width,
        boot_info.screen_height,
        boot_info.stride,
    );

    vprintln!("=== SwiftCore Kernel v0.1.0 ===");
    vprintln!("Framebuffer: {:#x}", boot_info.framebuffer_addr);
    vprintln!(
        "Resolution: {}x{}",
        boot_info.screen_width,
        boot_info.screen_height
    );
    vprintln!("");

    sprintln!(
        "Physical memory offset: {:#x}",
        boot_info.physical_memory_offset
    );

    // メモリマップを取得
    let memory_map = unsafe {
        core::slice::from_raw_parts(
            boot_info.memory_map_addr as *const MemoryRegion,
            boot_info.memory_map_len,
        )
    };

    sprintln!("Memory map entries: {}", boot_info.memory_map_len);
    for (i, region) in memory_map.iter().enumerate() {
        sprintln!(
            "  Region {}: {:#x} - {:#x} ({:?})",
            i,
            region.start,
            region.start + region.len,
            region.region_type
        );
    }

    // メモリ管理初期化
    mem::init(boot_info.physical_memory_offset);
    mem::init_frame_allocator(memory_map);

    sprintln!("Kernel ready");
    vprintln!("Kernel ready - entering idle loop...");

    // hlt前に明示的にメッセージを出力
    sprintln!("Entering HLT loop...");
    vprintln!("Entering HLT loop...");

    // 最初のhlt命令を実行
    sprintln!("Executing first HLT");
    x86_64::instructions::hlt();
    
    // hlt後に戻ってきた場合（割り込みなど）
    sprintln!("Returned from HLT");
    
    loop {
        x86_64::instructions::hlt();
    }
}
