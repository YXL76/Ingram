ENTRY(_start)

SECTIONS {
        . = 4096M;
        .text : ALIGN(4K) { * (.text); }
}