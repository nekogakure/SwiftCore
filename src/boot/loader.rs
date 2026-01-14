#![no_std]
#![no_main]

extern crate alloc;

use swiftcore::{kernel_entry, BootInfo, MemoryRegion, MemoryType};
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;

#[global_allocator]
static ALLOCATOR: uefi::allocator::Allocator = uefi::allocator::Allocator;

static mut BOOT_INFO: BootInfo = BootInfo {
    physical_memory_offset: 0,
    framebuffer_addr: 0,
    framebuffer_size: 0,
    screen_width: 0,
    screen_height: 0,
    stride: 0,
    memory_map_addr: 0,
    memory_map_len: 0,
    memory_map_entry_size: 0,
};

// メモリマップを静的に保存
static mut MEMORY_MAP: [MemoryRegion; 256] = [MemoryRegion {
    start: 0,
    len: 0,
    region_type: MemoryType::Reserved,
}; 256];

/// UEFIエントリーポイント
#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    if let Err(_) = uefi::helpers::init(&mut system_table) {
        return Status::UNSUPPORTED;
    }

    let _ = system_table.stdout().clear();
    let _ = system_table
        .stdout()
        .output_string(cstr16!("SwiftCore starting...\n"));

    // Graphics Output Protocolを取得
    let gop_handle = match system_table
        .boot_services()
        .get_handle_for_protocol::<GraphicsOutput>()
    {
        Ok(handle) => handle,
        Err(_) => return Status::UNSUPPORTED,
    };

    let mut gop = match system_table
        .boot_services()
        .open_protocol_exclusive::<GraphicsOutput>(gop_handle)
    {
        Ok(gop) => gop,
        Err(_) => return Status::UNSUPPORTED,
    };

    let mode_info = gop.current_mode_info();
    let mut framebuffer = gop.frame_buffer();

    // UEFI 0.30のmemory_map APIを使用してメモリマップを取得
    let memory_map = match system_table
        .boot_services()
        .memory_map(uefi::table::boot::MemoryType::LOADER_DATA)
    {
        Ok(map) => map,
        Err(_) => return Status::OUT_OF_RESOURCES,
    };

    // メモリマップを静的配列にコピー
    let map_count;
    unsafe {
        let mut count = 0;
        for (i, desc) in memory_map.entries().enumerate() {
            if i >= 256 {
                break;
            }
            MEMORY_MAP[i] = MemoryRegion {
                start: desc.phys_start,
                len: desc.page_count * 4096,
                region_type: match desc.ty {
                    uefi::table::boot::MemoryType::CONVENTIONAL => MemoryType::Usable,
                    uefi::table::boot::MemoryType::ACPI_RECLAIM => MemoryType::AcpiReclaimable,
                    uefi::table::boot::MemoryType::ACPI_NON_VOLATILE => MemoryType::AcpiNvs,
                    uefi::table::boot::MemoryType::UNUSABLE => MemoryType::BadMemory,
                    uefi::table::boot::MemoryType::LOADER_CODE
                    | uefi::table::boot::MemoryType::LOADER_DATA => {
                        MemoryType::BootloaderReclaimable
                    }
                    _ => MemoryType::Reserved,
                },
            };
            count += 1;
        }
        map_count = count;
    }

    #[allow(static_mut_refs)]
    unsafe {
        BOOT_INFO.physical_memory_offset = 0;
        BOOT_INFO.framebuffer_addr = framebuffer.as_mut_ptr() as u64;
        BOOT_INFO.framebuffer_size = framebuffer.size();
        BOOT_INFO.screen_width = mode_info.resolution().0;
        BOOT_INFO.screen_height = mode_info.resolution().1;
        BOOT_INFO.stride = mode_info.stride();
        BOOT_INFO.memory_map_addr = MEMORY_MAP.as_ptr() as u64;
        BOOT_INFO.memory_map_len = map_count;
        BOOT_INFO.memory_map_entry_size = core::mem::size_of::<MemoryRegion>();
    }

    unsafe {
        kernel_entry(&*core::ptr::addr_of!(BOOT_INFO));
    }
}
