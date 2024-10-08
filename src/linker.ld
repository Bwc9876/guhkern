/* This is the linker script that defines how our kernel's various bits an pieces come together */

/* You can mostly ignore the stuff here, what's important is us settings BASE_ADDRESS and putting .text.entry */
/* first in the .text section, and .text.trampoline second in the .text section */

/* We're on riscv, so hint that, our entry function is _entry, and our base address is 0x80000000 in memory */
/* this is the address where the kernel will be loaded into memory */
OUTPUT_ARCH(riscv)
ENTRY(_entry)
BASE_ADDRESS = 0x80000000;

SECTIONS
{
    /* The kernel is loaded at 0x80000000, so we need to make sure that the first section is at that address */
    . = BASE_ADDRESS;
    kernel_start = .;

    /* The .text section is where the actual code of the kernel goes, we want this first */
    text_start = .;
    .text : {
        /* The entry point of the kernel, make sure this is the first thing in the .text section */
        *(.text.entry)
        . = ALIGN(4k);

        /* The trampoline code is used to jump from the kernel to user space, make sure this is the second thing in the .text section */
        trampoline_start = .;
        *(.text.trampoline)
        . = ALIGN(4k);
        trampoline_end = .;

        /* The rest of the kernel code */
        *(.text .text.*)
    }
    . = ALIGN(4K);
    text_end = .;

    /* The .rodata section is where read-only data goes, like strings and such, we want this next */
    rodata_start = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }
    . = ALIGN(4K);
    rodata_end = .;

    /* The .data section is where mutable data goes, like global variables and such, we want this next */
    data_start = .;
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }
    . = ALIGN(4K);
    data_end = .;

    /* The .bss section is where uninitialized data goes, like global variables that are zero-initialized, we want this last */
    bss_stack_start = .;
    .bss : {
        *(.bss.stack)
        bss_stack_end = .;

        bss_start = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }
    . = ALIGN(4K);
    bss_end = .;

    /* This is the end of the kernel, we use this marker so our page allocator knows where the end of the kernel is */
    /* and can allocate memory beyond it */
    kernel_end = .;
}