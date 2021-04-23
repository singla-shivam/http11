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

    pub fn peek(&self) -> Option<u8> {
        self.buf.get(self.pos).cloned()
    }

    pub fn next(&mut self) -> &Bytes {
        self.advance(1)
    }

    pub fn buffer(&self) -> &'a [u8] {
        self.buf
    }

    pub fn advance(&mut self, count: usize) -> &Bytes {
        assert!(self.pos + count <= self.len(), "bytes advance overflow");
        self.pos += count;
        self
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
