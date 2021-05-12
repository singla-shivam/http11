use crate::errors::Error;
use crate::grammar::is_hex_digit;
use crate::headers::Headers;
use crate::helpers::bytes::{Bytes, FragmentedBytes};
use crate::helpers::parser::*;
use crate::request::RequestBody;
use std::collections::LinkedList;
use std::{mem, str};

#[derive(Debug)]
pub(crate) struct ChunkedBody {
    chunks: FragmentedBytes,
    /// chunk_size and extensions of last pending chunk
    last_pending_chunk: Option<(usize, Vec<String>)>,
    is_completed: bool,
}

impl ChunkedBody {
    pub fn new() -> ChunkedBody {
        ChunkedBody {
            chunks: FragmentedBytes::default(),
            last_pending_chunk: None,
            is_completed: false,
        }
    }

    pub fn push_back(&mut self, buffer: Vec<u8>) {
        self.chunks.push_bytes(buffer.into());
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

    pub fn parse(
        &mut self,
        bytes: &mut FragmentedBytes,
        message_headers: &Headers,
    ) -> Result<&mut Self, Error> {
        if self.is_chunked() {
            self.parse_chunked(bytes, message_headers)?;
        } else {
            self.parse_whole(bytes)?;
        }

        Ok(self)
    }

    pub fn build(self) -> RequestBody {
        match self.body {
            PartialRequestBody::Chunked(b) => RequestBody::Chunked(b.chunks),
            PartialRequestBody::Whole(b) => RequestBody::Whole(b),
        }
    }

    fn is_chunked(&self) -> bool {
        matches!(self.body, PartialRequestBody::Chunked(_))
    }

    fn parse_chunked(
        &mut self,
        bytes: &mut FragmentedBytes,
        message_headers: &Headers,
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
                self.parse_last_chunk(bytes, message_headers);
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

    fn parse_last_chunk(
        &mut self,
        bytes: &mut FragmentedBytes,
        message_headers: &Headers,
    ) {
        let body = match &mut self.body {
            PartialRequestBody::Whole(_) => return,
            PartialRequestBody::Chunked(b) => b,
        };

        let trailer = message_headers.trailer();
        if trailer.is_none() {
            body.is_completed = true;
            return;
        }

        // TODO parse trailer headers
        // let trailer = trailer.unwrap();
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

    fn parse_whole(
        &mut self,
        bytes: &mut FragmentedBytes,
    ) -> Result<(), Error> {
        if self.body_length <= bytes.total_len() {
            let bytes = mem::take(bytes);
            self.body = PartialRequestBody::Whole(bytes);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests_request_body_builder {
    use super::*;
    use std::convert::TryFrom;

    fn empty_message_headers() -> Headers {
        Headers::try_from("".to_string()).unwrap()
    }

    #[test]
    fn test_whole_body() {
        let headers = empty_message_headers();
        const BODY_LENGTH: usize = 20;
        let mut builder = RequestBodyBuilder::new_whole(BODY_LENGTH);
        let bytes1 = Bytes::new(vec![1, 2, 3], 3);
        let mut bytes = fragmented_bytes![bytes1];

        assert!(!builder.is_parsed());
        assert!(!builder.is_chunked());
        assert_eq!(builder.body_length, BODY_LENGTH);

        builder.parse(&mut bytes, &headers);
        assert!(!builder.is_parsed());
        assert!(!builder.is_chunked());
        assert_eq!(builder.body_length, BODY_LENGTH);

        let bytes1 = Bytes::new(vec![4, 5, 6, 7, 8, 9, 10], 7);
        bytes.push_bytes(bytes1);
        builder.parse(&mut bytes, &headers);
        assert!(!builder.is_parsed());
        assert!(!builder.is_chunked());
        assert_eq!(builder.body_length, BODY_LENGTH);

        let bytes1 = Bytes::new(vec![14, 15, 16, 17, 18, 19, 20], 7);
        bytes.push_bytes(bytes1);
        builder.parse(&mut bytes, &headers);
        assert!(!builder.is_parsed());
        assert!(!builder.is_chunked());
        assert_eq!(builder.body_length, BODY_LENGTH);

        let bytes1 = Bytes::new(vec![21, 22, 23, 24], 4);
        bytes.push_bytes(bytes1);
        builder.parse(&mut bytes, &headers);
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
    #[cfg(test)]
    mod chunked_body {
        use super::*;
        #[test]
        fn test_chunked_in_one_pass() {
            let headers = empty_message_headers();
            let mut builder = RequestBodyBuilder::new_chunked();

            let buffer =
                b"A\r\nabcdefghij\r\nA;ext1=1;ext2=2\r\nklmnopqrst\r\n0\r\n";

            let mut bytes = fragmented_bytes![buffer.to_vec().into()];

            builder.parse(&mut bytes, &headers);
            assert!(builder.is_parsed());

            match builder.body {
                PartialRequestBody::Chunked(b) => {
                    assert!(b.is_completed);
                    assert!(b.last_pending_chunk.is_none());

                    let mut expected_chunks = FragmentedBytes::default();
                    expected_chunks.push_bytes(b"abcdefghij".to_vec().into());
                    expected_chunks.push_bytes(b"klmnopqrst".to_vec().into());
                    assert_eq!(b.chunks, expected_chunks);
                }

                PartialRequestBody::Whole(_) => {
                    panic!("Expected chunked body");
                }
            }
        }

        #[test]
        fn test_chunked_in_multiple_pass() {
            let headers = empty_message_headers();
            let mut builder = RequestBodyBuilder::new_chunked();

            // incomplete CRLF of first line of the chunk
            let buffer = b"A\r";
            let mut bytes = fragmented_bytes![buffer.to_vec().into()];
            builder.parse(&mut bytes, &headers);
            assert!(!builder.is_parsed());

            // total length was 10, 6 is passed
            let buffer = b"\nabcdef";
            bytes.push_bytes(buffer.to_vec().into());
            builder.parse(&mut bytes, &headers);
            assert!(!builder.is_parsed());

            // last CRLF is not yet received
            let buffer = b"ghij";
            bytes.push_bytes(buffer.to_vec().into());
            builder.parse(&mut bytes, &headers);
            assert!(!builder.is_parsed());

            // complete CRLF of first line of the chunk
            let buffer = b"\r\n0b\r\n";
            bytes.push_bytes(buffer.to_vec().into());
            builder.parse(&mut bytes, &headers);
            assert!(!builder.is_parsed());

            // last CRLF also received
            let buffer = b"12345678910\r\n";
            bytes.push_bytes(buffer.to_vec().into());
            builder.parse(&mut bytes, &headers);
            assert!(!builder.is_parsed());

            // first line incomplete of the last chunk
            let buffer = b"0";
            bytes.push_bytes(buffer.to_vec().into());
            builder.parse(&mut bytes, &headers);
            assert!(!builder.is_parsed());

            // first line complete of the last chunk
            let buffer = b"00\r\n";
            bytes.push_bytes(buffer.to_vec().into());
            builder.parse(&mut bytes, &headers);
            assert!(builder.is_parsed());

            match builder.body {
                PartialRequestBody::Chunked(b) => {
                    assert!(b.is_completed);
                    assert!(b.last_pending_chunk.is_none());

                    let mut expected_chunks = FragmentedBytes::default();
                    expected_chunks.push_bytes(b"abcdefghij".to_vec().into());
                    expected_chunks.push_bytes(b"12345678910".to_vec().into());
                    assert_eq!(b.chunks, expected_chunks);
                }

                PartialRequestBody::Whole(_) => {
                    panic!("Expected chunked body");
                }
            }
        }

        // TODO
        // #[test]
        // fn test_trailer_part() {}
    }
}
