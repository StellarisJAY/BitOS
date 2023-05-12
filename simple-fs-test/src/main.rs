use std::fs::{File, OpenOptions};
use simplefs::block_device::BlockDevice;
use simplefs::layout::BLOCK_SIZE;
use simplefs::simple_fs::SimpleFileSystem;
use simplefs::vfs::Inode;
use spin::Mutex;
use alloc::sync::Arc;
use std::io::{Seek, SeekFrom, Read, Write};

extern crate simplefs;
extern crate alloc;
struct FileBlockDev (Mutex<File>);

fn main() {
    create_fs();
}

fn create_fs() {
    let paths: Vec<&str> = vec!["../user_lib/target/riscv64gc-unknown-none-elf/release/hello",
    "../user_lib/target/riscv64gc-unknown-none-elf/release/shell",
    "../user_lib/target/riscv64gc-unknown-none-elf/release/fork_test",
    "../user_lib/target/riscv64gc-unknown-none-elf/release/thread_test"];
    let app_names: Vec<&str> = vec!["hello_world", "shell", "fork_test", "thread_test"];

    let block_dev: Arc<dyn BlockDevice> = Arc::new(FileBlockDev::new("./fs.bin", true));
    let mut fs = SimpleFileSystem::new(Arc::clone(&block_dev), 4096, 1);
    let root_inode = fs.create_root_dir();
    println!("root inode seq: {}", root_inode);
    let (blk_id, _, offset) = fs.get_inode_position(root_inode);
    let fs = Arc::new(Mutex::new(fs));
    let root_inode = Inode::new(blk_id, offset, Arc::clone(&fs), Arc::clone(&block_dev));
    println!("root inode got, creating files");
    for (i, name) in app_names.iter().enumerate() {
        let inode = root_inode.create(name, false).unwrap();
        let mut elf = OpenOptions::new().read(true).open(paths.get(i).unwrap()).expect("open elf file error");
        let mut data: Vec<u8> = Vec::new();
        elf.read_to_end(&mut data).expect("read elf file error");
        inode.write(0, data.as_slice());
    }
    fs.lock().fsync();
}

impl FileBlockDev {
    fn new(path: &str, create: bool) -> Self {
        let file = OpenOptions::new().read(true).write(true).create_new(create).open(path).unwrap();
        file.set_len(4096 * 4096).unwrap();
        return Self(Mutex::new(file));
    }
}

impl BlockDevice for FileBlockDev {
    fn read(&self, block_id: u32, data: &mut [u8]) {
        let mut file = self.0.lock();
        file.seek(SeekFrom::Start((block_id * BLOCK_SIZE) as u64)).expect("file seek error");
        file.read(data).expect("file read error");
    }
    fn write(&self, block_id: u32, data: &[u8]) {
        let mut file = self.0.lock();
        file.seek(SeekFrom::Start((block_id * BLOCK_SIZE) as u64)).expect("file seek error");
        file.write(data).expect("file write error");
    }
}