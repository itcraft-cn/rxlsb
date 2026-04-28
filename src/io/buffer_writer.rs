use bytes::{BytesMut, BufMut, Bytes};

pub struct BufferWriter {
    buffer: BytesMut,
}

impl BufferWriter {
    pub fn new(capacity: usize) -> Self {
        Self { buffer: BytesMut::with_capacity(capacity) }
    }
    
    pub fn write_u8(&mut self, v: u8) {
        self.buffer.put_u8(v);
    }
    
    pub fn freeze(self) -> Bytes {
        self.buffer.freeze()
    }
}