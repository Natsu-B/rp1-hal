ENTRY(__rp1_reset0)
EXTERN(__rp1_reset0 __rp1_reset1)
MEMORY {
  SRAM  (rwx) : ORIGIN = 0x20000000, LENGTH = 48K
  ISRAM (rwx) : ORIGIN = 0x10000000, LENGTH = 8K
  DSRAM (rwx) : ORIGIN = 0x10002000, LENGTH = 8K
  LINK  (rw)  : ORIGIN = 0x2000c000, LENGTH = 0x3f00
}
PHDRS { text PT_LOAD; isram PT_LOAD; data PT_LOAD; shared PT_LOAD; }
SECTIONS {
  .vectors ORIGIN(SRAM) : ALIGN(512) {
    __vectors0 = .; KEEP(*(.vectors.proc0));
    . = ALIGN(512); __vectors1 = .; KEEP(*(.vectors.proc1));
  } > SRAM :text
  .boot 0x20000400 : { KEEP(*(.text.rp1_reset)); } > SRAM :text
  .text : ALIGN(4) { *(.text .text.*); *(.rodata .rodata.*); . = ALIGN(4); } > SRAM :text
  .ARM.exidx : { *(.ARM.exidx*); } > SRAM :text
  .isram : ALIGN(4) {
    __sisram = .; *(.isram .isram.*); . = ALIGN(4); __eisram = .;
  } > ISRAM AT> SRAM :isram
  __siisram = LOADADDR(.isram);
  .data : ALIGN(8) {
    __sdata = .; *(.data .data.*); *(.sdata .sdata.*); *(.got .got.*);
    . = ALIGN(4); __edata = .;
  } > DSRAM AT> SRAM :data
  __sidata = LOADADDR(.data);
  /* Not part of any PT_LOAD: loaders zero p_memsz-p_filesz at p_paddr, which would land in shared SRAM.
     Each core zeroes its own DSRAM copy in startup. */
  .bss (NOLOAD) : ALIGN(8) { __sbss = .; *(.bss .bss.*); *(COMMON); . = ALIGN(8); __ebss = .; } > DSRAM :NONE
  .shared : ALIGN(8) { *(.shared .shared.*); . = ALIGN(8); __image_end = .; } > SRAM :shared
  .link ORIGIN(LINK) (NOLOAD) : { __link_start = .; . += LENGTH(LINK); __link_end = .; } > LINK :NONE
  .note.rp1 0 (INFO) : { KEEP(*(.note.rp1)); } :NONE
  .rp1.dtbo 0 (INFO) : { KEEP(*(.rp1.dtbo)); } :NONE
  /DISCARD/ : { *(.ARM.extab*); *(.eh_frame*); }
}
ASSERT(SIZEOF(.isram) <= 8192, "ISRAM budget exceeded")
ASSERT(__ebss + MAX(__stack_size0, __stack_size1) <= 0x10004000, "DSRAM + stack budget exceeded")
ASSERT(__image_end <= 0x2000c000, "shared SRAM image exceeds 48 KiB")
ASSERT(__link_end == 0x2000ff00, "link page overlaps ABI reservation")
