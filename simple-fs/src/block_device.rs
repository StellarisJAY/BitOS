// 块设备接口，定义从块设备读写数据的方法
pub trait BlockDevice: Send + Sync {
    fn read(&self, block_id: usize, data: &mut [u8]);
    fn write(&self, block_id: usize, data: &[u8]);
}
