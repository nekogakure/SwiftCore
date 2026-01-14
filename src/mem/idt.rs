//! IDT管理モジュール
//!
//! IDTの初期化と管理

use crate::{debug, error, mem::gdt, warn};
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Once;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

static IDT: Once<InterruptDescriptorTable> = Once::new();

// タイマー割り込みカウンタ（100回 = 1秒）
static TIMER_TICKS: AtomicU64 = AtomicU64::new(0);

/// IDTを初期化
pub fn init() {
    debug!("Initializing IDT...");

    let idt = IDT.call_once(|| {
        let mut idt = InterruptDescriptorTable::new();

        // CPU例外ハンドラ
        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.debug.set_handler_fn(debug_handler);
        idt.non_maskable_interrupt.set_handler_fn(nmi_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.overflow.set_handler_fn(overflow_handler);
        idt.bound_range_exceeded
            .set_handler_fn(bound_range_exceeded_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.device_not_available
            .set_handler_fn(device_not_available_handler);

        // ダブルフォルトハンドラ（専用スタック使用）
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        idt.segment_not_present
            .set_handler_fn(segment_not_present_handler);
        idt.stack_segment_fault
            .set_handler_fn(stack_segment_fault_handler);
        idt.general_protection_fault
            .set_handler_fn(general_protection_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.x87_floating_point
            .set_handler_fn(x87_floating_point_handler);
        idt.alignment_check.set_handler_fn(alignment_check_handler);
        idt.machine_check.set_handler_fn(machine_check_handler);
        idt.simd_floating_point
            .set_handler_fn(simd_floating_point_handler);
        idt.virtualization.set_handler_fn(virtualization_handler);

        // ハードウェア割り込みハンドラ（32-47番）
        idt[32].set_handler_fn(timer_interrupt_handler); // Timer
        idt[33].set_handler_fn(keyboard_interrupt_handler); // Keyboard

        // それ以外のハードウェア割り込みはとりあえずスタブ
        for i in 34..48 {
            idt[i].set_handler_fn(generic_interrupt_handler);
        }

        // 48-255番も念のため設定（未使用の割り込みベクタ）
        for i in 48..=255 {
            idt[i].set_handler_fn(generic_interrupt_handler);
        }

        idt
    });

    idt.load();

    // IDTが正しくロードされたか確認
    use x86_64::instructions::tables::sidt;
    let idtr = sidt();
    debug!(
        "IDT loaded: base={:p}, limit={}",
        idtr.base.as_ptr::<u8>(),
        idtr.limit
    );
}

extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    error!("EXCEPTION: DIVIDE ERROR");
    debug!("{:#?}", stack_frame);
    halt_cpu();
}

extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    debug!("EXCEPTION: DEBUG");
    debug!("{:#?}", stack_frame);
}

extern "x86-interrupt" fn nmi_handler(stack_frame: InterruptStackFrame) {
    error!("EXCEPTION: NON-MASKABLE INTERRUPT");
    debug!("{:#?}", stack_frame);
    halt_cpu();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    warn!("EXCEPTION: BREAKPOINT");
    debug!("{:#?}", stack_frame);
}

extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    error!("EXCEPTION: OVERFLOW");
    debug!("{:#?}", stack_frame);
    halt_cpu();
}

extern "x86-interrupt" fn bound_range_exceeded_handler(stack_frame: InterruptStackFrame) {
    error!("EXCEPTION: BOUND RANGE EXCEEDED");
    debug!("{:#?}", stack_frame);
    halt_cpu();
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    error!("EXCEPTION: INVALID OPCODE");
    debug!("{:#?}", stack_frame);
    halt_cpu();
}

extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    error!("EXCEPTION: DEVICE NOT AVAILABLE");
    debug!("{:#?}", stack_frame);
    halt_cpu();
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    error!("EXCEPTION: DOUBLE FAULT");
    error!("Error code: {:#x}", error_code);
    debug!("{:#?}", stack_frame);
    halt_forever();
}

extern "x86-interrupt" fn invalid_tss_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    error!("EXCEPTION: INVALID TSS");
    error!("Error code: {:#x}", error_code);
    debug!("{:#?}", stack_frame);
    halt_cpu();
}

extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    error!("EXCEPTION: SEGMENT NOT PRESENT");
    error!("Error code: {:#x}", error_code);
    debug!("{:#?}", stack_frame);
    halt_cpu();
}

extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    error!("EXCEPTION: STACK SEGMENT FAULT");
    error!("Error code: {:#x}", error_code);
    debug!("{:#?}", stack_frame);
    halt_cpu();
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    error!("EXCEPTION: GENERAL PROTECTION FAULT");
    error!("Error code: {:#x}", error_code);
    debug!("{:#?}", stack_frame);
    halt_cpu();
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: x86_64::structures::idt::PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    error!("EXCEPTION: PAGE FAULT");
    error!("Accessed address: {:?}", Cr2::read());
    error!("Error code: {:?}", error_code);
    debug!("{:#?}", stack_frame);
    halt_cpu();
}

extern "x86-interrupt" fn x87_floating_point_handler(stack_frame: InterruptStackFrame) {
    error!("EXCEPTION: X87 FLOATING POINT");
    debug!("{:#?}", stack_frame);
    halt_cpu();
}

extern "x86-interrupt" fn alignment_check_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    error!("EXCEPTION: ALIGNMENT CHECK");
    error!("Error code: {:#x}", error_code);
    debug!("{:#?}", stack_frame);
    halt_cpu();
}

extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
    error!("EXCEPTION: MACHINE CHECK");
    debug!("{:#?}", stack_frame);
    halt_forever();
}

extern "x86-interrupt" fn simd_floating_point_handler(stack_frame: InterruptStackFrame) {
    error!("EXCEPTION: SIMD FLOATING POINT");
    debug!("{:#?}", stack_frame);
    halt_cpu();
}

extern "x86-interrupt" fn virtualization_handler(stack_frame: InterruptStackFrame) {
    error!("EXCEPTION: VIRTUALIZATION");
    debug!("{:#?}", stack_frame);
    halt_cpu();
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // タイマーカウンタを増加
    let _ticks = TIMER_TICKS.fetch_add(1, Ordering::Relaxed);

    // 割り込みコンテキストではログ出力を避ける（VGAバッファへのアクセスが競合する可能性）
    // TODO: 割り込み安全なロギング機構を実装

    // End of Interrupt (EOI) 信号をPICに送信
    send_eoi(32);
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    debug!("INTERRUPT: KEYBOARD");
    // キーボード入力を処理
    // TODO: キーボードドライバ実装
    send_eoi(33);
}

extern "x86-interrupt" fn generic_interrupt_handler(_stack_frame: InterruptStackFrame) {
    debug!("INTERRUPT: GENERIC");
    // EOIを送信
    unsafe {
        PIC_SLAVE.end_of_interrupt();
        PIC_MASTER.end_of_interrupt();
    }
}

/// マスタPIC
struct Pic {
    offset: u8,
    command: u16,
    data: u16,
}

impl Pic {
    unsafe fn end_of_interrupt(&self) {
        use x86_64::instructions::port::Port;
        Port::new(self.command).write(0x20u8);
    }
}

const PIC_MASTER: Pic = Pic {
    offset: 32,
    command: 0x20,
    data: 0x21,
};

const PIC_SLAVE: Pic = Pic {
    offset: 40,
    command: 0xa0,
    data: 0xa1,
};

/// PITを停止（UEFI起動時の状態をクリア）
pub fn disable_pit() {
    debug!("Disabling PIT...");
    unsafe {
        use x86_64::instructions::port::Port;

        // Channel 0を停止（one-shot mode、カウント0）
        Port::<u8>::new(0x43).write(0x30);
        Port::<u8>::new(0x40).write(0x00);
        Port::<u8>::new(0x40).write(0x00);
        // Channel 1,2も停止
        Port::<u8>::new(0x43).write(0x70); // Channel 1
        Port::<u8>::new(0x41).write(0x00);
        Port::<u8>::new(0x41).write(0x00);

        Port::<u8>::new(0x43).write(0xb0); // Channel 2
        Port::<u8>::new(0x42).write(0x00);
        Port::<u8>::new(0x42).write(0x00);
    }
    debug!("PIT disabled");
}

/// PITを初期化して10ms周期のタイマー割り込みを設定
pub fn init_pit() {
    debug!("Initializing PIT for 10ms timer interrupt...");
    unsafe {
        use x86_64::instructions::port::Port;

        // PIT base frequency: 1.193182 MHz
        // 10ms = 100 Hz
        // Divisor = 1193182 / 100 = 11932 (0x2E9C)
        let divisor: u16 = 11932;

        // Channel 0, LSB+MSB, Mode 2 (rate generator), Binary
        Port::<u8>::new(0x43).write(0x34);

        // IO待機
        for _ in 0..100 {
            core::hint::spin_loop();
        }

        // LSBを送信
        Port::<u8>::new(0x40).write((divisor & 0xff) as u8);

        // IO待機
        for _ in 0..100 {
            core::hint::spin_loop();
        }

        // MSBを送信
        Port::<u8>::new(0x40).write(((divisor >> 8) & 0xff) as u8);
    }
    debug!("PIT configured for 10ms interrupts");
}

/// タイマー割り込み（IRQ0）を有効化
pub fn enable_timer_interrupt() {
    debug!("Enabling timer interrupt (IRQ0)...");
    unsafe {
        use x86_64::instructions::port::Port;

        // PIC master のIRQ0のマスクを解除（ビット0を0にする）
        // 他の割り込みは全てマスク（0xfe = 11111110）
        Port::<u8>::new(0x21).write(0xfe);

        // IO待機
        for _ in 0..1000 {
            core::hint::spin_loop();
        }
    }
    debug!("Timer interrupt enabled");
}

pub fn init_pic() {
    debug!("Initializing PIC (8259A)...");

    unsafe {
        use x86_64::instructions::port::Port;

        // 先にすべての割り込みをマスク
        Port::<u8>::new(PIC_MASTER.data).write(0xffu8);
        Port::<u8>::new(PIC_SLAVE.data).write(0xffu8);
        for _ in 0..1000 {
            core::hint::spin_loop();
        }

        // ICW1: Initialize
        Port::new(PIC_MASTER.command).write(0x11u8);
        for _ in 0..100 {
            core::hint::spin_loop();
        }
        Port::new(PIC_SLAVE.command).write(0x11u8);
        for _ in 0..100 {
            core::hint::spin_loop();
        }

        // ICW2: Vector offset
        Port::new(PIC_MASTER.data).write(PIC_MASTER.offset);
        for _ in 0..100 {
            core::hint::spin_loop();
        }
        Port::new(PIC_SLAVE.data).write(PIC_SLAVE.offset);
        for _ in 0..100 {
            core::hint::spin_loop();
        }

        // ICW3: Cascade
        Port::new(PIC_MASTER.data).write(4u8); // Slave on IRQ2
        for _ in 0..100 {
            core::hint::spin_loop();
        }
        Port::new(PIC_SLAVE.data).write(2u8); // Cascade identity
        for _ in 0..100 {
            core::hint::spin_loop();
        }

        // ICW4: 8086 mode
        Port::new(PIC_MASTER.data).write(0x01u8);
        for _ in 0..100 {
            core::hint::spin_loop();
        }
        Port::new(PIC_SLAVE.data).write(0x01u8);
        for _ in 0..100 {
            core::hint::spin_loop();
        }

        // 再度すべての割り込みをマスク（念のため）
        Port::<u8>::new(PIC_MASTER.data).write(0xffu8);
        for _ in 0..100 {
            core::hint::spin_loop();
        }
        Port::<u8>::new(PIC_SLAVE.data).write(0xffu8);
        for _ in 0..100 {
            core::hint::spin_loop();
        }

        // EOIを送信して保留中の割り込みをクリア
        Port::<u8>::new(PIC_MASTER.command).write(0x20u8);
        Port::<u8>::new(PIC_SLAVE.command).write(0x20u8);
    }

    debug!("PIC initialized, all interrupts masked");
}

/// CPU割り込みを無効化してシステムを停止
fn halt_cpu() {
    x86_64::instructions::interrupts::disable();
    loop {
        x86_64::instructions::hlt();
    }
}

/// CPU割り込みを無効化してシステムを停止（戻らない）
fn halt_forever() -> ! {
    x86_64::instructions::interrupts::disable();
    loop {
        x86_64::instructions::hlt();
    }
}

/// PICにEnd of Interrupt信号を送信
fn send_eoi(interrupt_id: u8) {
    unsafe {
        if interrupt_id >= 40 {
            // Slave PIC
            PIC_SLAVE.end_of_interrupt();
        }
        // Master PIC
        PIC_MASTER.end_of_interrupt();
    }
}
