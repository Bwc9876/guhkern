
qemu:
    cargo build
    rm --force log.txt
    qemu-system-riscv64 -machine virt -bios none -kernel target/riscv64gc-unknown-none-elf/debug/guhkern -m 128M -smp 3 -nographic -global virtio-mmio.force-legacy=false -no-reboot -no-shutdown -D log.txt -d exec

dump-asm:
    cargo rustc --release -- --emit asm -C "llvm-args=-x86-asm-syntax=intel"
    echo "Check target/riscv64gc-unknown-none-elf/release/deps"
    
fmt:
    cargo fmt
    nix fmt
