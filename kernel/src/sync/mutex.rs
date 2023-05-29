pub trait Mutex: Sync + Send{
    fn lock(&self);
    fn unlock(&self);
}

pub struct SpinMutex {
    
}

pub struct BlockingMutex {
    
}

impl SpinMutex {
    pub fn new() -> Self {
        Self{}
    }
}

impl BlockingMutex {
    pub fn new() -> Self {
        Self{}
    }
}

impl Mutex for SpinMutex {
    fn lock(&self) {
    }
    fn unlock(&self) {
    }
}

impl Mutex for BlockingMutex {
    fn lock(&self) {
        
    }
    fn unlock(&self) {
        
    }
}