.section .data
.global _fs_start
.global _fs_end
_fs_start:
    .incbin "fs.bin"
_fs_end: