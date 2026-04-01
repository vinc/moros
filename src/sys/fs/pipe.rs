use super::{FileIO, IO};

use alloc::sync::Arc;
//use alloc::rc::Rc;
use alloc::vec::Vec;
//use core::borrow::BorrowMut;
//use core::cell::RefCell;
use core::cmp;
use spin::Mutex;

#[derive(Debug, Clone)]
pub struct Pipe {
    //buf: Rc<RefCell<Vec<u8>>>,
    buf: Arc<Mutex<Vec<u8>>>,
}

impl Pipe {
    pub fn new() -> Self {
        Self {
            //buf: Rc::new(RefCell::new(Vec::with_capacity(super::BLOCK_SIZE)))
            buf: Arc::new(Mutex::new(Vec::with_capacity(super::BLOCK_SIZE)))
        }
    }

    pub fn size() -> usize {
        super::BLOCK_SIZE
    }
}

impl FileIO for Pipe {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        let mut pipe = self.buf.lock();
        let n = cmp::min(buf.len(), pipe.len());
        buf[..n].clone_from_slice(&pipe[..n]);
        pipe.drain(..n);
        Ok(n)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        let mut pipe = self.buf.lock();
        pipe.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn close(&mut self) {
    }

    fn poll(&mut self, event: IO) -> bool {
        let pipe = self.buf.lock();
        match event {
            IO::Read => !pipe.is_empty(),
            IO::Write => true,
        }
    }
}
