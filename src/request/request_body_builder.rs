use crate::errors::Error;
use crate::grammar::is_hex_digit;
use crate::helpers::bytes::{Bytes, FragmentedBytes};
use crate::helpers::parser::*;
use std::collections::LinkedList;
use std::{mem, str};

#[derive(Debug)]
pub(crate) struct ChunkedBody {
    chunks: LinkedList<Vec<u8>>,
    /// chunk_size and extensions of last pending chunk
    last_pending_chunk: Option<(usize, Vec<String>)>,
    is_completed: bool,
}

impl ChunkedBody {
    pub fn new() -> ChunkedBody {
        ChunkedBody {
            chunks: LinkedList::new(),
            last_pending_chunk: None,
            is_completed: false,
        }
    }

    pub fn push_back(&mut self, buffer: Vec<u8>) {
        self.chunks.push_back(buffer);
    }

    pub fn set_pending(&mut self, chunk_size: usize, extensions: Vec<String>) {
        self.last_pending_chunk = Some((chunk_size, extensions));
    }

    pub fn get_pending(&mut self) -> Option<(usize, Vec<String>)> {
        mem::take(&mut self.last_pending_chunk)
    }
}

#[derive(Debug)]
pub(crate) enum PartialRequestBody {
    Chunked(ChunkedBody),
    Whole(FragmentedBytes),
}

impl PartialRequestBody {
    pub fn push_buffer(&mut self, buffer: Vec<u8>) {
        match self {
            PartialRequestBody::Chunked(list) => {
                list.push_back(buffer);
            }
            _ => (),
        }
    }
}

#[derive(Debug)]
pub(crate) struct RequestBodyBuilder {
    body_length: usize,
    body: PartialRequestBody,
    last_pending_buffer: Option<Vec<u8>>,
}

impl RequestBodyBuilder {
    pub fn new_chunked() -> RequestBodyBuilder {
        RequestBodyBuilder {
            body_length: 0,
            body: PartialRequestBody::Chunked(ChunkedBody::new()),
            last_pending_buffer: None,
        }
    }

    pub fn new_whole(content_length: usize) -> RequestBodyBuilder {
        RequestBodyBuilder {
            body_length: content_length,
            body: PartialRequestBody::Whole(FragmentedBytes::default()),
            last_pending_buffer: None,
        }
    }

    pub fn set_body_length(&mut self, new_length: usize) -> &Self {
        self.body_length = new_length;
        self
    }

    pub fn is_parsed(&self) -> bool {
        match &self.body {
            PartialRequestBody::Whole(fragments) => {
                return fragments.total_len() > 0;
            }
            PartialRequestBody::Chunked(b) => b.is_completed,
        }
    }

    pub fn parse(&mut self, bytes: &mut FragmentedBytes) {
        if self.is_chunked() {
            self.parse_chunked(bytes);
        } else {
            self.parse_whole(bytes);
        }
    }

    fn is_chunked(&self) -> bool {
        matches!(self.body, PartialRequestBody::Chunked(_))
    }

    fn parse_chunked(
        &mut self,
        bytes: &mut FragmentedBytes,
    ) -> Result<(), Error> {
        loop {
            let body = match &mut self.body {
                PartialRequestBody::Whole(_) => return Ok(()),
                PartialRequestBody::Chunked(b) => b,
            };
            let last_pending = body.get_pending();
            let (chunk_size, extensions) = match last_pending {
                Some(x) => x,
                None => {
                    let first_line = RequestBodyBuilder::get_chunk_data(bytes)?;
                    let (chunk_size, extensions) = match first_line {
                        None => return Ok(()),
                        Some(f) => f,
                    };
                    (chunk_size, extensions)
                }
            };

            // last chunk
            if chunk_size == 0 {
                self.parse_last_chunk(bytes);
                return Ok(());
            }
            let buffer = bytes.copy_buffer_of_len(chunk_size);
            if buffer.is_none() {
                // chunk of size `chunk_size` is not yet received
                body.set_pending(chunk_size, extensions);
                return Ok(());
            }
            bytes.advance_read_pos(chunk_size + 2);
            let buffer = buffer.unwrap();

            self.body.push_buffer(buffer);
        }
    }

    fn parse_last_chunk(&mut self, bytes: &mut FragmentedBytes) {
        let body = match &mut self.body {
            PartialRequestBody::Whole(_) => return,
            PartialRequestBody::Chunked(b) => b,
        };

        body.is_completed = true;
    }

    fn get_chunk_data(
        bytes: &mut FragmentedBytes,
    ) -> Result<Option<(usize, Vec<String>)>, Error> {
        let first_line = look_for_crlf(bytes);
        let first_line = match first_line {
            None => return Ok(None),
            Some(f) => f,
        };

        let first_line_string = String::from_utf8(first_line);
        let first_line_string = match first_line_string {
            Err(f) => {
                return Err(Error::InvalidUtf8String(f.into_bytes()));
            }
            Ok(f) => f,
        };

        let mut first_line_parts: Vec<String> = first_line_string
            .split(";")
            .map(|v| v.trim().to_string())
            .collect();

        // TODO check validity of extensions

        let chunk_size = first_line_parts.remove(0);
        let chunk_size_r = usize::from_str_radix(&chunk_size, 16);
        let chunk_size = match chunk_size_r {
            Err(e) => return Err(Error::ParseIntError(chunk_size)),
            Ok(s) => s,
        };

        Ok(Some((chunk_size, first_line_parts)))
    }

    fn parse_whole(&mut self, bytes: &mut FragmentedBytes) {
        if self.body_length <= bytes.total_len() {
            let bytes = mem::take(bytes);
            self.body = PartialRequestBody::Whole(bytes);
        }
    }
}

#[cfg(test)]
mod tests_request_body_builder {
    use super::*;

    #[test]
    fn test_whole_body() {
        const BODY_LENGTH: usize = 20;
        let mut builder = RequestBodyBuilder::new_whole(BODY_LENGTH);
        let bytes1 = Bytes::new(vec![1, 2, 3], 3);
        let mut bytes = fragmented_bytes![bytes1];

        assert!(!builder.is_parsed());
        assert!(!builder.is_chunked());
        assert_eq!(builder.body_length, BODY_LENGTH);

        builder.parse(&mut bytes);
        assert!(!builder.is_parsed());
        assert!(!builder.is_chunked());
        assert_eq!(builder.body_length, BODY_LENGTH);

        let bytes1 = Bytes::new(vec![4, 5, 6, 7, 8, 9, 10], 7);
        bytes.push_bytes(bytes1);
        builder.parse(&mut bytes);
        assert!(!builder.is_parsed());
        assert!(!builder.is_chunked());
        assert_eq!(builder.body_length, BODY_LENGTH);

        let bytes1 = Bytes::new(vec![14, 15, 16, 17, 18, 19, 20], 7);
        bytes.push_bytes(bytes1);
        builder.parse(&mut bytes);
        assert!(!builder.is_parsed());
        assert!(!builder.is_chunked());
        assert_eq!(builder.body_length, BODY_LENGTH);

        let bytes1 = Bytes::new(vec![21, 22, 23, 24], 4);
        bytes.push_bytes(bytes1);
        builder.parse(&mut bytes);
        assert!(builder.is_parsed());
        assert!(!builder.is_chunked());
        assert_eq!(builder.body_length, BODY_LENGTH);

        assert_match!(builder.body, PartialRequestBody::Whole(_));
        if let PartialRequestBody::Whole(b) = builder.body {
            assert_eq!(b.total_len(), 21);
            assert_eq!(b.read_pos(), 0);
        }
    }

    #[cfg(test)]
    mod get_chunk_data {
        use super::*;

        fn create_fragmented_bytes(bytes: Vec<&[u8]>) -> FragmentedBytes {
            let bytes: Vec<Bytes> =
                bytes.into_iter().map(|v| v.to_vec().into()).collect();
            FragmentedBytes::new(bytes)
        }

        #[test]
        fn test_valid() {
            let mut bytes = create_fragmented_bytes(vec![
                b"1A",
                b";ext1",
                b";ext2=v2",
                b" ; ext3=v3",
                b"\r\n",
            ]);

            let chunk_data = RequestBodyBuilder::get_chunk_data(&mut bytes);
            assert!(chunk_data.is_ok());

            let (chunk_size, ext) = chunk_data.unwrap().unwrap();
            assert_eq!(chunk_size, 26);
            assert_eq!(ext, vec!["ext1", "ext2=v2", "ext3=v3"]);
        }

        #[test]
        fn test_invalid_chunk_size() {
            let mut bytes = create_fragmented_bytes(vec![
                b"0S",
                b";ext1",
                b";ext2=v2",
                b" ; ext3=v3",
                b"\r\n",
            ]);

            let chunk_data = RequestBodyBuilder::get_chunk_data(&mut bytes);
            assert!(chunk_data.is_err());

            let expected_error = Error::ParseIntError("0S".to_string());

            assert_match_error!(chunk_data.err().unwrap(), expected_error);
        }

        #[test]
        fn test_incomplete_chunk() {
            let mut bytes = create_fragmented_bytes(vec![
                b"0S",
                b";ext1",
                b";ext2=v2",
                b" ; ext3=v3",
                b"\r",
            ]);

            let chunk_data = RequestBodyBuilder::get_chunk_data(&mut bytes);
            assert!(chunk_data.is_ok());
            let chunk_data = chunk_data.unwrap();
            assert!(chunk_data.is_none());
        }
    }
}
