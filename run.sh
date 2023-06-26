clear
cd ./user_lib
echo "Building user space applications"
make -i build
sleep 1s
clear
cd ../simple-fs-test
echo "Building file system image"
make -i run
sleep 1s
clear
cd ../kernel
echo "Building kernel, target at @/kernel/target/riscv64gc-unknown-none-elf/kernel.bin"
make qemu