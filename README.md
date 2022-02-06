# A Flexible System Call Layer For The Hedron Microhypervisor

This repository is the practical work of my diplom (= bachelor + master in Germany) thesis project
at [TU Dresden](https://tu-dresden.de) in cooperation with Cyberus Technology GmbH.
(Diplom is  thingy at universities). This repository contains my custom runtime environment written in
Rust as well as modifications to Hedron that enables a flexible system call layer. The flexible
system call layer doesn't introduce a policy in the kernel but provides only a mechanism. Similar
to Hedron, my work only focus on Intel x86_64 architecture.

# Overview
TODO

# Build

This project only builds on UNIX systems with typical GNU tools, such as `make`, `grep`, `bash` etc. The
build tries to require as few packages/modifications to your host system as possible. It won't work on
MacOS, because right now I don't make a special treatment to produce ELF files on other systems
(MacOS default format is Macho-O).

### 1) Checkout And Init git Submodules
```shell
$ git clone https://github.com/phip1611/diplomarbeit-impl.git
$ cd "diplomarbeit-impl"
$ git submodule update --init --recursive
```

If the git submodule init procedure fails, please delete the corresponding git submodule directory and
execute `git submodule update --init --recursive` again. I really have no clue why this fails sometimes.

### 2) Install Required Packages And Tooling
- Relevant packages: \
  `$ sudo apt install build-essential make cmake`
- `cargo` and `rustc` via `rustup`:
    - <https://www.rust-lang.org/tools/install>
    - `$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
    - ⚠ When you install rustup, you have to reload your shell once in order for cargo
      to be present in $PATH.
- QEMU to run everything: \
  `$ sudo apt install qemu-system-x86` \
  I highly recommend to use QEMU 6.2 or above! Otherwise, the startup is really slow.
  Older QEMU versions are really slow when larger payloads are used as multiboot modules. This patch is only
  available with 6.2 or above
 (<https://gitlab.com/qemu-project/qemu/-/commit/48972f8cad24eb4462c97ea68003e2dd35be0444>)

### 3) Build And Run
My setup only builds and runs on x86_64 Linux systems, because Hedron only supports x86_64 and
the runtime environment relies on ELF files as compiler/linker output. It also can be started
in Linux VMs, if nested virtualization is available, i.e. the virtualized Linux can use KVM.
(*However, it may be possible to build this on other systems/platforms with
relatively small modifications to the build system and emulate x86_64 code with QEMU,
but this is out of scope.*)

- `$ make -j $(nproc)`
- `$ make run` or `$ make run_nogui`

**I highly recommend to use QEMU 6.2 or above**, when executing `$ make run[_nogui]`! See notice in "required tooling"
above. Otherwise, you may see "BOOTING FROM ROM..." for 20+ seconds, until something happens.

You should use `$ make run_nogui` on headless systems, such as when you are connected via SSH to a remote machine.
The regular `make run` opens a GUI window with a VGA buffer for Hedron.

All output from the roottask/the runtime environment gets printed to serial (which QEMU maps to stdout) and
also to `qemu_debugcon.txt`.

#### Build Troubleshooting
- parallel make build (with jobs parameter) sometimes fails
  - this happens because multiple Rust builds may trigger rustup to download
    missing components/toolchains. Rustup can only install stuff on a "first come, first serve"
    base. However, my Makefile supports a workaround which should enable a stable build
    all the time.
  - just run again `$ make -j $(nproc)` or to be 100% safe `$ make`
  - the error will fix itself on the second build most likely
