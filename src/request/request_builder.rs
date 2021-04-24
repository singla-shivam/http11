use crate::errors::Error;
use crate::grammar::is_token_char;
use crate::helpers::bytes::Bytes;
use crate::request::RequestUri;
use std::marker::PhantomData;
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

#[derive(Debug)]
pub enum HttpVersion {
    Http11,
}

impl HttpVersion {
    pub fn from_vector(v: Vec<u8>) -> Result<HttpVersion, Error> {
        let version = String::from_utf8(v).unwrap().to_lowercase();
        println!("{}", version);

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
        let uri = RequestUri::from_vector(buffer).expect("invalid uri");
        self.uri = Some(uri);

        if is_partial_used {
            self.partial = PartialRequest::Uninit;
        }

        self
    }

    // TODO (singla-shivam) remove repetitive code
    fn add_http_version(&mut self, v: Vec<u8>) -> &RequestBuilder<'buf, T> {
        let mut is_partial_used = false;
        let buffer = if let PartialRequest::HttpVersion(ref mut v1) = self.partial {
            v1.extend_from_slice(&*v);
            is_partial_used = true;
            v1
        } else {
            &*v
        };

        let buffer = buffer.to_vec();
        let version = HttpVersion::from_vector(buffer).expect("invalid http version");
        self.http_version = Some(version);

        if is_partial_used {
            self.partial = PartialRequest::Uninit;
        }

        self
    }

    fn add_partial(&mut self, new_partial: PartialRequest) -> &RequestBuilder<'buf, T> {
        self.partial = new_partial;
        self
    }

    fn parse_method(&mut self, bytes: &mut Bytes) -> &mut RequestBuilder<'buf, T> {
        if self.method.is_some() {
            return self;
        }

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

    fn parse_request_uri(&mut self, bytes: &mut Bytes) -> &mut RequestBuilder<'buf, T> {
        if self.uri.is_some() {
            return self;
        }

        if let PartialRequest::RequestUri(_) = self.partial {
        } else if let PartialRequest::Uninit = self.partial {
        } else {
            return self;
        }

        let request_uri = parse_request_uri(bytes).unwrap();

        match request_uri {
            PartialRequest::RequestUri(ref v) => {
                // TODO (singla-shivam) improve this new copy
                let v = v.to_vec();
                self.add_partial(PartialRequest::RequestUri(v));
            }

            PartialRequest::Complete(v) => {
                self.add_request_uri(v);
            }

            _ => (),
        }

        self
    }

    fn parse_http_version(&mut self, bytes: &mut Bytes) -> &mut RequestBuilder<'buf, T> {
        if self.http_version.is_some() {
            return self;
        }

        if let PartialRequest::HttpVersion(_) = self.partial {
        } else if let PartialRequest::Uninit = self.partial {
        } else {
            return self;
        }

        let http_version = parse_http_version(bytes).expect("Invalid http version");

        match http_version {
            PartialRequest::HttpVersion(ref v) => {
                // TODO (singla-shivam) improve this new copy
                let v = v.to_vec();
                self.add_partial(PartialRequest::HttpVersion(v));
            }

            PartialRequest::Complete(v) => {
                self.add_http_version(v);
            }

            _ => (),
        }

        self
    }

    pub(crate) fn parse(buf: &'buf [u8]) -> RequestBuilder<'buf, T> {
        let mut request_builder = RequestBuilder::new();
        let mut bytes = Bytes::new(buf);
        skip_initial_crlf(&mut bytes);

        request_builder
            .parse_method(&mut bytes)
            .parse_request_uri(&mut bytes)
            .parse_http_version(&mut bytes);

        request_builder
    }

    pub(crate) fn parse_more(&mut self, buf: &'buf [u8]) -> &RequestBuilder<'buf, T> {
        if self.is_parsed() && !self.partial.has_partial() {
            return self;
        }

        let mut bytes = Bytes::new(buf);

        match &self.partial {
            PartialRequest::Method(_) => {
                self.parse_method(&mut bytes);
            }

            PartialRequest::RequestUri(_) => {
                self.parse_request_uri(&mut bytes);
            }

            PartialRequest::HttpVersion(_) => {
                self.parse_http_version(&mut bytes);
            }

            _ => (),
        }

        self.parse_method(&mut bytes)
            .parse_request_uri(&mut bytes)
            .parse_http_version(&mut bytes);

        self
    }
}

fn skip_initial_crlf(bytes: &mut Bytes) {
    loop {
        let byte = bytes.peek();

        match byte {
            Some(b'\r') => {
                bytes.bump();
                let next_byte = bytes.bump().unwrap();
                assert!(next_byte == b'\n', "expected \\n but found {}", next_byte);
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

        if byte == Some(b' ') || byte == None {
            let end = bytes.current_pos() - 1;

            if byte == None {
                let vec = bytes.copy_buffer(start, end);
                return Ok(on_found(vec));
            }

            let vec = bytes.copy_buffer(start, end - 1); // remove space

            return Ok(PartialRequest::Complete(vec));
        }
    }
}

fn parse_token(bytes: &mut Bytes) -> Result<PartialRequest, Error> {
    let on_empty = || PartialRequest::Token(vec![]);
    let on_found = |vec: Vec<u8>| PartialRequest::Token(vec);

    let verify_character = |byte: u8| {
        if !is_token_char(byte) {
            println!("not token byte {}", byte);
            return Err(Error::Token);
        }

        Ok(())
    };

    get_token_buffer(bytes, on_empty, on_found, verify_character)
}

fn parse_http_version(bytes: &mut Bytes) -> Result<PartialRequest, Error> {
    let on_empty = || PartialRequest::HttpVersion(vec![]);
    let on_found = |vec: Vec<u8>| PartialRequest::HttpVersion(vec);
    let verify_character = |_: u8| Ok(());

    get_token_buffer(bytes, on_empty, on_found, verify_character)
}

fn parse_request_uri(bytes: &mut Bytes) -> Result<PartialRequest, Error> {
    let on_empty = || PartialRequest::RequestUri(vec![]);
    let on_found = |vec: Vec<u8>| PartialRequest::RequestUri(vec);
    let verify_character = |_: u8| Ok(());

    get_token_buffer(bytes, on_empty, on_found, verify_character)
}

#[cfg(test)]
mod request_tests {
    use super::{HttpMethods, HttpVersion, PartialRequest, RequestBuilder};

    #[test]
    fn test_remove_initial_empty_lines() {
        let buffer = b"\r\n\r\n\n\nGET HTTP/1.1";
        let builder = RequestBuilder::<String>::parse(buffer);
        assert!(builder.method.is_some());
    }

    #[test]
    fn test_use_parse_more() {
        let buffer = b"GE";
        let mut builder = RequestBuilder::<String>::parse(buffer);
        assert!(builder.method.is_none());
        assert!(builder.partial.has_partial());

        let buffer = b"T ";
        builder.parse_more(buffer);

        assert!(builder.method.is_some());
        assert_match!(
            builder.method,
            Some(HttpMethods::GET),
            "expected method {:?}; found {:?}",
            HttpMethods::GET,
            builder.method
        );

        let empty: Vec<u8> = vec![];
        assert_match!(builder.partial, PartialRequest::RequestUri(empty));
    }

    #[test]
    fn test_parse_uri() {
        let buffer = b"\r\n\r\n\n\nGET /abc ";
        let builder = RequestBuilder::<String>::parse(buffer);

        assert!(builder.method.is_some());
        assert!(builder.uri.is_some());
        assert_match!(builder.uri, Some(..));

        assert_eq!(builder.uri.unwrap().uri(), &String::from("/abc"));
    }

    #[test]
    fn test_one_pass_parse() {
        let buffer = b"\r\n\r\n\n\nGET /abc HTTP/1.1 ";
        let builder = RequestBuilder::<String>::parse(buffer);

        assert!(builder.method.is_some());
        assert!(builder.uri.is_some());
        assert!(builder.http_version.is_some());

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

    // #[test]
    // fn test_parse_more() {
    //     let buffer = b"GE";
    //     let mut builder = RequestBuilder::<String>::parse(buffer);

    //     assert!(builder.method.is_none());
    //     assert!(builder.uri.is_none());
    //     assert!(builder.http_version.is_none());
    //     assert!(builder.partial.has_partial());

    //     let buffer = b"T /ab";
    //     builder.parse_more(buffer);

    //     assert!(builder.method.is_some());
    //     assert!(builder.uri.is_none());
    //     assert!(builder.http_version.is_none());
    //     assert!(builder.partial.has_partial());

    //     assert_match!(builder.method, Some(HttpMethods::GET));
    //     assert_match!(builder.partial, PartialRequest::RequestUri(_));

    //     let buffer = b"cd ";
    //     let empty: Vec<u8> = vec![];
    //     builder.parse_more(buffer);

    //     assert!(builder.method.is_some());
    //     assert!(builder.uri.is_some());
    //     assert!(builder.http_version.is_none());
    //     assert!(builder.partial.has_partial());

    //     // assert_eq!(&builder.uri.unwrap().uri().clone()[..], "/abcd");
    //     // assert_match!(builder.partial, PartialRequest::HttpVersion(empty));

    //     let buffer = b"HTTP";
    //     builder.parse_more(buffer);

    //     assert!(builder.method.is_some());
    //     assert!(builder.uri.is_some());
    //     assert!(builder.http_version.is_none());
    //     assert!(builder.partial.has_partial());

    //     assert_match!(builder.partial, PartialRequest::HttpVersion(_));

    //     let buffer = b"/1.1 ";
    //     builder.parse_more(buffer);

    //     assert!(builder.method.is_some());
    //     assert!(builder.uri.is_some());
    //     assert!(builder.http_version.is_some());
    //     assert!(builder.partial.has_partial());

    //     assert_match!(builder.http_version, Some(HttpVersion::Http11));
    //     assert_match!(builder.partial, PartialRequest::Uninit);
    // }
}
