# Novus

一个用 Rust 编写的简易 RISC-V 64 裸机操作系统。

## 特性

- 目标平台：`riscv64gc-unknown-none-elf`，QEMU `virt` 机型
- 通过 SBI 与底层交互（含 srst 系统复位）
- UART 驱动：ns16550a，基于设备树自动探测
- 块设备驱动：virtio-blk（MMIO 模式）
- 内存管理：物理页帧分配器
- 陷阱处理与定时器中断（Stimecmp）
- 用户态切换与 ecall 系统调用框架

## 目录结构

```
src/
├── arch/         RISC-V 寄存器与 SBI 封装
├── dev/          设备驱动（UART、virtio-blk）
├── mem/          地址抽象、页帧分配
├── entry.asm     启动汇编
├── linker.ld     链接脚本
├── trap.rs       陷阱处理
├── syscall.rs    系统调用
├── io.rs         格式化输出（print!/println!）
├── locks.rs      同步原语
├── marco.rs      宏定义
├── error.rs      错误类型
└── main.rs       内核入口
```

## 构建与运行

依赖：

- Rust nightly（edition 2024）
- `riscv64-elf-objcopy`
- `qemu-system-riscv64`

构建并启动：

```bash
cargo build --release
./start.sh
```

`start.sh` 选项：

| 选项 | 作用 |
| ---- | ---- |
| `-s` | 启用 GDB 调试（`-s -S`）|
| `-m` | 输出 MMU 日志 |
| `-i` | 输出中断日志 |
| `-p` | 挂载 virtio-blk PCI 磁盘（`resources/disk.qcow2`）|
| `-d` | 挂载 virtio-blk MMIO 磁盘 |

示例：

```bash
./start.sh -p          # 带 PCI 块设备启动
./start.sh -s -i       # 调试 + 中断日志
```

## 许可证

GPL-3，详见 [LICENSE](LICENSE)。
