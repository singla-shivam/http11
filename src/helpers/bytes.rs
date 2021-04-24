use std::boxed::Box;
use std::ptr;

pub(crate) struct Bytes<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Bytes<'a> {
    pub fn new(buf: &'a [u8]) -> Bytes {
        Bytes { buf, pos: 0 }
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn current_pos(&self) -> usize {
        self.pos
    }

    pub fn peek(&self) -> Option<u8> {
        self.buf.get(self.pos).cloned()
    }

    pub fn bump(&mut self) -> Option<u8> {
        let byte = self.peek();
        self.advance(1);
        byte
    }

    pub fn buffer(&self) -> &'a [u8] {
        self.buf
    }

    pub fn advance(&mut self, count: usize) -> &Bytes {
        assert!(self.pos + count <= self.len(), "bytes advance overflow");
        self.pos += count;
        self
    }

    pub fn copy_buffer(&mut self, start: usize, end: usize) -> Vec<u8> {
        let size = end - start + 1;
        let mut dest = vec![0; size];

        let pointer = self.buf.as_ptr();

        unsafe {
            ptr::copy(pointer.offset(start as isize), dest.as_mut_ptr(), size);
        }

        dest
    }
}

impl<'a> Iterator for Bytes<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if self.len() > self.pos {
            let byte = self.buf.get(self.pos).cloned().unwrap();
            self.pos += 1;
            Some(byte)
        } else {
            None
        }
    }
}
