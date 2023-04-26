.section .data
.global _app_names
_app_names:
    .string "shell"
    .string "hello_world"
.global _app_addrs
_app_addrs:
    .quad 2
    .quad shell_start
    .quad hello_world_start
    .quad hello_world_end
.section .data
.global shell_start
.global hello_world_start
.global hello_world_end
shell_start:
    .incbin "../user_lib/target/riscv64gc-unknown-none-elf/release/shell"
hello_world_start:
    .incbin "../user_lib/target/riscv64gc-unknown-none-elf/release/hello"
hello_world_end: