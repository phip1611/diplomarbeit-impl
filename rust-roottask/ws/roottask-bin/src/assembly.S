# This is a barely documented feature of Rust.
# Via the `global_asm!`-macro one can include assembly source files
# in the "native assembly syntax flavor of the compiling machine",
# i.e. GNU Assembler on Linux.

# The ability of Rust to include global assembly is poorly documented.
# It uses GAS (GNU Assembly) syntax

# . intel_syntax noprefix
# is already the default => compiler warning

.code64

.global start
.extern roottask_rust_entry

# always produce x-bit x86 code (even if this would be compiled to an ELF-32 file)
.code64

.section .text

# references in linker script
start:
    # prepare stuff

    # INTEL SYNTAX! Destination is left

    # move HIP pointer to first function argument
    mov     rdi,    rsp
    # UTCB is 1 page above HIP
    lea     rsi,    [rsp - 0x1000]

    # set up stack
    # OFFSET is like "moveabs" in AT&T syntax
    mov  rsp,    OFFSET stack_top
    mov  rbp,    OFFSET stack_top

    jmp roottask_rust_entry


.section .data

# TODO stack alignment?!
.ALIGN 0x1000
stack_bottom:
    # zeroed 16KiB stack
    .fill 0x4000, 1, 0
stack_top:
