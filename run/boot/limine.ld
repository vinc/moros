KERNEL_OFFSET = 0xFFFFFFFF80000000;

SECTIONS {
    . = KERNEL_OFFSET;

    .text : { *(.text .text.*) }
    . = ALIGN(4096);

    .rodata : { *(.rodata .rodata.*) }
    . = ALIGN(4096);

    .data : {
        *(.data .data.*)
        KEEP(*(.limine_req_start))
        KEEP(*(.limine_reqs))
        KEEP(*(.limine_req_end))
    }
    . = ALIGN(4096);

    .bss : { *(.bss .bss.*) *(COMMON) }
    . = ALIGN(4096);

    /DISCARD/ : { *(.eh_frame*) *(.note*) *(.comment*) }
}
