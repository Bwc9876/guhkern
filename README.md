# ya-xv6-riscv-rust

Yet another xv6 riscv implementation in Rust. This is a learning project to understand how Kernels work at a low-level.
I'm re-implementing the [xv6-riscv kernel](https://github.com/mit-pdos/xv6-riscv/tree/riscv) in Rust
in order to understand how it works.

## Follow Along

I'm trying to write this with as many comments as possible to help me understand what's going on.
If you want to follow along, I'd recommend starting in [main.rs](src/main.rs) with the comment that says `START HERE`.

### Recommended Tools

I'd recommend you use [VSCode](https://code.visualstudio.com/) with a few extensions to make reading this easy:

- [Rust Analyzer](https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer) for Rust syntax highlighting and autocompletion
- [RISC-V Support](https://marketplace.visualstudio.com/items?itemName=zhwu95.riscv) for RISC-V assembly syntax highlighting
- [LinkerScript](https://marketplace.visualstudio.com/items?itemName=ZixuanWang.linkerscript) for GNU linker script syntax highlighting

## Pre-requisites

- rust
- just
- qemu-system-riscv64

If you have `nix` installed, you can do `nix develop` to get all these dependencies, you also won't
need to worry about the env vars or target listed below as they are set in the `flake.nix` file.

## Building

To build the kernel, you'll need Rust and the `riscv64gc-unknown-none-elf` target installed.
Also you'll need to set these env vars:

```env
RUSTFLAGS = "-C link-arg=-Tsrc/linker.ld"
CARGO_BUILD_TARGET = "riscv64gc-unknown-none-elf
```

The first is so that Rust knows what linker script to use, and the second is so that Cargo knows what target to build for
without us having to specify it every time.

Then, just run `cargo build`!

## Running

The kernel is built for qemu-system-riscv64, so you'll need to have that installed and setup, once you do, run

```sh
just qemu
```

This will run the kernel in qemu and you should see the output of the kernel in the terminal.

### Killing

As of the moment, the only way to kill the qemu process is to run `pkill -15 .qemu-system-r`. This
will send the `SIGTERM` signal to the qemu process and it will exit.

### Logs

After running you can view the `exec` logs in `log.txt` this will be quite a big file and it will have every
core's logs. I'd recommend doing `| grep "Trace N:"` replacing N with the core number if you want more filtered output.
You can also filter by the name of the function as well using grep.

### Assembly

If you're curious what the assembly for the kernel looks like, you can run `just dump-asm` and it will dump the assembly
to `target/riscv64gc-unknown-none-elf/release/deps/`. The `.s` file in this folder will contain the assembly of the
kernel in Intel syntax.

I'd recommend copying the contents of this file and pasting it into VSCode with the [RISC-V support](https://marketplace.visualstudio.com/items?itemName=zhwu95.riscv) VSCode extension installed as it'll give you syntax highlighting.

## Inspiration and Resources

This project is based on the [xv6-riscv kernel](https://github.com/mit-pdos/xv6-riscv/tree/riscv) and the [xv6 book](https://pdos.csail.mit.edu/6.828/2019/xv6/book-riscv-rev0.pdf).

In addition, the video series [Source Dive](https://www.youtube.com/playlist?list=PLP29wDx6QmW4Mw8mgvP87Zk33LRcKA9bl) by Low Byte Productions has been amazing at explaining how the xv6 kernel / OS works.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
