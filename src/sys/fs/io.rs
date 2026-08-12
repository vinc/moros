#[derive(Clone, Copy)]
pub enum IO {
    Read,
    Write,
}

pub trait FileIO {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()>;
    fn write(&mut self, buf: &[u8]) -> Result<usize, ()>;
    fn close(&mut self);
    fn poll(&mut self, event: IO) -> bool;
}
