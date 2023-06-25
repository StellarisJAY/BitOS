STRIP="riscv64-unknown-elf-strip"
TARGET="./target/riscv64gc-unknown-none-elf/release"
files = hello cat stat ls shell fork_test thread_test help echo mkdir timeshard_test file_test 
build:
	@cargo build --release
	# remove debug info
	@for name in $(files); do \
	       	echo strip debug sections for $$name; \
		$(STRIP) -g $(TARGET)/$$name; \
	done; \
	echo strip debug done
	@rm -r ../kernel/target
