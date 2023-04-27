.section .data
.global _app_names
_app_names:
    .string "shell"
    .string "hello_world"
    .string "fork_test"
.global _app_addrs
_app_addrs:
    .quad 3
    .quad shell_start
    .quad hello_world_start
    .quad fork_test_start
    .quad fork_test_end
.section .data
.global shell_start
.global hello_world_start
.global fork_test_start
.global fork_test_end
shell_start:
    .incbin "../user_lib/target/riscv64gc-unknown-none-elf/release/shell"
hello_world_start:
    .incbin "../user_lib/target/riscv64gc-unknown-none-elf/release/hello"
fork_test_start:
    .incbin "../user_lib/target/riscv64gc-unknown-none-elf/release/fork_test"
fork_test_end: