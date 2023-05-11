use std::fs::{File, OpenOptions};
use simplefs::block_device::BlockDevice;
use simplefs::layout::BLOCK_SIZE;
use simplefs::simple_fs::SimpleFileSystem;
use simplefs::vfs::{Inode, DirEntry};
use spin::Mutex;
use alloc::sync::Arc;
use std::io::{Seek, SeekFrom, Read, Write};
extern crate simplefs;
extern crate alloc;
struct FileBlockDev (Mutex<File>);

fn main() {
    test_create();
    test_open();
}

fn test_create() {
    let block_dev: Arc<dyn BlockDevice> = Arc::new(FileBlockDev::new("./fs.bin", true));
    let mut fs = SimpleFileSystem::new(Arc::clone(&block_dev), 4096, 1);
    let root_inode = fs.create_root_dir();
    println!("root inode seq: {}", root_inode);
    let (blk_id, _, offset) = fs.get_inode_position(root_inode);
    let fs = Arc::new(Mutex::new(fs));
    let root_inode = Inode::new(blk_id, offset, Arc::clone(&fs), Arc::clone(&block_dev));
    println!("root inode got, creating files");
    root_inode.create("file1", false);
    root_inode.create("file2", false);
    root_inode.create("dir1", true);
    root_inode.create("dir2", true);
    println!("files created, listing files");
    root_inode.ls().unwrap().iter().for_each(|name| {
        println!("{}", name);
    });
    let id = root_inode.find("file1").unwrap();
    let file1 = Inode::from_inode_seq(id, Arc::clone(&fs), Arc::clone(&block_dev));
    file1.write(0, "hello world".as_bytes());
    fs.lock().fsync();
}

fn test_open() {
    let block_dev: Arc<dyn BlockDevice> = Arc::new(FileBlockDev::new("./fs.bin", false));
    let fs = SimpleFileSystem::open(Arc::clone(&block_dev));
    let fs = Arc::new(Mutex::new(fs));
    let root_inode = Inode::from_inode_seq(0, Arc::clone(&fs), Arc::clone(&block_dev));
    root_inode.ls().unwrap().iter().for_each(|name| {
        println!("{}", name);
    });
    let id = root_inode.find("file1").unwrap();
    let file1 = Inode::from_inode_seq(id, Arc::clone(&fs), Arc::clone(&block_dev));
    let mut buf = [0u8; 5];
    file1.read(6, &mut buf);
    println!("file content: {}", core::str::from_utf8(&buf).unwrap());
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


