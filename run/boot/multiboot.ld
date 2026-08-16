KERNEL_OFFSET = 0x100000;

SECTIONS {
    . = KERNEL_OFFSET;

    .multiboot : { *(.multiboot) }
    . = ALIGN(4096);

    .text : {
        *(.text .text.*)
    }
    . = ALIGN(4096);

    .rodata : {
        *(.rodata .rodata.*)
    }
    . = ALIGN(4096);

    .data : {
        *(.data .data.*)
    }
    . = ALIGN(4096);

    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    }
    . = ALIGN(4096);
}
