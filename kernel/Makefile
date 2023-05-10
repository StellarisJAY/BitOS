TOOL = "riscv64-unknown-elf-"
TARGET = "./target/riscv64gc-unknown-none-elf/release/"
ELF = $(TARGET)kernel
BIN = $(TARGET)kernel.bin
CPUS = 1
QEMU = qemu-system-riscv64
QEMU_OPTS = -machine virt \
	    -bios none \
	    -kernel $(TARGET)kernel.bin \
	    -smp $(CPUS) \
	    -nographic 
build:
	@cargo build --release
bin:build
	@$(TOOL)objcopy --strip-all $(ELF) -O binary $(BIN)
objdump:
	@$(TOOL)objdump -d $(TARGET)kernel
qemu:bin
	@$(QEMU) $(QEMU_OPTS)
debug:bin
	@$(QEMU) $(QEMU_OPTS) -s -S
gdb:
	@$(TOOL)gdb -ex 'file $(TARGET)kernel' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'
