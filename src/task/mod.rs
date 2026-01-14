//! タスク管理モジュール
//!
//! マルチタスク機能を提供（プロセスとスレッドの管理）

use crate::interrupt::spinlock::SpinLock;
use core::sync::atomic::{AtomicU64, Ordering};

/// プロセスID生成用カウンタ
static NEXT_PROCESS_ID: AtomicU64 = AtomicU64::new(1);

/// スレッドID生成用カウンタ
static NEXT_THREAD_ID: AtomicU64 = AtomicU64::new(1);

/// プロセスID
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcessId(u64);

impl ProcessId {
    /// 新しいプロセスIDを生成
    pub fn new() -> Self {
        Self(NEXT_PROCESS_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// プロセスIDの値を取得
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// スレッドID
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ThreadId(u64);

impl ThreadId {
    /// 新しいスレッドIDを生成
    pub fn new() -> Self {
        Self(NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// スレッドIDの値を取得
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// スレッドの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadState {
    /// 実行可能（スケジューラ待ち）
    Ready,
    /// 実行中
    Running,
    /// ブロック中（I/O待ちなど）
    Blocked,
    /// スリープ中
    Sleeping,
    /// 終了済み
    Terminated,
}

/// プロセスの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    /// 実行中（少なくとも1つのスレッドがRunning/Ready）
    Running,
    /// スリープ中（すべてのスレッドがSleeping）
    Sleeping,
    /// ゾンビ（終了したが親に回収されていない）
    Zombie,
    /// 終了済み
    Terminated,
}

/// タスクが保有する権限レベル。ServiceとUserは区別のためであり、両方ともRing3で動作する。
///
/// - Core: カーネルモード（Ring0）で動作するタスク。システムの中核機能を担当。
/// - Service: ユーザーモード（Ring3）で動作するが、システムサービスやドライバを担当。
/// - User: ユーザーモード（Ring3）で動作。一般的なアプリケーションを担当。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivilegeLevel {
    /// コアレベルタスク（Ring0）
    Core,
    /// サービスレベルタスク（Ring3）
    Service,
    /// ユーザーレベルタスク（Ring3）
    User,
}

/// CPUコンテキスト（レジスタ保存用）
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Context {
    /// スタックポインタ
    pub rsp: u64,
    /// ベースポインタ
    pub rbp: u64,
    /// Callee-saved レジスタ
    pub rbx: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    /// 命令ポインタ（戻り先アドレス）
    pub rip: u64,
    /// RFLAGSレジスタ
    pub rflags: u64,
}

impl Context {
    /// 新しいコンテキストを作成
    pub const fn new() -> Self {
        Self {
            rsp: 0,
            rbp: 0,
            rbx: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip: 0,
            rflags: 0,
        }
    }
}

/// プロセス構造体
///
/// メモリ空間とリソースを管理する実行単位。
/// 1つ以上のスレッドを持つ。
pub struct Process {
    /// プロセスID
    id: ProcessId,
    /// プロセス名
    name: &'static str,
    /// プロセスの状態
    state: ProcessState,
    /// 権限レベル
    privilege: PrivilegeLevel,
    /// 親プロセスID（存在する場合）
    parent_id: Option<ProcessId>,
    /// ページテーブルのアドレス（メモリ空間）。Noneの場合はカーネル空間を共有。
    page_table: Option<u64>,
    /// 優先度（0が最高、値が大きいほど低い）
    priority: u8,
}

impl Process {
    /// 新しいプロセスを作成
    ///
    /// # Arguments
    /// * `name` - プロセス名
    /// * `privilege` - 権限レベル
    /// * `parent_id` - 親プロセスID
    /// * `priority` - プロセスの優先度
    pub fn new(
        name: &'static str,
        privilege: PrivilegeLevel,
        parent_id: Option<ProcessId>,
        priority: u8,
    ) -> Self {
        Self {
            id: ProcessId::new(),
            name,
            state: ProcessState::Running,
            privilege,
            parent_id,
            page_table: None, // TODO: ページテーブル実装後に設定
            priority,
        }
    }

    /// プロセスIDを取得
    pub fn id(&self) -> ProcessId {
        self.id
    }

    /// プロセス名を取得
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// プロセスの状態を取得
    pub fn state(&self) -> ProcessState {
        self.state
    }

    /// プロセスの状態を設定
    pub fn set_state(&mut self, state: ProcessState) {
        self.state = state;
    }

    /// 権限レベルを取得
    pub fn privilege(&self) -> PrivilegeLevel {
        self.privilege
    }

    /// 親プロセスIDを取得
    pub fn parent_id(&self) -> Option<ProcessId> {
        self.parent_id
    }

    /// 優先度を取得
    pub fn priority(&self) -> u8 {
        self.priority
    }

    /// ページテーブルアドレスを取得
    pub fn page_table(&self) -> Option<u64> {
        self.page_table
    }

    /// ページテーブルアドレスを設定
    pub fn set_page_table(&mut self, page_table: u64) {
        self.page_table = Some(page_table);
    }
}

impl core::fmt::Debug for Process {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut debug_struct = f.debug_struct("Process");
        debug_struct
            .field("id", &self.id)
            .field("name", &self.name)
            .field("state", &self.state)
            .field("privilege", &self.privilege)
            .field("parent_id", &self.parent_id)
            .field("priority", &self.priority);

        if let Some(pt) = self.page_table {
            debug_struct.field("page_table", &format_args!("{:#x}", pt));
        } else {
            debug_struct.field("page_table", &None::<u64>);
        }

        debug_struct.finish()
    }
}

/// スレッド構造体
///
/// プロセス内で実行される軽量な実行単位。
/// 同じプロセス内のスレッドはメモリ空間を共有する。
pub struct Thread {
    /// スレッドID
    id: ThreadId,
    /// 所属するプロセスID
    process_id: ProcessId,
    /// スレッド名
    name: &'static str,
    /// 現在の状態
    state: ThreadState,
    /// CPUコンテキスト
    context: Context,
    /// カーネルスタックの開始アドレス
    kernel_stack: u64,
    /// カーネルスタックのサイズ
    kernel_stack_size: usize,
}

impl Thread {
    /// 新しいスレッドを作成
    ///
    /// # Arguments
    /// * `process_id` - 所属するプロセスID
    /// * `name` - スレッド名
    /// * `entry_point` - スレッドのエントリーポイント関数
    /// * `kernel_stack` - カーネルスタックのアドレス
    /// * `kernel_stack_size` - カーネルスタックのサイズ
    pub fn new(
        process_id: ProcessId,
        name: &'static str,
        entry_point: fn() -> !,
        kernel_stack: u64,
        kernel_stack_size: usize,
    ) -> Self {
        let mut context = Context::new();

        // スタックポインタをスタックの最後に設定（スタックは下に伸びる）
        context.rsp = kernel_stack + kernel_stack_size as u64;
        context.rbp = context.rsp;

        // エントリーポイントをripに設定
        context.rip = entry_point as u64;

        // RFLAGSの初期値（割り込み有効）
        context.rflags = 0x202; // IF (Interrupt Flag) = 1

        Self {
            id: ThreadId::new(),
            process_id,
            name,
            state: ThreadState::Ready,
            context,
            kernel_stack,
            kernel_stack_size,
        }
    }

    /// スレッドIDを取得
    pub fn id(&self) -> ThreadId {
        self.id
    }

    /// 所属するプロセスIDを取得
    pub fn process_id(&self) -> ProcessId {
        self.process_id
    }

    /// スレッド名を取得
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// スレッドの状態を取得
    pub fn state(&self) -> ThreadState {
        self.state
    }

    /// スレッドの状態を設定
    pub fn set_state(&mut self, state: ThreadState) {
        self.state = state;
    }

    /// コンテキストへの可変参照を取得
    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }

    /// コンテキストへの参照を取得
    pub fn context(&self) -> &Context {
        &self.context
    }
}

impl core::fmt::Debug for Thread {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Thread")
            .field("id", &self.id)
            .field("process_id", &self.process_id)
            .field("name", &self.name)
            .field("state", &self.state)
            .field("kernel_stack", &format_args!("{:#x}", self.kernel_stack))
            .field("kernel_stack_size", &self.kernel_stack_size)
            .finish()
    }
}

// TODO: プロセステーブルの実装
// TODO: スレッドキューの実装
// TODO: スケジューラの実装
// TODO: コンテキストスイッチの実装
