use crate::errors::Error;
use crate::grammar::is_token_char;
use crate::headers::Headers;
use crate::helpers::bytes::Bytes;
use crate::request::{Request, RequestBodyBuilder, RequestUri};
use std::collections::LinkedList;
use std::convert::TryFrom;
use std::mem;
use std::vec::Vec;

#[derive(Debug)]
pub enum HttpMethods {
    GET,
    OPTIONS,
    HEAD,
    POST,
    PUT,
    DELETE,
    TRACE,
    CONNECT,
    Extension(String),
}

impl HttpMethods {
    pub fn from_vector(v: Vec<u8>) -> HttpMethods {
        let method_name = String::from_utf8(v).unwrap();
        let method_name = method_name.to_lowercase();

        match &method_name[..] {
            "get" => HttpMethods::GET,
            "options" => HttpMethods::OPTIONS,
            "head" => HttpMethods::HEAD,
            "post" => HttpMethods::POST,
            "put" => HttpMethods::PUT,
            "delete" => HttpMethods::DELETE,
            "trace" => HttpMethods::TRACE,
            "connect" => HttpMethods::CONNECT,
            _ => HttpMethods::Extension(method_name),
        }
    }
}

#[derive(Debug)]
pub enum HttpVersion {
    Http11,
}

impl HttpVersion {
    pub fn from_vector(v: Vec<u8>) -> Result<HttpVersion, Error> {
        let version = String::from_utf8(v).unwrap().to_lowercase();

        match &version[..] {
            "http/1.1" => Ok(HttpVersion::Http11),
            _ => Err(Error::InvalidHttpVersion),
        }
    }
}

#[derive(PartialEq, Debug)]
pub(crate) enum PartialRequest {
    Method(Vec<u8>),
    RequestUri(Vec<u8>),
    HttpVersion(Vec<u8>),
    Headers(Vec<u8>),
    Token(Vec<u8>),
    Complete(Vec<u8>),
    Uninit,
}

pub struct RequestBuilder {
    method: Option<HttpMethods>,
    uri: Option<RequestUri>,
    http_version: Option<HttpVersion>,
    headers: Option<Headers>,
    body: Option<RequestBodyBuilder>,
    partial: PartialRequest,
}

impl PartialRequest {
    fn is_complete(&self) -> bool {
        matches!(self, PartialRequest::Complete(_))
    }

    fn is_uninit(&self) -> bool {
        matches!(self, PartialRequest::Uninit)
    }

    fn has_partial(&self) -> bool {
        !(self.is_complete() || self.is_uninit())
    }

    fn is_same_as(&self, other: &PartialRequest) -> bool {
        use PartialRequest::*;

        match (self, other) {
            (Method(_), Method(_))
            | (RequestUri(_), RequestUri(_))
            | (HttpVersion(_), HttpVersion(_))
            | (Token(_), Token(_)) => true,
            _ => false,
        }
    }

    fn append(&mut self, other: PartialRequest) -> PartialRequest {
        use PartialRequest::*;

        if self.is_uninit() {
            return other;
        }

        let p = self.take();

        match (p, other) {
            (HttpVersion(mut v1), HttpVersion(ref mut v)) => {
                v1.extend_from_slice(v);
                return HttpVersion(v1);
            }

            (RequestUri(mut v1), HttpVersion(ref mut v)) => {
                v1.extend_from_slice(v);
                return RequestUri(v1);
            }

            (Method(mut v1), Method(ref mut v)) => {
                v1.extend_from_slice(v);
                return Method(v1);
            }

            (Headers(mut v1), Headers(ref mut v)) => {
                v1.extend_from_slice(v);
                return Headers(v1);
            }

            (_, o) => return o,
        }
    }

    fn take(&mut self) -> PartialRequest {
        mem::take(self)
    }

    fn take_vec(&mut self) -> Vec<u8> {
        use PartialRequest::*;

        let partial_request = self.take();
        match partial_request {
            Method(v) => v,
            RequestUri(v) => v,
            HttpVersion(v) => v,
            Token(v) => v,
            Complete(v) => v,
            Headers(v) => v,
            _ => vec![],
        }
    }
}

impl Default for PartialRequest {
    fn default() -> Self {
        PartialRequest::Uninit
    }
}

impl RequestBuilder {
    pub fn new() -> RequestBuilder {
        RequestBuilder {
            method: None,
            uri: None,
            http_version: None,
            headers: None,
            body: None,
            partial: PartialRequest::Uninit,
        }
    }

    fn is_parsed(&self) -> bool {
        self.method.is_some()
            && self.uri.is_some()
            && self.http_version.is_some()
            && self.headers.is_some()
            && !self.partial.has_partial()
    }

    fn add_method(&mut self, v: Vec<u8>) -> &RequestBuilder {
        self.partial = self.partial.append(PartialRequest::Method(v));

        let buffer = self.partial.take_vec();
        let method = HttpMethods::from_vector(buffer);
        self.method = Some(method);

        self
    }

    fn add_request_uri(&mut self, v: Vec<u8>) -> &RequestBuilder {
        self.partial = self.partial.append(PartialRequest::RequestUri(v));

        let buffer = self.partial.take_vec();
        let uri = RequestUri::try_from(buffer).expect("invalid uri");
        self.uri = Some(uri);

        self
    }

    fn add_http_version(&mut self, v: Vec<u8>) -> &RequestBuilder {
        self.partial = self.partial.append(PartialRequest::HttpVersion(v));

        let buffer = self.partial.take_vec();
        let version =
            HttpVersion::from_vector(buffer).expect("invalid http version");
        self.http_version = Some(version);

        self
    }

    fn add_headers(&mut self, v: Vec<u8>) -> &RequestBuilder {
        self.partial = self.partial.append(PartialRequest::Headers(v));

        let buffer = self.partial.take_vec();
        let headers = Headers::try_from(buffer).expect("invalid http version");
        self.headers = Some(headers);

        self
    }

    fn add_partial(&mut self, new_partial: PartialRequest) -> &RequestBuilder {
        self.partial = self.partial.append(new_partial);
        self
    }

    fn parse_method(&mut self, bytes: &mut Bytes) -> &mut RequestBuilder {
        if self.method.is_some() {
            return self;
        }

        let method = parse_token(bytes).unwrap();

        match method {
            PartialRequest::Token(v) => {
                self.add_partial(PartialRequest::Method(v));
            }

            PartialRequest::Complete(v) => {
                self.add_method(v);
            }

            _ => (),
        }

        self
    }

    fn parse_request_uri(&mut self, bytes: &mut Bytes) -> &mut RequestBuilder {
        if self.uri.is_some() {
            return self;
        }

        if let PartialRequest::RequestUri(_) = self.partial {
        } else if self.partial.is_uninit() {
        } else {
            return self;
        }

        let request_uri = parse_request_uri(bytes).unwrap();

        match request_uri {
            PartialRequest::RequestUri(v) => {
                self.add_partial(PartialRequest::RequestUri(v));
            }

            PartialRequest::Complete(v) => {
                self.add_request_uri(v);
            }

            _ => (),
        }

        self
    }

    fn parse_http_version(&mut self, bytes: &mut Bytes) -> &mut RequestBuilder {
        if self.http_version.is_some() {
            return self;
        }

        if let PartialRequest::HttpVersion(_) = self.partial {
        } else if self.partial.is_uninit() {
        } else {
            return self;
        }

        let http_version =
            parse_http_version(bytes).expect("Invalid http version");

        match http_version {
            PartialRequest::HttpVersion(v) => {
                self.add_partial(PartialRequest::HttpVersion(v));
            }

            PartialRequest::Complete(v) => {
                self.add_http_version(v);
            }

            _ => (),
        }

        self
    }

    fn parse_headers(&mut self, bytes: &mut Bytes) -> &mut RequestBuilder {
        if self.headers.is_some() {
            return self;
        }

        if let PartialRequest::Headers(_) = self.partial {
        } else if self.partial.is_uninit() {
        } else {
            return self;
        }

        let mut previous_buffer = &vec![];
        if let PartialRequest::Headers(v) = &self.partial {
            previous_buffer = v;
        }

        let mut previous_buffer =
            Bytes::new(previous_buffer, previous_buffer.len());
        let headers = parse_headers(bytes, &mut previous_buffer)
            .expect("Invalid headers");

        match headers {
            PartialRequest::Headers(v) => {
                self.add_partial(PartialRequest::Headers(v));
            }

            PartialRequest::Complete(v) => {
                self.add_headers(v);
            }

            _ => (),
        }

        self
    }

    fn are_headers_valid(&self) -> Result<bool, Error> {
        let headers = self.headers.as_ref().unwrap();

        let transfer_encoding = headers.transfer_encoding();
        if let Some(transfer_encoding) = transfer_encoding {
            if !transfer_encoding.is_chunked() {
                // TODO it is an error, send 400
                return Err(Error::NoChunkedCoding);
            }

            return Ok(true);
        }

        let content_length = headers.content_length();
        return Ok(true);
    }

    fn are_headers_parsed(&self) -> bool {
        self.headers.is_some()
    }

    fn can_have_body(&self) -> bool {
        let headers = self.headers.as_ref().unwrap();

        let transfer_encoding = headers.transfer_encoding();

        if transfer_encoding.is_some() {
            return true;
        }

        let content_length = headers.content_length();
        return content_length.is_some();
    }

    pub(crate) fn parse_body(&mut self, bytes: Bytes) -> &mut RequestBuilder {
        if !self.are_headers_parsed() || !self.can_have_body() {
            return self;
        }

        self.body.as_ref().unwrap().parse(bytes);

        self
    }

    pub(crate) fn parse(
        &mut self,
        buf: &[u8],
        length: usize,
    ) -> Result<&RequestBuilder, Error> {
        let mut bytes = Bytes::new(buf, length);

        if self.method.is_none() {
            skip_initial_crlf(&mut bytes);
        }

        self.parse_method(&mut bytes)
            .parse_request_uri(&mut bytes)
            .parse_http_version(&mut bytes)
            .parse_headers(&mut bytes);

        if self.are_headers_parsed() {
            let headers_valid = self.are_headers_valid();
            if headers_valid.is_err() {
                return Err(headers_valid.err().unwrap());
            }

            if self.body.is_none() {
                let headers = self.headers.as_ref().unwrap();
                let transfer_encoding = headers.transfer_encoding();
                let is_chunked = transfer_encoding.is_some()
                    && transfer_encoding.unwrap().is_chunked();
                if is_chunked {
                    self.body = Some(RequestBodyBuilder::new_chunked());
                } else {
                    self.body = Some(RequestBodyBuilder::new_whole());
                }
            }

            self.parse_body(bytes);
        }

        Ok(self)
    }

    pub(crate) fn can_parse_more(&self) -> bool {
        // headers are not parsed yet, so we can parse more
        if self.headers.is_none() {
            return true;
        }

        if self.can_have_body() {
            let body = self.body.as_ref();
            return match body {
                None => true,
                Some(b) => !b.is_parsed(),
            };
        }

        return false;
    }

    pub(crate) fn build(self) -> Result<Request, Error> {
        if !self.is_parsed() {
            return Err(Error::RequestNotParsed);
        }

        let RequestBuilder {
            method,
            uri,
            http_version,
            headers,
            body,
            partial: _,
        } = self;

        let body = LinkedList::new();

        Ok(Request::new(
            method.unwrap(),
            uri.unwrap(),
            http_version.unwrap(),
            headers.unwrap(),
            body,
        ))
    }
}

fn skip_initial_crlf(bytes: &mut Bytes) {
    loop {
        let byte = bytes.peek();

        match byte {
            Some(b'\r') => {
                bytes.bump();
                let next_byte = bytes.bump().unwrap();
                assert!(
                    next_byte == b'\n',
                    "expected \\n but found {}",
                    next_byte
                );
            }
            Some(b'\n') => {
                bytes.bump();
            }
            Some(_) => {
                return;
            }
            None => return (),
        }
    }
}

fn get_token_buffer<F1, F2, F3>(
    bytes: &mut Bytes,
    on_empty: F1,
    on_found: F2,
    verify_character: F3,
    end_token: u8,
) -> Result<PartialRequest, Error>
where
    F1: FnOnce() -> PartialRequest,
    F2: FnOnce(Vec<u8>) -> PartialRequest,
    F3: FnOnce(u8) -> Result<(), Error>,
{
    if bytes.is_empty() {
        return Ok(on_empty());
    }

    let start = bytes.current_pos();
    loop {
        let byte = bytes.next();

        if byte == Some(end_token) || byte == None {
            let end = bytes.current_pos() - 1;
            if byte == Some(b'\r') {
                bytes.next();
            }

            if byte == None {
                let vec = bytes.copy_buffer(start, end);
                return Ok(on_found(vec));
            }

            // remove space
            let vec = bytes.copy_buffer(start, end - 1);

            return Ok(PartialRequest::Complete(vec));
        }
    }
}

fn parse_token(bytes: &mut Bytes) -> Result<PartialRequest, Error> {
    let on_empty = || PartialRequest::Token(vec![]);
    let on_found = |vec: Vec<u8>| PartialRequest::Token(vec);

    let verify_character = |byte: u8| {
        if !is_token_char(byte) {
            return Err(Error::Token);
        }

        Ok(())
    };

    get_token_buffer(bytes, on_empty, on_found, verify_character, b' ')
}

fn parse_http_version(bytes: &mut Bytes) -> Result<PartialRequest, Error> {
    let on_empty = || PartialRequest::HttpVersion(vec![]);
    let on_found = |vec: Vec<u8>| PartialRequest::HttpVersion(vec);
    let verify_character = |_: u8| Ok(());

    get_token_buffer(bytes, on_empty, on_found, verify_character, b'\r')
}

fn parse_request_uri(bytes: &mut Bytes) -> Result<PartialRequest, Error> {
    let on_empty = || PartialRequest::RequestUri(vec![]);
    let on_found = |vec: Vec<u8>| PartialRequest::RequestUri(vec);
    let verify_character = |_: u8| Ok(());

    get_token_buffer(bytes, on_empty, on_found, verify_character, b' ')
}

fn parse_headers(
    bytes: &mut Bytes,
    previous_bytes: &mut Bytes,
) -> Result<PartialRequest, Error> {
    if bytes.is_empty() {
        return Ok(PartialRequest::Headers(vec![]));
    }

    let start = bytes.current_pos();
    // point to last third byte
    previous_bytes.advance(previous_bytes.len() as isize - 3);

    let return_partial = |bytes: &mut Bytes, start, end| {
        let vec = bytes.copy_buffer(start, end);
        return Ok(PartialRequest::Headers(vec));
    };

    let mut next_byte = |bytes: &mut Bytes| {
        if previous_bytes.is_empty() {
            return bytes.next();
        }

        return previous_bytes.next();
    };

    let mut outstanding_crlf_buffer = [0; 4];
    loop {
        let byte = next_byte(bytes);

        match byte {
            Some(b'\r') => {
                let b = next_byte(bytes);
                if b.is_none() {
                    let end = bytes.current_pos() - 1;
                    return return_partial(bytes, start, end);
                }

                let b = b.unwrap();
                assert!(b == b'\n', "expected \\n but found {}", b);
                push_to_buffer(&mut outstanding_crlf_buffer, b'\r');
                push_to_buffer(&mut outstanding_crlf_buffer, b'\n');

                let is_double_crlf =
                    &outstanding_crlf_buffer[..] == b"\r\n\r\n";
                if is_double_crlf {
                    let end = bytes.current_pos() - 1;
                    let vec = bytes.copy_buffer(start, end);
                    return Ok(PartialRequest::Complete(vec));
                }
            }

            Some(b) => {
                push_to_buffer(&mut outstanding_crlf_buffer, b);
            }

            None => {
                let end = bytes.current_pos() - 1;
                return return_partial(bytes, start, end);
            }
        };
    }
}

fn push_to_buffer(buf: &mut [u8], byte: u8) {
    let len = buf.len();

    for i in 1..len {
        buf[i - 1] = buf[i]
    }

    buf[len - 1] = byte;
}

#[cfg(test)]
mod request_tests {
    use super::{HttpMethods, HttpVersion, PartialRequest, RequestBuilder};

    #[test]
    fn test_one_pass_parse() {
        let buffer = b"\r\n\r\n\n\nGET /abc HTTP/1.1\r\nAccept: */*\r\nUser-agent: abc\r\n\r\n";
        let mut builder = RequestBuilder::new();
        builder.parse(buffer, buffer.len());

        assert!(builder.method.is_some());
        assert!(builder.uri.is_some());
        assert!(builder.http_version.is_some());
        assert!(builder.headers.is_some());

        assert_match!(
            builder.method,
            Some(HttpMethods::GET),
            "expected method {:?}; found {:?}",
            HttpMethods::GET,
            builder.method
        );

        assert_eq!(builder.uri.unwrap().uri(), &String::from("/abc"));
        assert_match!(
            builder.http_version,
            Some(HttpVersion::Http11),
            "expected version {:?}; found {:?}",
            HttpVersion::Http11,
            builder.http_version
        );
    }

    #[test]
    fn test_many_pass_parse() {
        let buffer = b"GE";
        let mut builder = RequestBuilder::new();
        builder.parse(buffer, buffer.len());

        assert!(builder.method.is_none());
        assert!(builder.uri.is_none());
        assert!(builder.http_version.is_none());
        assert!(builder.partial.has_partial());

        let buffer = b"T /ab";
        builder.parse(buffer, buffer.len());

        assert!(builder.method.is_some());
        assert!(builder.uri.is_none());
        assert!(builder.http_version.is_none());
        assert!(builder.partial.has_partial());

        assert_match!(builder.method, Some(HttpMethods::GET));
        assert_match!(builder.partial, PartialRequest::RequestUri(_));

        let buffer = b"cd ";
        let empty: Vec<u8> = vec![];
        builder.parse(buffer, buffer.len());

        assert!(builder.method.is_some());
        assert!(builder.uri.is_some());
        assert!(builder.http_version.is_none());
        assert!(builder.partial.has_partial());

        let buffer = b"HTTP";
        builder.parse(buffer, buffer.len());

        assert!(builder.method.is_some());
        assert!(builder.uri.is_some());
        assert!(builder.http_version.is_none());
        assert!(builder.partial.has_partial());

        assert_match!(builder.partial, PartialRequest::HttpVersion(_));

        let buffer = b"/1.1\r\n";
        builder.parse(buffer, buffer.len());

        assert!(builder.method.is_some());
        assert!(builder.uri.is_some());
        assert!(builder.http_version.is_some());

        assert_match!(builder.http_version, Some(HttpVersion::Http11));

        let buffer = b"Accept: */*\r";
        builder.parse(buffer, buffer.len());
        let buffer = b"\n";
        builder.parse(buffer, buffer.len());
        let buffer = b"User-agent: abc\r\n";
        builder.parse(buffer, buffer.len());
        let buffer = b"Content-length: 4\r\n";
        builder.parse(buffer, buffer.len());
        let buffer = b"\r";
        builder.parse(buffer, buffer.len());
        let buffer = b"\n";
        builder.parse(buffer, buffer.len());

        assert!(builder.method.is_some());
        assert!(builder.uri.is_some());
        assert!(builder.http_version.is_some());
        assert!(builder.headers.is_some());
        assert!(builder.partial.is_uninit());

        println!("{:?}", builder.build());
    }
}
