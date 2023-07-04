# BitOS
用Rust实现类unix RISC-V内核。基于rCoreV3教程和xv6-riscv项目实现。

## 功能模块
1. **内存管理**：分页内存，进程虚拟地址空间
2. **进程**：进程管理，FIFO调度，fork、waitpid等系统调用
3. **系统调用**：重要的系统调用及用户库封装
4. **文件系统**：基于块设备的简单文件系统，支持多级目录
5. **并发**：内核线程，互斥锁、条件变量等并发数据结构
6. **shell**：shell程序，支持cd、mkdir、ls等基本命令，支持命令行参数传递
7. **应用程序**：echo、stat、cat等基本应用程序

## Build & Run

### 前置要求

- Rust环境：Rust-nightly版本
- riscv64-unknown-elf binutils：readelf、strip、objdump等工具
- Qemu7.0.0：安装qemu-system-riscv64，暂时只测试过7.0.0版本，其余版本是否可运行未知

### Build

可直接运行run.sh开始编译和启动，run.sh会首先编译应用程序，然后创建文件系统镜像，最后编译并启动内核。

手动Build过程按照编译应用程序、构建文件系统、编译运行内核三个步骤进行。

1. 进入user_lib目录，输入**make build**命令构建应用程序。
2. 进入simple_fs_test目录，输入**make run**命令构建文件系统镜像。
3. 进入kernel目录，输入**make qemu**命令编译并启动内核

### 运行

在根目录运行run.sh或者在kernel目录**make qemu**运行内核。

看到下面界面之后表示启动成功，可输入help列出可用的命令。

![](https://images-1257369645.cos.ap-chengdu.myqcloud.com/BitOS/start.PNG)

## TODO

- [ ] IPS 跨进程通信：管道、信号
- [ ] 完成virtio-blk驱动程序，在虚拟块设备上创建文件系统
- [ ] bitscript 脚本语言
- [ ] PCI总线，驱动程序框架
- [ ] 网络驱动，以太网协议、ARP、IP协议
- [ ] TCP/UDP协议栈
- [ ] GUI
