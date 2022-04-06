ENTRY(_start)

SECTIONS {
        . = 4G;
        .text : ALIGN(4K) { * (.text); }
}