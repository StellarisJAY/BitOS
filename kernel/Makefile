TOOL = "riscv64-unknown-elf-"
TARGET = "./target/riscv64gc-unknown-none-elf/release/"
build:
	@cargo build --release
bin:build
	@$(TOOL)objcopy --binary
objdump:
	@$(TOOL)objdump -d $(TARGET)kernel
