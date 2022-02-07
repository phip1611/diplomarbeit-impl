# GNU assembler with intel syntax by default.

# This is a barely documented feature of Rust.
# Via the `global_asm!`-macro one can include assembly source files
# in the "native assembly syntax flavor of the compiling machine",
# i.e. GNU Assembler on Linux.

# The ability of Rust to include global assembly is poorly documented.

.code64

.global start
.extern roottask_rust_entry ROOTTASK_STACK_TOP_PTR

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
    # symbol ROOTTASK_STACK_TOP is the pointer to the stack top
    # => therefore no "OFFSET" (movabs)
    # => instead it loads the value from the address of ROOTTASK_STACK_TOP_PTR,
    #    where the stack pointer stands
    mov     rsp,    ROOTTASK_STACK_TOP_PTR
    mov     rbp,    ROOTTASK_STACK_TOP_PTR

    jmp     roottask_rust_entry
