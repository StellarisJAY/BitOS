TOOL = "riscv64-unknown-elf-"
TARGET = "./target/riscv64gc-unknown-none-elf/release/"
FS_IMG = "./fs.bin"
ELF = $(TARGET)kernel
BIN = $(TARGET)kernel.bin
CPUS = 1
LEGACY_VIRTIO = false
QEMU = qemu-system-riscv64
DUMP_DTB = -machine dumpdtb=riscv64-virt.dtb
QEMU_TRACE_EVENTS = -trace events=./events,file=./trace.log
QEMU_OPTS = -machine virt \
	    -bios none \
	    -kernel $(TARGET)kernel.bin \
	    -smp $(CPUS) \
	    -global virtio-mmio.force-legacy=$(LEGACY_VIRTIO) \
	    -drive file=$(FS_IMG),if=none,format=raw,id=x0 \
	    -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
	    -nographic 
build:
	@cargo build --release
bin:build
	@$(TOOL)objcopy --strip-all $(ELF) -O binary $(BIN)
objdump:
	@$(TOOL)objdump -d $(TARGET)kernel
qemu:bin
	@$(QEMU) $(QEMU_OPTS)
trace:bin
	@$(QEMU) $(QEMU_TRACE_EVENTS) $(QEMU_OPTS)
debug:bin
	@$(QEMU) $(QEMU_OPTS) -s -S
gdb:
	@$(TOOL)gdb -ex 'file $(TARGET)kernel' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'
dtb:bin
	@$(QEMU) $(QEMU_OPTS) $(DUMP_DTB)
