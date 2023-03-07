TOOL = "riscv64-unknown-elf-"
TARGET = "./target/riscv64gc-unknown-none-elf/release/"
CPUS = 4
QEMU_OPTS = -machine virt -bios none -kernel $(TARGET)kernel.bin -smp $(CPUS) -nographic 
build:
	@cargo build --release
bin:build
	@$(TOOL)objcopy --strip-all $(TARGET)kernel -O binary $(TARGET)kernel.bin
objdump:
	@$(TOOL)objdump -d $(TARGET)kernel
qemu:bin
	@qemu-system-riscv64 $(QEMU_OPTS)
debug:bin
	@qemu-system-riscv64 $(QEMU_OPTS) -s -S
gdb:
	@$(TOOL)gdb -ex 'file $(TARGET)kernel' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'
