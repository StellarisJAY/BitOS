use super::block_device::BlockDevice;
use super::layout::BLOCK_SIZE;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use lazy_static::lazy_static;
use spin::mutex::Mutex;

const BLOCK_CACHE_LIMIT: usize = 128;

// CacheFrame 一个块缓存项
pub struct CacheEntry {
    block_id: u32,                         // 块id
    modified: bool,                        // 是否被修改
    block_data: [u8; BLOCK_SIZE as usize], // 缓存数据
    block_device: Arc<dyn BlockDevice>,    // 块设备接口
}

pub struct BlockCache {
    cache_map: BTreeMap<u32, Arc<Mutex<CacheEntry>>>,
}

lazy_static! {
    pub static ref BLOCK_CACHE: Mutex<BlockCache> = Mutex::new(BlockCache::new());
}

// get_block_cache_entry 获取一个磁盘块的缓存对象，如果缓存中没有则通过block_device接口读取
pub fn get_block_cache_entry(
    block_id: u32,
    block_device: Arc<dyn BlockDevice>,
) -> Option<Arc<Mutex<CacheEntry>>> {
    let mut cache = BLOCK_CACHE.lock();
    let entry = cache.get_block(block_id, block_device);
    drop(cache);
    return entry;
}

impl BlockCache {
    pub fn new() -> Self {
        Self {
            cache_map: BTreeMap::new(),
        }
    }

    pub fn get_block(
        &mut self,
        block_id: u32,
        block_device: Arc<dyn BlockDevice>,
    ) -> Option<Arc<Mutex<CacheEntry>>> {
        if let Some(entry) = self.cache_map.get(&block_id) {
            return Some(Arc::clone(&entry));
        }
        // cache已满，弹出没有被使用的entry
        if self.cache_map.len() == BLOCK_CACHE_LIMIT {
            let result = self
                .cache_map
                .iter()
                .find(|(_, v)| Arc::strong_count(v) == 1)
                .expect("block cache full");
            let id = *(result.0);
            let entry = Arc::clone(result.1);
            let e = entry.lock();
            if e.modified {
                e.block_device.write(id, &e.block_data);
            }
            drop(e);
            self.cache_map.remove(&id);
        }
        // 从块设备读取数据，创建entry并添加到缓存map
        let mut data = [0u8; BLOCK_SIZE as usize];
        block_device.read(block_id, &mut data);
        let entry = Arc::new(Mutex::new(CacheEntry::new(block_id, data, block_device)));
        self.cache_map.insert(block_id, Arc::clone(&entry));
        return Some(entry);
    }
}

impl CacheEntry {
    pub fn new(
        block_id: u32,
        data: [u8; BLOCK_SIZE as usize],
        block_device: Arc<dyn BlockDevice>,
    ) -> Self {
        Self {
            block_id: block_id,
            modified: false,
            block_data: data,
            block_device: block_device,
        }
    }

    pub fn sync(&mut self) {
        if self.modified {
            self.block_device.write(self.block_id, &self.block_data);
            self.modified = false;
        }
    }

    // 从块缓存的offset位置，获取T类型的不可变引用
    pub fn as_ref<'a, T: Sized>(&self, offset: u32) -> &'a T {
        assert!(
            (offset + core::mem::size_of::<T>() as u32) >= BLOCK_SIZE,
            "block offset overflow"
        );
        unsafe {
            let ptr = self.block_data.as_ptr().add(offset as usize) as usize as *const T;
            ptr.as_ref().unwrap()
        }
    }

    // 从块缓存的offset位置，获取T类型的可变引用，将导致块缓存modified
    pub fn as_mut<'a, T: Sized>(&mut self, offset: u32) -> &'a mut T {
        assert!(
            (offset + core::mem::size_of::<T>() as u32) >= BLOCK_SIZE,
            "block offset overflow"
        );
        unsafe {
            self.modified = true;
            let ptr = self.block_data.as_ptr().add(offset as usize) as usize as *mut T;
            ptr.as_mut().unwrap()
        }
    }

    // 从块缓存读取数据并转换成T类型，然后执行F函数从T得到R
    pub fn read<T, V, F>(&self, offset: u32, f: F) -> V
    where
        T: Sized,
        V: Sized,
        F: FnOnce(&T) -> V,
    {
        assert!(
            (offset + core::mem::size_of::<T>() as u32) >= BLOCK_SIZE,
            "block offset overflow"
        );
        unsafe {
            let ptr = self.block_data.as_ptr().add(offset as usize) as usize as *const T;
            return f(ptr.as_ref().unwrap());
        }
    }

    // 从块缓存读取数据并转换成mut T类型，执行函数F处理T并返回
    pub fn modify<T, V, F>(&mut self, offset: u32, f: F) -> V
    where
        T: Sized,
        V: Sized,
        F: FnOnce(&mut T) -> V,
    {
        assert!(
            (offset + core::mem::size_of::<T>() as u32) >= BLOCK_SIZE,
            "block offset overflow"
        );
        unsafe {
            self.modified = true;
            let ptr = self.block_data.as_ptr().add(offset as usize) as usize as *mut T;
            f(ptr.as_mut().unwrap())
        }
    }
}

#[cfg(test)]
mod block_cache_tests {
    use super::*;
    struct BlockDev;

    impl BlockDevice for BlockDev {
        fn read(&self, _: u32, _: &mut [u8]) {}
        fn write(&self, _: u32, _: &[u8]) {}
    }

    #[test]
    fn test_block_entry_read_write() {
        let mut cache = CacheEntry::new(0, [0u8; BLOCK_SIZE as usize], Arc::new(BlockDev {}));
        cache.modify(0, |data: &mut [u8; BLOCK_SIZE as usize]| {
            data.fill(10);
        });
        assert!(cache.modified, "cache should be modified");
        cache.read(0, |data: &[u8; BLOCK_SIZE as usize]| {
            data.iter().for_each(|d: &u8| {
                assert!(*d == 10, "element should be the modified value");
            });
        });
    }
}
