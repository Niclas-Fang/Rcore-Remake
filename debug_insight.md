# debug_insight.md — RISC-V 裸机内核调试手册

> 适用对象：本项目（riscv64 + QEMU virt + OpenSBI 上的 `no_std` 内核，Sv39 分页，用户态多任务）。
> 目录：① 调试思路与方法论总纲；② 本次排障**实际用到**的工具与方法；③ 案例复盘（三个 bug 的可复用套路）；④ 将来**大概率会用**的工具与方法；⑤ 本项目速查表。

---

## 一、调试思路与方法论（总纲）

### 1.1 先复现，用"最后一个正常输出"做锚点

死机类 bug 的第一现场就是"输出停在哪一行"。内核是从 `kernel_main` 逐步推进的，**最后一行成功打印就是分界点**：bug 必然在它之后的第一段代码里。本次排障的第一步就是完整跑一遍、把输出存盘，然后从"输出停在 `[walk]` 之后"定位到低地址读取。

### 1.2 把系统按启动阶段分层，逐层缩小

本项目的阶段链条：`boot（SBI 打印）→ 堆/帧分配 → 开 MMU → 设置 stvec → 定时器 → 建任务 → 进用户态 → 系统调用/异常 → 调度 → 关机`。
每层都有自己的典型故障（MMU 层查页表，trap 层查 sscratch/satp，syscall 层查参数翻译）。**先确认哪一层坏了，再在这一层内部找原因**，不要从上到下无差别扫。

### 1.3 硬件日志是"地面真相"，软件打印只是"证人"

内核自己的打印可能**本身就是错的**：它读的是被污染的 trap context、陈旧的映射、被重入的全局变量。当内核打印与 QEMU 硬件日志矛盾时，**以 QEMU 为准**。本次决定性证据就是：内核打印 `Exception(15) sepc=0x10000`，而 `-d int` 日志显示真实异常是 `illegal_instruction @0x10038`——说明内核读到的是错误的上下文，而不是硬件出错。

### 1.4 永远先确认"二进制 == 源码"

症状与源码对不上时，先怀疑这三件事：

- **stale 构建**：`build.rs` 在编译期才嵌入用户程序、`link_app.rs` 由构建生成——文件改了但没重新构建，或构建顺序错了（必须先 build user 再 build kernel），就会出现"源码里有的代码，二进制里没有"。
- **并发编辑**：文件在调试过程中被别人/别的终端改掉（本次就发生了 `memory_set.rs` 的 DEBUG 映射在会话中途被改回去）。
- **工具链默认值**：RVC 压缩指令、链接脚本生效范围（`-T` 会作用到整个 workspace）都可能让"显而易见"的解读出错。
- **调试地址随构建漂移**：每次改动代码后重新构建，内核布局（页表 root_ppn、各帧物理地址、符号地址）全都会变——GDB 里硬编码的地址、手动推算的帧号、`info mem` 里的值，**换了构建就全部失效**（本次 root 从 `0x8224a` 漂到 `0x822a6` 又到 `0x82331`）。对策：每次构建后重新 `nm` 取地址、`satp::read()` 取 root，不要沿用旧值；这也是"反汇编/读页表结论与源码矛盾"的常见原因之一。
核对手段：`git diff` + 反汇编 + 符号表（见 2.4/2.6）。

### 1.5 一次只验证一个假设；打印要能区分假设

每次加打印/改代码前，先写下"如果假设 A 成立，输出应该是什么；如果假设 B 成立，输出应该是什么"。比如本次用 `ctx_sepc`（保存的）与 `csr_sepc`（实时 CSR）对比，一组打印就区分了"保存没发生"和"保存到了错的地方"两种假设。

### 1.6 从症状反推故障模型

异常码 + 现场值 → 故障模型，常见组合（RISC-V Sv39）：

- `load/store page fault (0xd/0xf)`：PTE 缺失或权限不足 → 查页表层级、U 位、映射范围
- `instruction access fault (0x1)` + `epc=TRAMPOLINE`：trap 向量本身取指失败 → **satp 是垃圾值**（上下文被污染）
- `illegal instruction (0x2)`：代码错位、RVC 误读、特权指令（U-mode 执行 `csrr`/`sret`）
- 无限重复同一个异常：trap 向量死循环（stvec 指向无法取指/无法处理的地址）、OpenSBI 未委派异常
- 内核态异常（epc 在 0x8020xxxx）：内核代码 bug 或**用户指针直解引用**

### 1.7 环境会骗你

- `-nographic` 会吞掉 Ctrl-C → 必须用 `timeout` 包裹
- 定时器/竞态导致**同一份二进制每次表现不同**（本次 timer 抢在任务切换的不同位置）→ 多跑几次区分"稳定 bug"和"竞态 bug"
- 日志文件别只放内存/临时目录（有的环境不持久），写到项目目录
- **死循环日志会撑爆磁盘**：一次 `-d int` 的 TRAMPOLINE 死循环产生了 870 万行日志，把 `/tmp` 塞满导致后续命令全部 ENOSPC 失败——`-D` 落盘后**及时清理**（`rm /tmp/qemu_*.log`），或先 `grep -c` 看行数再决定是否保留；`timeout` 也给短一点（5–10s 足够看死循环现场）

---

## 二、本次排障实际使用的工具与方法

### 2.1 QEMU `-d int` 异常/中断日志 —— 本次最核心的工具

把所有异常/中断（含异常类型、epc、tval）记到文件，而不是只给一个黑屏：

```bash
timeout 15 qemu-system-riscv64 -machine virt -nographic -bios default \
  -kernel target/riscv64gc-unknown-none-elf/release/my_os -d int -D /tmp/qemu_int.log
```

日志行格式：`cause:0xf, epc:0x..., tval:0x..., desc=store_page_fault`。
**用法要点**：

- 先看**异常序列**而非单条：`grep riscv_cpu_do_interrupt /tmp/qemu_int.log | awk '{print $4,$6,$8}' | uniq -c` 一眼看出"哪些异常各发生了几次、顺序如何"。
- 用 `-D` 落盘避免刷屏；boot 阶段的 SBI putchar（cause 9）是噪音，过滤：`grep -vE "cause:0000000000000009"`。
- 无限重复的同一行 = 死循环现场（本次的 `fault_fetch @ TRAMPOLINE × 81万次` 直接锁定 satp 被写坏）。
- **找"第一个非噪音异常"并看它前面的上下文**（`grep -n -v ... | head` 拿行号，`sed -n 'N-8,N+2p'` 看前几行）——死循环的"第一现场"往往藏在开头：本次第一个异常是 `store_page_fault tval=0x82290`（memmove 写页号），它之前的 ecall 序列说明一切正常，之后的 TRAMPOLINE 死循环只是**次生灾害**。先修第一个异常，后面的连锁反应常常自动消失。

### 2.2 QEMU `-d in_asm` 指令级追踪 —— 看 CPU 到底执行了什么

```bash
timeout 12 qemu-system-riscv64 ... -d in_asm -D /tmp/qemu_asm.log
grep -n "0x0000000000010000" /tmp/qemu_asm.log   # 找用户程序地址
```

**本次的价值**：怀疑"应用代码是垃圾"时，追踪显示 bad_register 其实正常执行到了 `csrrs sstatus`（0x10038），证明**应用本身没问题**，把怀疑引向内核的 trap 上下文路径。它还顺带揭穿了 RVC 压缩指令（`1141` = `c.addi16sp`，2 字节）——`objdump` 地址跨度是 2 而不是 4 时，那是压缩指令，不是文件损坏。

### 2.3 `timeout` 包裹运行 —— 防挂死

`-nographic` 模式下 QEMU 吞掉 Ctrl-C，一旦死机就只能 kill 进程；`timeout 15 qemu ...` 让每条命令都有自杀时限，配合 `echo $?`（124 = 超时被杀，0 = 正常关机）判断是"死机"还是"正常结束"。

### 2.4 反汇编与符号表：rust-objdump / rust-nm / rust-readobj

把故障地址映射回函数/指令，是连接"硬件现场"和"源码"的桥梁：

```bash
rust-objdump -d --no-show-raw-insn --start-address=0x80204f40 --stop-address=0x80204fb0 target/riscv64gc-unknown-none-elf/release/my_os
rust-nm -n target/riscv64gc-unknown-none-elf/release/my_os | grep -E "ekernel|LOGGER|strampoline"
rust-readobj --sections target/riscv64gc-unknown-none-elf/release/bad_register   # 或 readelf -S
```

**本次的价值**：`epc=0x80204f7e` 反汇编出 `set_logger`；`nm` 确认 `LOGGER.0`、`ekernel`、`HEAP_SPACE` 的地址，验证堆与帧分配区不重叠；`readelf -S` 看到 ELF 的 LOAD 段偏移，解释 `.bin` 为何从文件偏移 0x1000 开始。**注意**：release 下小函数会被内联（如 `map_trampoline`），符号找不到时就反汇编它的调用者 `new_kernel`。

**符号的段类型字母有含义**：`nm` 输出第三列是小写 `r`（只读段 .rodata/.text）或 `b`/`d`（可写 .bss/.data）。本次就是靠 `KERNEL_STACK` 是 `r` 才发现"内核栈被链接进只读段"——Rust 的不可变 `static` + 常量初始化会进 .rodata，MMU 开启后写它必 store fault（修复：改 `static mut`）。看到 `r` 却要写它，第一反应就是段放错了。

**用户程序也要查符号**：`nm target/.../release/hello | grep _start`——本次用它发现 `_start = 0x80200000`（旧链接地址），定位到"用户程序没按新 linker.ld 重编 / 链接参数指向了内核脚本"。检查 `_start`/`sbss` 的地址是否符合预期（0x10000 附近），是"用户侧链路是否健康"的最快体检。

### 2.5 原始字节比对：xxd

怀疑"加载到内存的代码和 bin 不一致"时直接看字节：

```bash
xxd -s 0x1000 -l 64 target/riscv64gc-unknown-none-elf/release/bad_register
xxd target/riscv64gc-unknown-none-elf/release/hello.bin | head -3
```

本次用它将"内存中 0x10000 处的内容"与 ELF 入口指令比对，确认 bin 内容正确（之前误判为损坏，实际是 RVC）。

### 2.6 git diff 与源码/二进制一致性核对

- `git diff` 看未提交改动：本次一开始就看到 4 个文件被改（页表相关调试代码），这是"用户正在改什么"的第一手情报。
- 会话中 `memory_set.rs` 的 DEBUG 映射凭空消失 → 反汇编发现二进制里根本没有那次 `map()` 调用 → 确认**二进制与源码不同步**（stale build / 并发编辑）。**反汇编是核对二进制的最终手段**。

### 2.7 临时插桩打印（CSR 快照 + 页表软件走查）

死机时没法用交互调试器，打印是最快的测量手段。本次几类有效的插桩：

- **CSR 快照**：在关键点打印 `stvec`/`sscratch`/`satp`/`sepc` 的值（如 `[boot] stvec=0x80200000` 揭示了 OpenSBI 留下的 trap 向量，解释了"异常后跳回 `_start` 重跑"）。
- **页表软件走查**：仿照硬件手动解析 Sv39 三级页表（L2→L1→L0 逐级打印 PTE），内核态直接读物理地址即可：

  ```rust
  fn walk_one(va: usize, root: usize) {
      let idx2 = (va >> 30) & 0x1ff; let idx1 = (va >> 21) & 0x1ff; let idx0 = (va >> 12) & 0x1ff;
      // 依次读 (root<<12)|(idx2*8) → ((pte2>>10)<<12)|(idx1*8) → ((pte1>>10)<<12)|(idx0*8)
  }
  ```

  本次靠它确认 `L2[0]` 为空（低地址映射不存在）→ 死机原因从"硬件坏了"变成"映射没建"。
- **trap 处理器的现场打印**：`scause`/`stval`/`sepc`/`sp`/`a7`/`x1` 一次打全；打印 `KERNEL_SPACE.translate(TRAP_CONTEXT)` 确认内核映射指向哪个物理帧。
- 打印后**记得清理**，或用 `LOG` 级别控制，避免长期噪音。

### 2.8 多视角交叉验证 —— 本次破案的关键方法

同一件事从三个独立视角看，两两比对：

1. **内核打印**（`context.sepc` —— 保存的上下文）
2. **实时 CSR**（`riscv::register::sepc::read()` —— 硬件现场）
3. **QEMU 日志**（`-d int` 的 epc —— 硬件事实）
三者一致 → 链路健康；不一致 → 定位到具体环节。本次正是"内核打印 sepc=0x10000 但硬件 epc=0x10038"暴露了**保存的上下文没落在内核读取的帧上**，最终追到 TLB 陈旧项。这是"硬件 vs 软件"矛盾时最有生产力的核对方式，建议长期保留这种打印能力。

### 2.9 GDB batch 脚本模式 + QEMU Monitor —— 本次"读物理内存/看 MMU 视角"的主力

交互式 GDB 在无头/自动化环境里容易卡死（断点不命中、异常信号停在 batch 里），**本次实际用的是 batch 脚本模式**：一条命令完成"连接 → 断点 → 取证据 → 退出"：

```bash
gdb-multiarch -batch \
  -ex "target remote :1234" \
  -ex "b *0x802023c6" \              # 地址先 nm 查（见 2.4）
  -ex "continue" \
  -ex "info reg satp sp sscratch" \
  -ex "monitor xp/2gx 0x8228bff8"    # ← 关键：按物理地址读内存
```

**monitor 命令**（QEMU gdbstub 转发给 QEMU monitor）是本手册里最该提前的部分，本次实际用了两类：

- **`monitor xp/<n>gx <物理地址>`**——按**物理地址**读内存。用它在"软件说页表完美、硬件说翻译失败"的死局里逐级读根页表 → L2 → L1 的 PTE（`root[0x1FF]` → `0x208a2c01` → `L2[0x1FF]` → `0x208a3001` → `L1[0x1FF]` → `0x2008041b`），证明**页表物理内容 100% 正确**，把矛盾收窄到 MMU/TLB 层面。`gdb` 的 `x` 是虚拟读（此时不可靠），**物理读必须 `monitor xp`**。
- **`monitor info mem`**——打印当前 MMU 视角的完整映射表（`fffffffffffff000 → 80201000 (r-xu)`）。这是"硬件翻译器看到的页表"的直接证据，和软件 `translate()` 交叉验证。

**操作要点**：每次 GDB 会话结束 QEMU 会继续跑（detached），下次断点可能不命中——**重连前先 `pkill -9 -f qemu-system` 重启**；`-s -S` 时 QEMU 停在第一条指令等 GDB，batch 里 `continue` 前记得先设断点。

---

## 三、案例复盘：四个 bug 的可复用套路

### 案例 A：低地址读取 fault 后"死机"（无任何输出）

- 现象：输出停在 `[walk]` 之后，QEMU 无退出。
- 套路：`-d int` 看到 `load_page_fault tval=0x10000` → 低地址映射问题 → 软件走查 L2[0] 为空 → **映射从未建立**（stale 构建 / 源码与二进制不同步）。
- 复用点：**页表类问题 = 先问"映射建了吗？"，软件走查页表，别急着怀疑硬件。**

### 案例 B：任务切换后重跑已死任务的代码

- 现象：内核打印的 sepc 永远是初始值 0x10000，且同一个 store fault 反复出现。
- 套路：`ctx_sepc vs csr_sepc` 对比 → 保存值与硬件现场不符 → 追 `_trap_entry` 的保存目标（`sscratch`）与 `trap_handler` 的读取目标（内核 `TRAP_CONTEXT` 映射）是否同一帧 → 发现 `remap_trap_context` 改了 PTE 但 **TLB 没刷**，`__restore` 读的是旧映射（上一个任务的帧）。
- 复用点：**改页表 = 必须 `sfence.vma`**。遇到"读到的还是旧值"先想 TLB，再想缓存。

### 案例 C：sys_write 读用户缓冲区导致 satp 被写坏、TRAMPOLINE 无限取指失败

- 现象：hello 的 write 系统调用后死机，`-d int` 显示 `fault_fetch @ 0xfffffffffffff000` 无限循环。
- 套路：异常序列反推——内核态 `load_page_fault @ 0x80205f9a, tval=0x100e6`（内核读用户地址）→ 嵌套异常时 `sscratch` 还是用户 sp → `_trap_entry` 把现场存到用户栈页（内核页表无映射）→ 再嵌套后从内核栈读到垃圾 `kernel_satp` → `csrw satp` 写坏。
- 复用点：**用户指针不能在内核态直解引用**（用 `current_user_token()` + `PageTable::from_token` 逐页翻译）；**trap 入口对 `sscratch` 的假设只在"从用户态陷入"时成立**，嵌套异常会打破它——内核自身 fault 需要独立的处理路径（或至少 panic 而不是继续陷入）。

### 案例 D："页表物理完美但 MMU 翻译失败"——权限位陷阱（本次最长的一役）

- 现象：`translate()`、软件走查（`manual_walk`）、GDB `monitor xp` 三方都证明 TRAMPOLINE 页表链完美（`root[0x1FF] → L2 → L1 → 0x2008041b`，R|X|U|V），但 `-d mmu` 显示 S 模式取指 `ret 1`（翻译失败），低地址对照映射也失败。
- 弯路（可复用教训）：手工心算 PTE 的 ppn（`0x208a2c01 >> 10`）**连续算错三次**（0x8228A/0x8228B/0x82291），读错了 L2 页表帧，一度误判"页表被破坏"——**PTE 移位必须用工具**（`printf '%x' $((0x208a2c01 >> 10))` 或 python），读到"异常 PTE"时**先怀疑自己地址算错了**，再用 `monitor xp` 从根页表逐级重读一遍。
- 正解：映射**存在且正确**，但叶子带 `U` 位——QEMU 7.0.0 对 **S 模式取指 U=1 页也拒绝**（`SUM` 不只限制 load/store）。去掉 `U` 位后立即翻译成功（`read high va: 0x73` 打印出 trampoline 首字节）。
- 复用点：**"映射存在但翻译失败"时，检查点不止存在性，还有权限位**（U/SUM 对取指的影响、R/W/X 的 leaf 判定）；**权限假设用"改一比特"隔离实验验证**（去掉 U 位重跑，行为立即分化）；这也是"软件走查 ≠ 硬件翻译"的典型场景——软件走查只查存在性，硬件还查权限。

---

## 四、将来大概率会使用的工具与方法

### 4.1 GDB 交互式调试（最值得先掌握）

> 注：**batch 脚本模式本次已经用过**（见 2.9，`gdb-multiarch -batch -ex ...` 一条命令取一个证据）；这里是交互式会话的进阶玩法，适合需要连续单步/观察点的场景。

本项目已配好 `.vscode/launch.json`。命令行用法：

```bash
# 终端 1：QEMU 带 GDB stub 停在第一条指令
timeout 600 qemu-system-riscv64 -machine virt -nographic -bios default \
  -kernel target/riscv64gc-unknown-none-elf/release/my_os -s -S
# 终端 2：
gdb-multiarch target/riscv64gc-unknown-none-elf/release/my_os
(gdb) target remote :1234
(gdb) b *0x80204d36        # 在内核地址下断点（先 nm 查地址）
(gdb) c
(gdb) info registers        # 看 sepc/satp/sscratch/ra
(gdb) x/8gx 0x8224a000      # 读物理内存（页表）
(gdb) si                    # 单步，跨过 trap 入口观察寄存器变化
(gdb) watch *0x8025b438     # 硬件观察点："谁写坏了这个地址"
```

适合：单步跟踪 `_trap_entry`/`__restore` 的寄存器流转、确认 satp/sscratch 切换时刻、用 `watch` 抓"谁在改页表/全局变量"。**注意 release 下地址会随代码变动，先 `nm` 再下断点；GDB 对用户态 VA 无能为力（无调试信息时），但内核态完全可用。**

### 4.2 QEMU `-d mmu`：硬件页表走查日志

```bash
timeout 15 qemu-system-riscv64 ... -d mmu -D /tmp/qemu_mmu.log
```

记录每次访问的硬件翻译结果（逐级 PTE）。当**软件走查与硬件行为矛盾**时（比如软件说映射存在但硬件报 page fault），用它看硬件到底走了哪条链、命中了哪个 PTE。输出量大，配合 `grep` 过滤目标地址。`-d int,mmu` 可同时看异常与翻译。

### 4.3 QEMU Monitor（`-nographic` 下按 `Ctrl-A c`）

进入后：

- `info registers` —— 全寄存器
- `info mem` / `info tlb` —— 内存映射与 TLB 视图（若支持）
- `xp /16gx 0x80200000` —— 按**物理地址**读内存（`x` 是虚拟地址，此时不可靠）
- `q` 退出
适合：死循环时看 PC/satp 现场、读物理内存验证页表内容。也可用 `-monitor telnet:127.0.0.1:5555,server,nowait` 从另一个终端进。

### 4.4 OpenSBI 视角（M-mode 异常）

未委派给 S-mode 的异常（检查 boot 日志的 `MEDELEG` 位）会直接进 OpenSBI。此时：

- 看 OpenSBI 是否打印了 trap 信息（`sbi_trap_error` 等）；
- 确认 `medeleg`/`mideleg`：`0xf0b509` 的位 2=0 意味着 illegal instruction **不委派**，内核永远看不到——需要内核自己避免这类指令，或改委派配置；
- 需要更深层诊断时，可换用带调试输出的 OpenSBI 构建，或用 `-bios none` + 自写 M-mode stub。

### 4.5 内核 panic 回溯

`no_std` + 无 unwinder，panic 只有一行信息。将来代码量大了需要：

- 构建时保留 frame pointer：`[profile.release] rustflags = ["-Cforce-frame-pointers"]`；
- panic handler 里沿 `fp` 链打印 `ra`（或简单地打印 `riscv::register::sepc`/`ra` 附近的值）；
- 配合 `rust-objdump -d` + `nm` 把 ra 地址映射回函数名，得到简易 backtrace。

### 4.6 带调试信息的构建

```toml
[profile.release]
debug = true
```

- `rust-objdump -d -S` 得到源码级反汇编；
- GDB 可 `list`/`bt`（需要 `-g` 调试信息）。
代价：镜像变大、地址变化（可能掩盖/暴露对齐类 bug），调试完记得关掉。

### 4.7 回归测试自动化

死机类 bug 很容易反复。写一个脚本把 5 个应用全跑通作为 golden：

```bash
#!/usr/bin/env bash
set -e
timeout 15 qemu-system-riscv64 -machine virt -nographic -bios default \
  -kernel target/riscv64gc-unknown-none-elf/release/my_os 2>&1 | tee /tmp/run.log
grep -q "hello from user mode before yielded!" /tmp/run.log
grep -q "hello world from user mode after yielded!" /tmp/run.log
grep -qE "^(A|B|C)" /tmp/run.log        # 注意输出行尾可能是 \r\n
grep -q "killed it" /tmp/run.log        # 三个 bad_* 应用各杀一次（count>=3）
```

之后每次改动先跑脚本，5 秒内知道有没有回归。

### 4.8 git bisect

bug 在某个提交引入时：

```bash
git bisect start
git bisect bad HEAD && git bisect good <已知正常提交>
git bisect run ./run_test.sh           # 脚本以退出码 0/非0 表示通过/失败
```

注意：`user/` 是嵌套 git 仓库，bisect 前确认 user 侧也需要回退；build 时间较长，可先 `cargo build -p user_lib --release` 再 bisect 内核。

### 4.9 结构化日志与分级

- 现状：`LOG` 环境变量控制级别（ERROR..TRACE），日志带颜色；
- 将来：给关键事件统一前缀（`[mm]`/`[task]`/`[trap]`/`[syscall]`），方便 `grep` 过滤；trap 入口的现场打印（2.8）建议保留成可开关的 TRACE 级；
- 给每个任务打印 id + 地址（`[task1] sepc=...`），多任务时能分辨"谁在说话"。

### 4.10 时序与确定性控制

同一二进制每次行为不同（本次 timer 抢在不同位置）时：

- `-icount` 让时间挂钩指令数，消除宿主计时抖动，使竞态可复现；
- 调试期把 `TICK_PER_SEC` 调小（如 10Hz）减少抢占干扰；
- 需要复现特定交错时，用 GDB 断点卡住一个任务，单步推进另一个。

### 4.11 规范与源码

- **RISC-V Privileged Spec**：Sv39 页表、异常委派（medeleg/mideleg）、TLB/sfence 语义、`sscratch` 在 trap 中的约定用法——本项目的 `_trap_entry` 设计全部基于它；
- **riscv crate 源码**：`riscv::register::*` 的读写实现、`Sstatus` 位定义；
- **QEMU 源码**：`riscv_cpu_do_interrupt` 的日志格式、`-d` 各选项的输出内容；
- 本项目参考的 **rCore 教程**（ch4 的 trap/task 设计）——当自己的实现与参考实现行为不一致时，diff 一下往往立刻暴露设计偏差。

---

## 五、本项目速查表

### 关键地址（`src/config.rs`）

| 符号 | 值 | 说明 |
| --- | --- | --- |
| `KERNEL_BASE` | `0x80200000` | 内核物理基址（恒等映射） |
| `USER_BASE_VA` | `0x10000` | 用户代码虚拟基址 |
| `TRAMPOLINE` | `0xFFFFFFFFFFFFF000` | 共享 trap 入口页（R/X，无 U 位） |
| `TRAP_CONTEXT` | `TRAMPOLINE - 0x1000` | 每任务 trap 帧（内核态重映射） |
| `MEMORY_END` | `0x88000000` | 物理内存上界 |
| `ekernel` | BSS 末尾 | 帧分配起点（`ekernel..MEMORY_END`） |

### 常用命令速查

```bash
# 构建（顺序不能错：先 user 再 kernel）
cargo build -p user_lib --release
cargo build --release

# 运行/挂死防护
timeout 15 cargo run
timeout 15 qemu-system-riscv64 -machine virt -nographic -bios default \
  -kernel target/riscv64gc-unknown-none-elf/release/my_os

# 异常/指令/页表日志
qemu ... -d int -D qemu_int.log
qemu ... -d in_asm -D qemu_asm.log
qemu ... -d mmu  -D qemu_mmu.log

# 反汇编/符号/段表
rust-objdump -d --no-show-raw-insn --start-address=<addr> --stop-address=<addr2> target/riscv64gc-unknown-none-elf/release/my_os
rust-nm -n target/riscv64gc-unknown-none-elf/release/my_os | grep <sym>
rust-readobj --sections target/riscv64gc-unknown-none-elf/release/<app>

# GDB
gdb-multiarch target/riscv64gc-unknown-none-elf/release/my_os
(gdb) target remote :1234
```

### 常见异常码速查（S-mode）

| cause | 含义 | 首要怀疑 |
| --- | --- | --- |
| 0x1 | 取指访问错误 | satp 被写坏 / trap 向量不可取指 |
| 0x2 | 非法指令 | 代码错位 / 特权指令 / RVC 误读 |
| 0x5 | 定时器中断 | 抢占路径（正常） |
| 0x8 | ecall | 系统调用（正常） |
| 0xd / 0xf | load/store page fault | 页表缺失 / 权限位 / 用户指针直解引用 |
