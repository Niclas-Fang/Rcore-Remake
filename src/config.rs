//! 内核配置常量
//!
//! 集中管理所有可配置的系统参数。
//! 汇编文件 (.S) 和链接脚本 (.ld) 无法直接引用这些常量，
//! 如需修改请确保同步更新对应文件中的硬编码值。

pub const KERNEL_BASE: usize = 0x80200000;

pub const KERNEL_MEMORY_SIZE: usize = 128 * 1024 * 1024; // 128M

pub const MAX_NUM_APP: usize = 16;

pub const KERNEL_STACK_SIZE: usize = 4096 * 2; // 8KiB

pub const USER_STACK_SIZE: usize = 4096; // 4KiB

pub const MEMORY_END: usize = 0x88000000;

pub const USER_BASE_VA: usize = 0x1_0000;

pub const KERNEL_HEAP_SIZE: usize = 0x200_0000;

pub const PAGE_SIZE: usize = 1 << 12; // 4KiB

pub const PAGE_SIZE_BIT: usize = 12;

pub const CLOCK_FREQ: usize = 10_000_000;

pub const TICK_PER_SEC: usize = 100;

pub const SYS_WRITE: usize = 64;
pub const SYS_EXIT: usize = 93;
pub const SYS_YIELD: usize = 124;
pub const SYS_GET_TIME: usize = 169;
pub const SYS_FORK: usize = 220;
pub const SYS_EXEC: usize = 221;

pub const STD_OUTPUT: usize = 1;

pub const TRAMPOLINE: usize = 0xFFFFFFFFFFFFF000;

pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;
