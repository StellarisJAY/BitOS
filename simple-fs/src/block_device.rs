// 块设备接口，定义从块设备读写数据的方法
pub trait BlockDevice: Send + Sync {
    fn read(&self, block_id: u32, data: &mut [u8]);
    fn write(&self, block_id: u32, data: &[u8]);
}

pub struct DummyBlockDevice;
impl BlockDevice for DummyBlockDevice {
    fn read(&self, _: u32, _: &mut [u8]) {}
    fn write(&self, _: u32, _: &[u8]) {}
}
