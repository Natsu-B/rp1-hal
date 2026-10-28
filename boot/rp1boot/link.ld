/* The VPU firmware runs the raw (header-less) `kernel=` image at 0x80000. Its log says
   "Loading 'rp1boot.img' to 0x00000000 offset 0x200000", but a build linked at 0x200000 ran
   0x180000 below its link address (HW 2026-10-29, hwrun5 06-boot5); the PoC also links here. */
ENTRY(_start)
SECTIONS
{
    . = 0x80000;
    __image_start = .;
    .text : { KEEP(*(.text.boot)) *(.text .text.*) }
    .rodata : ALIGN(16) { *(.rodata .rodata.*) }
    .data : ALIGN(16) { *(.data .data.*) *(.got .got.*) }
    .bss (NOLOAD) : ALIGN(4096) {
        __bss_start = .;
        *(.bss .bss.*) *(COMMON)
        . = ALIGN(16);
        __bss_end = .;
    }
    .stack (NOLOAD) : ALIGN(4096) { . += 0x100000; __stack_top = .; }
    /* Bump heap for the DTB trees; not zeroed. Must stay below the kernel at 0x06000000. */
    .heap (NOLOAD) : ALIGN(4096) { __heap_start = .; . += 0x3000000; __heap_end = .; }
    /DISCARD/ : { *(.comment) *(.eh_frame*) *(.note.gnu*) }
}
ASSERT(__heap_end <= 0x06000000, "rp1boot heap overlaps the kernel placement")
