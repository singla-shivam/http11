use crate::request_uri::RequestUri;

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

pub struct Request<T> {
    method: HttpMethods,
    uri: RequestUri,
    http_version: HttpVersion,
    body: T,
}
