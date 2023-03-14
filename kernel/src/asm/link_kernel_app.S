.section .data
.global _app_names
_app_names:
    .string "init_proc"
    .string "hello_world"
.global _app_addrs
_app_addrs:
    .quad 2
    .quad init_proc_start
    .quad hello_world_start
    .quad hello_world_end
.section .data
.global init_proc_start
.global hello_world_start
.global hello_world_end
init_proc_start:
    .incbin "../user_lib/target/riscv64gc-unknown-none-elf/release/init_proc"
hello_world_start:
    .incbin "../user_lib/target/riscv64gc-unknown-none-elf/release/hello"
hello_world_end: