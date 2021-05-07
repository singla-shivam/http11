use crate::errors::Error;
use crate::grammar::is_hex_digit;
use crate::helpers::bytes::Bytes;
use std::collections::LinkedList;
use std::str;

pub(crate) enum PartialRequestBody {
    Chunked(LinkedList<Vec<u8>>),
    Whole(LinkedList<Vec<u8>>),
}

impl PartialRequestBody {
    pub fn push_buffer(&mut self, buffer: Vec<u8>) {
        match self {
            PartialRequestBody::Chunked(list) => {
                list.push_back(buffer);
            }
            PartialRequestBody::Whole(list) => {
                list.push_back(buffer);
            }
        }
    }
}

pub(crate) struct RequestBodyBuilder {
    body_length: isize,
    body: PartialRequestBody,
    last_pending_buffer: Option<Vec<u8>>,
}

impl RequestBodyBuilder {
    pub fn new_chunked() -> RequestBodyBuilder {
        RequestBodyBuilder {
            body_length: -1,
            body: PartialRequestBody::Chunked(LinkedList::new()),
            last_pending_buffer: None,
        }
    }

    pub fn new_whole() -> RequestBodyBuilder {
        RequestBodyBuilder {
            body_length: -1,
            body: PartialRequestBody::Whole(LinkedList::new()),
            last_pending_buffer: None,
        }
    }

    fn is_chunked(&self) -> bool {
        matches!(self.body, PartialRequestBody::Chunked(_))
    }

    fn parse_chunked(&mut self, mut bytes: Bytes) -> Result<(), Error> {
        let mut size = vec![];
        let start = 0;
        let end = -1;

        while let Some(byte) = bytes.next() {
            if is_hex_digit(byte) {
                size.push(byte);
                continue;
            }

            match byte {
                b'\r' => {
                    let next_byte = bytes.next();
                    if next_byte.is_none() {
                        // TODO no \n after \r
                    }

                    let next_byte = next_byte.unwrap();
                    if next_byte != b'\n' {
                        let error_string =
                            format!("Char: {}, in body chunk", next_byte);
                        return Err(Error::InvalidCrlf(error_string));
                    }
                    break;
                }
                _ => {}
            }
        }

        let size = str::from_utf8(&size).unwrap();
        let size = u8::from_str_radix(size, 16).unwrap();
        let chunk_body_start = bytes.current_pos();
        let mut chunk_body_end = 0;

        if size == 0 {
            // TODO handle last chunk
        }

        // TODO duplicate code
        while let Some(byte) = bytes.next() {
            match byte {
                b'\r' => {
                    let next_byte = bytes.next();
                    if next_byte.is_none() {
                        // TODO no \n after \r
                    }

                    let next_byte = next_byte.unwrap();
                    if next_byte != b'\n' {
                        let error_string =
                            format!("Char: {}, in body chunk", next_byte);
                        return Err(Error::InvalidCrlf(error_string));
                    }
                    chunk_body_end = bytes.current_pos() - 3;
                    break;
                }
                _ => {}
            }
        }

        let body_buffer = bytes.copy_buffer(chunk_body_start, chunk_body_end);
        self.body.push_buffer(body_buffer);

        Ok(())
    }

    fn parse_whole(&self, bytes: Bytes) {}

    pub fn set_body_length(&mut self, new_length: isize) -> &Self {
        self.body_length = new_length;
        self
    }

    pub fn is_parsed(&self) -> bool {
        // TODO
        unimplemented!()
    }

    pub fn parse(&mut self, bytes: Bytes) {
        if self.is_chunked() {
            self.parse_chunked(bytes);
        } else {
            self.parse_whole(bytes);
        }
    }
}
