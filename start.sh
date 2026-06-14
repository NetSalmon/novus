#!/usr/bin/bash

riscv64-elf-objcopy -O binary \
    target/riscv64gc-unknown-none-elf/release/risc_code \
    target/riscv64gc-unknown-none-elf/release/os.bin

QEMU_ARGS=(
    -machine virt
    -nographic
    -kernel target/riscv64gc-unknown-none-elf/release/os.bin
)

QEMU_OPTS=()
LOG_FLAGS=()

while getopts "misd" opt; do
  case $opt in
    m)
      LOG_FLAGS+=("mmu")
      ;;
    i)
      LOG_FLAGS+=("int")
      ;;
    s)
      QEMU_OPTS+=("-s" "-S")
      ;;
    d)
      QEMU_OPTS+=(
        -drive "file=./resource/disk.img,format=raw,id=hd0,if=none"
        -device "virtio-blk-device,drive=hd0"
      )
      ;;
    \?)
      echo "无效的选项: -$OPTARG" >&2
      exit 1
      ;;
  esac
done

if [ ${#LOG_FLAGS[@]} -ne 0 ]; then
    IFS=',' LOG_STR="${LOG_FLAGS[*]}"
    QEMU_OPTS+=("-d" "$LOG_STR")
fi

# 6. 最终执行 QEMU 命令
qemu-system-riscv64 "${QEMU_ARGS[@]}" "${QEMU_OPTS[@]}"