use std::boxed::Box;
use std::ops::{Index, IndexMut};
use std::{mem, ptr};

#[derive(Debug)]
pub(crate) struct Bytes {
    buf: Vec<u8>,
    pos: usize,
    len: usize,
}

impl Bytes {
    pub fn new(buf: Vec<u8>, len: usize) -> Bytes {
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

    pub fn buffer(&self) -> &Vec<u8> {
        &self.buf
    }

    pub fn buffer_mut(&mut self) -> &mut Vec<u8> {
        &mut self.buf
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

    pub fn split_off(&mut self, index: usize) {
        self.buf = self.buf.split_off(index);
    }
}

impl Index<usize> for Bytes {
    type Output = u8;

    fn index(&self, i: usize) -> &Self::Output {
        &self.buffer()[i]
    }
}

impl IndexMut<usize> for Bytes {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.buffer_mut()[i]
    }
}

impl Iterator for Bytes {
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

impl From<Vec<u8>> for Bytes {
    fn from(value: Vec<u8>) -> Self {
        let len = value.len();
        Bytes::new(value, len)
    }
}

impl Default for Bytes {
    fn default() -> Self {
        Bytes::new(vec![], 0)
    }
}

#[macro_export]
macro_rules! fragmented_bytes {
    () => ({
        {
            let temp_vec = vec![];
            FragmentedBytes::new(temp_vec)
        }
    });
    ($($x:expr),+) => (
        {
            let mut temp_vec: Vec<Bytes> = Vec::new();
            $(
                temp_vec.push($x);
            )+
            FragmentedBytes::new(temp_vec)
        }
    );
}

#[derive(Debug)]
pub(crate) struct FragmentedBytes {
    bytes_vec: Vec<Bytes>,
    read_pos: usize,
    total_len: usize,
}

impl FragmentedBytes {
    pub fn new(bytes_vec: Vec<Bytes>) -> FragmentedBytes {
        let mut total_len = 0;

        for bytes in &bytes_vec {
            total_len += bytes.len();
        }

        FragmentedBytes {
            bytes_vec,
            read_pos: 0,
            total_len,
        }
    }

    pub fn push_bytes(&mut self, bytes: Bytes) {
        self.total_len += bytes.len();
        self.bytes_vec.push(bytes);
    }

    pub fn iter(&mut self) -> FragmentedBytesIterator<'_> {
        FragmentedBytesIterator::new(self)
    }

    pub fn read_pos(&self) -> usize {
        self.read_pos
    }

    pub fn set_read_pos(&mut self, pos: usize) {
        self.read_pos = pos;
    }

    pub fn advance_read_pos(&mut self, len: usize) {
        self.set_read_pos(self.read_pos() + len);
    }

    pub fn total_len(&self) -> usize {
        self.total_len
    }

    pub fn copy_buffer_of_len(&mut self, len: usize) -> Option<Vec<u8>> {
        if !self.has_n_bytes(len) {
            return None;
        }

        let end = self.read_pos() + len - 1;
        return Some(self.copy_buffer(end));
    }

    /// Copy buffer from `self.read_pos` to `end` both inclusive
    pub fn copy_buffer(&mut self, end: usize) -> Vec<u8> {
        if end < self.read_pos() {
            return vec![];
        }

        let len = end - self.read_pos() + 1;
        let mut vector: Vec<u8> = Vec::with_capacity(len);
        let mut iter = self.iter();

        for i in 0..len {
            vector.push(iter.next().unwrap());
        }

        return vector;
    }

    pub fn remaining_bytes(self) -> FragmentedBytes {
        let mut bytes_vec = vec![];
        let read_pos = self.read_pos();
        let old_bytes_vec = self.bytes_vec;
        let mut i = 0;

        for mut bytes in old_bytes_vec {
            if read_pos >= i && read_pos < i + bytes.len() {
                let original_len = bytes.len();
                let mut vector = mem::take(bytes.buffer_mut());
                let unread_buffer_start = read_pos - i;
                // truncate the buffer to [already_read..]
                let vector = vector.split_off(unread_buffer_start);
                let len = vector.len();
                let bytes = Bytes::new(vector, bytes.len() - unread_buffer_start);
                bytes_vec.push(bytes);

                i += original_len;
            } else if bytes_vec.len() != 0 {
                bytes_vec.push(bytes);
            } else {
                let bytes = mem::take(&mut bytes);
                i += bytes.len();
                drop(bytes);
            }
        }

        FragmentedBytes::new(bytes_vec)
    }

    fn has_n_bytes(&self, n: usize) -> bool {
        self.read_pos() + n <= self.total_len()
    }
}

impl Default for FragmentedBytes {
    fn default() -> Self {
        FragmentedBytes::new(vec![])
    }
}

#[derive(Debug)]
pub(crate) struct FragmentedBytesIterator<'a> {
    fragmented_bytes: &'a mut FragmentedBytes,
    current_pos: usize,
}

impl<'a> FragmentedBytesIterator<'a> {
    pub fn new(
        fragmented_bytes: &'a mut FragmentedBytes,
    ) -> FragmentedBytesIterator<'a> {
        let current_pos = fragmented_bytes.read_pos();
        FragmentedBytesIterator {
            fragmented_bytes,
            current_pos,
        }
    }

    pub fn current_pos(&self) -> usize {
        self.current_pos
    }

    pub fn peek(&self) -> Option<u8> {
        self.at_current_pos()
    }

    fn at_current_pos(&self) -> Option<u8> {
        let mut c = self.current_pos;
        for bytes in &self.fragmented_bytes.bytes_vec {
            if c < bytes.len() {
                return Some(bytes[c]);
            }

            c -= bytes.len();
        }

        return None;
    }
}

impl<'a> Iterator for FragmentedBytesIterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        let byte = self.at_current_pos();
        if byte.is_some() {
            self.current_pos += 1;
            return byte;
        }

        return None;
    }
}

#[cfg(test)]
mod bytes_test {
    use super::{Bytes, FragmentedBytes};

    fn create_fragmented_bytes() -> FragmentedBytes {
        let bytes1 = Bytes::new(vec![1, 2, 3, 4], 4);
        let bytes2 = Bytes::new(vec![5, 6, 7, 8], 4);
        let bytes3 = Bytes::new(vec![11, 12, 13, 14], 4);
        let bytes4 = Bytes::new(vec![15, 16, 17, 18], 4);

        fragmented_bytes![bytes1, bytes2, bytes3, bytes4]
    }

    fn test_fragmented_byte_content(
        fragmented_bytes: &mut FragmentedBytes,
        expected_bytes: &Vec<u8>,
    ) {
        let actual_bytes: Vec<u8> = fragmented_bytes.iter().collect();
        assert_eq!(&actual_bytes, expected_bytes);
    }

    #[test]
    fn test_create_fragmented_bytes() {
        let expected_vector =
            vec![1, 2, 3, 4, 5, 6, 7, 8, 11, 12, 13, 14, 15, 16, 17, 18];

        let mut bytes = create_fragmented_bytes();

        test_fragmented_byte_content(&mut bytes, &expected_vector);

        let mut iter1 = bytes.iter();
        let mut v1 = expected_vector.iter();

        assert_eq!(iter1.peek().unwrap(), *v1.next().unwrap());
        iter1.next();

        assert_eq!(iter1.peek().unwrap(), *v1.next().unwrap());
        iter1.next();

        for i in iter1 {
            assert_eq!(*v1.next().unwrap(), i);
        }
    }

    #[test]
    fn test_remaining_bytes() {
        let expected_vector =
            vec![1, 2, 3, 4, 5, 6, 7, 8, 11, 12, 13, 14, 15, 16, 17, 18];

        let mut fragmented_bytes = create_fragmented_bytes();
        test_fragmented_byte_content(&mut fragmented_bytes, &expected_vector);

        assert_eq!(fragmented_bytes.copy_buffer(2), vec![1, 2, 3]);
        fragmented_bytes.set_read_pos(3);
        let mut fragmented_bytes = fragmented_bytes.remaining_bytes();
        let expected_vector =
            vec![4, 5, 6, 7, 8, 11, 12, 13, 14, 15, 16, 17, 18];
        test_fragmented_byte_content(&mut fragmented_bytes, &expected_vector);

        assert_eq!(fragmented_bytes.copy_buffer(2), vec![4, 5, 6]);
        fragmented_bytes.set_read_pos(3);
        let mut fragmented_bytes = fragmented_bytes.remaining_bytes();
        let expected_vector = vec![7, 8, 11, 12, 13, 14, 15, 16, 17, 18];
        test_fragmented_byte_content(&mut fragmented_bytes, &expected_vector);

        assert_eq!(fragmented_bytes.copy_buffer(2), vec![7, 8, 11]);
        fragmented_bytes.set_read_pos(3);
        let mut fragmented_bytes = fragmented_bytes.remaining_bytes();
        let expected_vector = vec![12, 13, 14, 15, 16, 17, 18];
        test_fragmented_byte_content(&mut fragmented_bytes, &expected_vector);

        assert_eq!(fragmented_bytes.copy_buffer(0), vec![12]);
        fragmented_bytes.set_read_pos(1);
        let mut fragmented_bytes = fragmented_bytes.remaining_bytes();
        let expected_vector = vec![13, 14, 15, 16, 17, 18];
        test_fragmented_byte_content(&mut fragmented_bytes, &expected_vector);

        assert_eq!(fragmented_bytes.copy_buffer(3), vec![13, 14, 15, 16]);
        fragmented_bytes.set_read_pos(4);
        let mut fragmented_bytes = fragmented_bytes.remaining_bytes();
        let expected_vector = vec![17, 18];
        test_fragmented_byte_content(&mut fragmented_bytes, &expected_vector);
    }
}
