.section .data
.global _app_names
_app_names:
    .string "shell"
    .string "hello_world"
    .string "fork_test"
    .string "help"
    .string "timeshard_test"
    .string "thread_test"
.global _app_addrs
_app_addrs:
    .quad 6
    .quad shell_start
    .quad hello_world_start
    .quad fork_test_start
    .quad help_start
    .quad timeshard_test_start
    .quad thread_test_start
    .quad thread_test_end
.section .data
.global shell_start
.global hello_world_start
.global fork_test_start
.global help_start
.global timeshard_test_start
.global thread_test_start
.global thread_test_end
shell_start:
    .incbin "../user_lib/target/riscv64gc-unknown-none-elf/release/shell"
hello_world_start:
    .incbin "../user_lib/target/riscv64gc-unknown-none-elf/release/hello"
fork_test_start:
    .incbin "../user_lib/target/riscv64gc-unknown-none-elf/release/fork_test"
help_start:
    .incbin "../user_lib/target/riscv64gc-unknown-none-elf/release/help"
timeshard_test_start:
    .incbin "../user_lib/target/riscv64gc-unknown-none-elf/release/timeshard_test"
thread_test_start:
    .incbin "../user_lib/target/riscv64gc-unknown-none-elf/release/thread_test"
thread_test_end: