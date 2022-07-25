# A Policy-Free System-Call Layer For The Hedron Microhypervisor

This repository is the practical work of my diplom (= bachelor + master in Germany) thesis project
at [TU Dresden](https://tu-dresden.de) in cooperation with Cyberus Technology GmbH. This repository contains my custom
runtime environment written in Rust as well as modifications to Hedron that enables a policy-free system-call layer.
It forwards foreign system calls, such as Linux system calls, to a user-space component ("OS personality") that
emulates a policy. The system call layer doesn't introduce a policy in the kernel but provides only a mechanism.
Similar to Hedron, my work only focus on x86_64 architecture.

## Task and Thesis

My diplom thesis ("Diplomarbeit") can be found inside the repository (english language):
[[PDF]](./diplom-thesis_unsigned.pdf). My thesis includes the description of my task. It is written in english but the
task and the abstract are also provided in german. Don't wonder: Just scroll a bit further.

## Pointers To Interesting Code
- My modifications to the system call handler at the bottom of `thesis-hedron-fork/src/syscall.cpp`.
- The roottask is defined in `runtime-environment/ws/roottask-bin/`.
  `runtime-environment/ws/roottask-bin/src/main.rs` is a good starting point.
- Handling Foreign Linux System Calls
  - `runtime-environment/ws/libroottask/src/pt_multiplex.rs`
  - `runtime-environment/ws/libroottask/src/services/foreign_syscall/mod.rs`

## Build and Run

### TL;DR
See below for more details.
- `git submodule update --init --recursive`
- `make check`
- `make`
- `make run`

### High Level Overview

This project only builds on UNIX systems with typical GNU tools, such as `make`, `grep`, `bash` etc. The build tries to
require as few packages/modifications to your host system as possible. It won't work on MacOS, because right now I don't
make a special treatment to produce ELF files on other systems
(MacOS default format is Macho-O).

The compiler target for the roottask and other binaries is
[ivybridge](https://en.wikipedia.org/wiki/List_of_Intel_CPU_microarchitectures). Hedron does not run on older
hardware.

The final project size after building it is ~2.2 GB in size, because the Rust build produces lots of
intermediate compiler output. When you run `make`, the build process will use cargo/rustup to automatically install the
relevant toolchain into your system. This is the only side effect to your system, that this project has.

To run the project, QEMU is used. This only works, if KVM is available on your Linux system. KVM is only required for
QEMU and Hedron performs no virtualization tasks in my setup.

The Tar archive that includes all applications that the roottask (i.e. my runtime environment) can start are mostly
all release builds, i.e. optimized. To change that, you can modify the paths in the Makefiles that copy
everything to the `build` directory. For example `../target/../{release => debug}`.

All binaries in directory `static-foreign-apps` can be executed on Linux. I take the unmodified ELF files
and put it into the Tar ball.

(*However, it may be possible to build this on other systems/platforms than Linux with relatively small modifications
to the build system and emulate x86_64 code with QEMU, but this is out of scope.*)

### Steps

#### 1) Checkout And Init git Submodules

```shell
$ git clone https://github.com/phip1611/diplomarbeit-impl.git
$ cd "diplomarbeit-impl"
$ git submodule update --init --recursive
```

If the git submodule init procedure fails, please delete the corresponding git submodule directory and
execute `git submodule update --init --recursive` again. I really have no clue why this fails sometimes.

#### 2) Install Required Packages And Tooling

- Relevant packages for building:
  `$ sudo apt install build-essential make cmake`
- `cargo` and `rustc` via `rustup`:
    - `$ sudo apt install curl`
    - `$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` \
      This command comes from: <https://www.rust-lang.org/tools/install>
    - ⚠ When you install rustup, you have to reload your shell once in order for cargo to be present in $PATH! You can
      also execute `$ source $HOME/.cargo/env` ⚠
- QEMU to run everything: \
  `$ sudo apt install qemu-system-x86` \
  I highly recommend to use QEMU 6.2 or above! Otherwise, the startup is really slow. Older QEMU versions are really
  slow when larger payloads are used as multiboot modules. This patch is only available with 6.2 or above
  (<https://gitlab.com/qemu-project/qemu/-/commit/48972f8cad24eb4462c97ea68003e2dd35be0444>)

#### 3) Build And Run
- `$ make check` \
  If everything is green, you are ready to go to the next step. Otherwise, please fix
  any problems. This should be trivial in most cases.
- `$ make`
- `$ make run` or `$ make run_nogui` \
  **I highly recommend to use QEMU 6.2 or above**, when executing `$ make run[_nogui]`! See notice in "required tooling"
  above. Otherwise, you may see "BOOTING FROM ROM..." for 20+ seconds, until something happens.


You should use `$ make run_nogui` on headless systems, such as when you are connected via SSH to a remote machine. The
regular `make run` opens a GUI window with a VGA buffer for Hedron.

All output from the roottask/the runtime environment gets printed to serial (which QEMU maps to stdout) and also
to `qemu_debugcon.txt`.

### Boot on Real Hardware
Currently, Hedron doesn't boot on UEFI without the closed-source UEFI loader of Cyberus Technology GmbH.
However, you can boot my project on real hardware that supports a legacy boot x86 boot flow (on UEFI systems the
CSM mode should work as well). Type `make && make bootimage` and write `legacy_boot_x86.img` to a USB drive or a CD.

The roottask will print information to the serial device (COM1 port) but not to the VGA framebuffer. Thus, you will
only see output from Hedron on the screen so far. Currently, there is no nice mechanism to enable the roottask to
print to a framebuffer.

### Build Troubleshooting
- git submodule init fails: \
  I have no clue why this fails sometimes. If so, it probably works
  to `rm -rf <libc.musl|thesis-hedron-fork` and init the submodules again.
- parallel make build (with jobs parameter) sometimes fails
    - you should not provide `-j $(numproc)` manually because the Makefile itself already parallelizes
      the sub invocations of Make-based projects
    - the bug with parallel builds happens sometimes because multiple Rust builds may trigger rustup to download
      missing components/toolchains. Rustup can only install stuff on a "first come, first serve"
      base.
    - just run again `$ make`
    - the error will fix itself on the second build most likely
- Hedron fails with Error 0x6 (Invalid Opcode) or QEMU fails with unsupported operation: \
  The compiler target for the roottask and other binaries is
  [ivybridge](https://en.wikipedia.org/wiki/List_of_Intel_CPU_microarchitectures).
  Hedron does not run on older hardware.
