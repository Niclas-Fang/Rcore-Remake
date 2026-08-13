//! 内核配置常量
//!
//! 集中管理所有可配置的系统参数。
//! 汇编文件 (.S) 和链接脚本 (.ld) 无法直接引用这些常量，
//! 如需修改请确保同步更新对应文件中的硬编码值。

// ─── 内存布局 ───────────────────────────────────────

/// 物理内存中内核起始地址 (linker.ld: ORIGIN)
pub const KERNEL_BASE: usize = 0x80200000;

/// 内核可用内存大小 (linker.ld: LENGTH)
pub const KERNEL_MEMORY_SIZE: usize = 128 * 1024 * 1024; // 128M

/// 用户程序加载基地址 (user/linker.ld: ORIGIN)
pub const USER_BASE: usize = 0x80400000;

/// 用户程序栈顶地址（legacy batch 模式使用）
pub const USER_STACK_TOP: usize = 0x84400000;

/// 每个用户程序占用的地址空间大小
pub const APP_SIZE: usize = 0x400000; // 4MB

/// 最大应用数量
pub const MAX_NUM_APP: usize = 16;

// ─── 栈大小 ─────────────────────────────────────────

/// 每个应用的内核栈大小
pub const KERNEL_STACK_SIZE: usize = 4096 * 2; // 8KiB

/// 每个应用的用户栈大小
pub const USER_STACK_SIZE: usize = 4096; // 4KiB

pub const MEMORY_END: usize = 0x88000000;

pub const USER_BASE_VA: usize = 0x1_0000;

// ─── 堆大小 ─────────────────────────────────────────

pub const KERNEL_HEAP_SIZE: usize = 0x200_0000;

// ─── 分页 ───────────────────────────────────────────

/// 物理页大小 (linker.ld: ALIGN)
pub const PAGE_SIZE: usize = 1 << 12; // 4KiB

/// 页大小对应的位偏移
pub const PAGE_SIZE_BIT: usize = 12;

// ─── 时钟与调度 ─────────────────────────────────────

/// RISC-V 时钟频率 (Hz)，QEMU virt 机器为 10MHz
pub const CLOCK_FREQ: usize = 10_000_000;

/// 每秒时钟中断次数（任务抢占频率）
pub const TICK_PER_SEC: usize = 100;

// ─── 系统调用号 ─────────────────────────────────────
/// 与 Linux RISC-V ABI 兼容
pub const SYS_WRITE: usize = 64;
pub const SYS_EXIT: usize = 93;
pub const SYS_YIELD: usize = 124;
pub const SYS_GET_TIME: usize = 169;

// ─── 文件描述符 ─────────────────────────────────────

/// 标准输出文件描述符
pub const STD_OUTPUT: usize = 1;

pub const TRAMPOLINE: usize = 0xFFFFFFFFFFFFF000;

pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;
