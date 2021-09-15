/** The "start"-symbol from assembly.S. */
ENTRY(start)

OUTPUT_FORMAT("elf64-x86-64")
OUTPUT_ARCH("i386:x86-64")

SECTIONS {

    /* Link Address: 4MiB */
    . = 0x400000;
    . += SIZEOF_HEADERS;

    .text 0x400000 : ALIGN (4096)
    {
        *(.text .text.*)
    } : rx

    .rodata : ALIGN (4096)
    {
      *(.rodata .rodata.*)
    } : r

    .data : ALIGN (4096)
    {
      *(.data .data.*)
      *(COMMON)

      /* put .bss in .data */
      *(.bss .bss.*)
    } : rw

    /* TODO Don't know yet why but if I do this, the second LOAD segment is not anymore in the ELF and everything works */
    /DISCARD/ :
    {
        *(.eh_frame*)
    }

}
