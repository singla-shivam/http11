use std::boxed::Box;
use std::ptr;

pub(crate) struct Bytes<'a> {
    buf: &'a [u8],
    pos: usize,
    len: usize,
}

impl<'a> Bytes<'a> {
    pub fn new(buf: &'a [u8], len: usize) -> Bytes {
        Bytes { buf, pos: 0, len }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.pos >= self.len()
    }

    pub fn current_pos(&self) -> usize {
        self.pos
    }

    pub fn peek(&self) -> Option<u8> {
        self.at(0)
    }

    pub fn at(&self, index: isize) -> Option<u8> {
        if index < 0 {
            let len = self.len() as isize;
            self.buf.get((len + index) as usize).cloned()
        } else {
            self.buf.get(self.pos + index as usize).cloned()
        }
    }

    pub fn bump(&mut self) -> Option<u8> {
        let byte = self.peek();
        self.advance(1);
        byte
    }

    pub fn buffer(&self) -> &'a [u8] {
        self.buf
    }

    pub fn advance(&mut self, count: isize) -> &Bytes {
        if count < 0 {
            return self;
        }

        let count = count as usize;
        assert!(self.pos + count <= self.len(), "bytes advance overflow");
        self.pos += count;
        self
    }

    /// Copy buffer from index `start` to `end` both inclusive
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

pub(crate) struct ContinuedBytes<'a> {
    bytes1: &'a mut Bytes<'a>,
    bytes2: &'a mut Bytes<'a>,
}

impl<'a> ContinuedBytes<'a> {
    pub fn new(
        bytes1: &'a mut Bytes<'a>,
        bytes2: &'a mut Bytes<'a>,
    ) -> ContinuedBytes<'a> {
        ContinuedBytes { bytes1, bytes2 }
    }
}

impl<'a> Iterator for ContinuedBytes<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        match self.bytes1.next() {
            Some(b) => Some(b),
            None => self.bytes2.next(),
        }
    }
}
