use crate::errors::Error;
use crate::helpers::bytes::Bytes;
use crate::request_uri::RequestUri;
use std::marker::PhantomData;

pub enum HttpMethods {
    GET,
    OPTIONS,
    HEAD,
    POST,
    PUT,
    DELETE,
    TRACE,
    CONNECT,
    Extension(&'static str),
}

pub enum HttpVersion {
    Http11,
}

enum PartialRequest {
    Method(),
    RequestUri(),
    HttpVersion(),
}

pub struct Request<'buf, T> {
    method: HttpMethods,
    uri: RequestUri,
    http_version: HttpVersion,
    body: T,
    phantom: PhantomData<&'buf T>,
    partial: PartialRequest
}

impl<'buf, T> Request<'buf, T> {
    pub(crate) fn parse(buf: &'buf [u8]) {
        let mut bytes = Bytes::new(buf);
        skip_initial_crlf(&mut bytes);
        println!("{}", String::from_utf8_lossy(bytes.buffer()));
    }
}

fn skip_initial_crlf(bytes: &mut Bytes) {
    loop {
        let byte = bytes.peek();

        match byte {
            Some(b'\r') => {
                let next_byte = bytes.peek().unwrap();
                assert!(next_byte == b'\n', Error::NewLine);
                bytes.next();
            }
            Some(b'\n') => {
                bytes.next();
            }
            Some(_) => {
                return;
            }
            None => return (),
        }
    }
}

fn parse_token() {

}
