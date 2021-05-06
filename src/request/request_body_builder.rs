use crate::helpers::bytes::Bytes;
use std::collections::LinkedList;

pub(crate) enum PartialRequestBody {
    Chunked(LinkedList<Vec<u8>>),
    Whole(LinkedList<Vec<u8>>),
}

pub(crate) struct RequestBodyBuilder {
    body_length: isize,
    body: PartialRequestBody,
    // last_pending_buffer: Option<Bytes>,
}

impl RequestBodyBuilder {
    pub fn new_chunked() -> RequestBodyBuilder {
        RequestBodyBuilder {
            body_length: -1,
            body: PartialRequestBody::Chunked(LinkedList::new()),
            // last_pending_buffer: None,
        }
    }

    pub fn new_whole() -> RequestBodyBuilder {
        RequestBodyBuilder {
            body_length: -1,
            body: PartialRequestBody::Whole(LinkedList::new()),
            // last_pending_buffer: None,
        }
    }

    fn is_chunked(&self) -> bool {
        matches!(self.body, PartialRequestBody::Chunked(_))
    }

    fn parse_chunked(&self, bytes: Bytes) {}

    fn parse_whole(&self, bytes: Bytes) {}

    pub fn set_body_length(&mut self, new_length: isize) -> &Self {
        self.body_length = new_length;
        self
    }

    pub fn is_parsed(&self) -> bool {
        // TODO
        unimplemented!()
    }

    pub fn parse(&self, bytes: Bytes) {
        if self.is_chunked() {
            self.parse_chunked(bytes);
        } else {
            self.parse_whole(bytes);
        }
    }
}
