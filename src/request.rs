use crate::errors::Error;
use crate::grammar::is_token_char;
use crate::helpers::bytes::Bytes;
use crate::request_uri::RequestUri;
use std::marker::PhantomData;
use std::vec::Vec;

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

        // HttpMethods::GET;
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

pub enum HttpVersion {
    Http11,
}

#[derive(PartialEq)]
enum PartialRequest {
    Method(Vec<u8>),
    RequestUri(Vec<u8>),
    HttpVersion(Vec<u8>),
    Token(Vec<u8>),
    Complete(Vec<u8>),
    Uninit,
}

pub struct RequestBuilder<'buf, T> {
    method: Option<HttpMethods>,
    uri: Option<RequestUri>,
    http_version: Option<HttpVersion>,
    // body: T,
    phantom: PhantomData<&'buf T>,
    partial: PartialRequest,
}

pub struct Request<'buf, T> {
    method: HttpMethods,
    uri: RequestUri,
    http_version: HttpVersion,
    // body: T,
    phantom: PhantomData<&'buf T>,
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
}

impl<'buf, T> RequestBuilder<'buf, T> {
    fn new() -> RequestBuilder<'buf, T> {
        RequestBuilder {
            method: None,
            uri: None,
            http_version: None,
            phantom: PhantomData,
            partial: PartialRequest::Uninit,
        }
    }

    fn is_parsed(&self) -> bool {
        self.method.is_some()
            && self.uri.is_some()
            && self.http_version.is_some()
            && !self.partial.has_partial()
    }

    fn add_method(&mut self, v: Vec<u8>) -> &RequestBuilder<'buf, T> {
        let mut is_partial_used = false;
        let buffer = if let PartialRequest::Method(ref mut v1) = self.partial {
            v1.extend_from_slice(&*v);
            is_partial_used = true;
            v1
        } else {
            &*v
        };

        let buffer = buffer.to_vec();
        let method = HttpMethods::from_vector(buffer);
        self.method = Some(method);

        if is_partial_used {
            self.partial = PartialRequest::Uninit;
        }

        self
    }

    // TODO (singla-shivam) remove repetitive code
    fn add_request_uri(&mut self, v: Vec<u8>) -> &RequestBuilder<'buf, T> {
        let mut is_partial_used = false;
        let buffer = if let PartialRequest::RequestUri(ref mut v1) = self.partial {
            v1.extend_from_slice(&*v);
            is_partial_used = true;
            v1
        } else {
            &*v
        };

        let buffer = buffer.to_vec();
        let uri = RequestUri::from_vector(buffer);
        self.uri = Some(uri);

        if is_partial_used {
            self.partial = PartialRequest::Uninit;
        }

        self
    }

    fn add_partial(&mut self, new_partial: PartialRequest) -> &RequestBuilder<'buf, T> {
        self.partial = new_partial;
        self
    }

    fn parse_method(&mut self, bytes: &mut Bytes) -> &RequestBuilder<'buf, T> {
        let method = parse_token(bytes).unwrap();

        match method {
            PartialRequest::Token(ref v) => {
                let v = v.to_vec();
                self.add_partial(PartialRequest::Method(v));
            }

            PartialRequest::Complete(v) => {
                self.add_method(v);
            }

            _ => (),
        }

        self
    }

    fn parse_request_uri(&mut self, bytes: &mut Bytes) -> &RequestBuilder<'buf, T> {
        let method = parse_token(bytes).unwrap();

        match method {
            PartialRequest::Token(ref v) => {
                let v = v.to_vec();
                self.add_partial(PartialRequest::RequestUri(v));
            }

            PartialRequest::Complete(v) => {
                self.add_method(v);
            }

            _ => (),
        }

        self
    }

    pub(crate) fn parse(buf: &'buf [u8]) -> RequestBuilder<'buf, T> {
        let mut request_builder = RequestBuilder::new();
        let mut bytes = Bytes::new(buf);
        skip_initial_crlf(&mut bytes);

        request_builder.parse_method(&mut bytes);
        request_builder
    }

    pub(crate) fn parse_more(&mut self, buf: &'buf [u8]) -> &RequestBuilder<'buf, T> {
        if self.is_parsed() && !self.partial.has_partial() {
            return self;
        }

        self
    }
}

fn skip_initial_crlf(bytes: &mut Bytes) {
    loop {
        let byte = bytes.peek();

        match byte {
            Some(b'\r') => {
                let next_byte = bytes.peek().unwrap();
                assert!(next_byte == b'\n', Error::NewLine);
                bytes.bump();
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

fn parse_token(bytes: &mut Bytes) -> Result<PartialRequest, Error> {
    let start = bytes.current_pos();
    loop {
        let byte = bytes.next();

        if byte == Some(b' ') || byte == None {
            let end = bytes.current_pos() - 1;
            let vec = bytes.copy_buffer(start, end);

            if byte == None {
                return Ok(PartialRequest::Token(vec));
            }

            return Ok(PartialRequest::Complete(vec));
        } else if !is_token_char(byte.unwrap()) {
            return Err(Error::Token);
        }
    }
}

// fn parse_request_uri(bytes: &mut Bytes) -> Result<PartialRequest, Error> {}

#[cfg(tests)]
mod tests {}
