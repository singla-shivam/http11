use crate::headers::Headers;
use crate::request::{HttpMethods, HttpVersion, RequestBody, RequestUri};
use std::collections::LinkedList;
use std::fmt;

#[derive(Debug)]
pub struct Request {
    method: HttpMethods,
    uri: RequestUri,
    http_version: HttpVersion,
    body: Option<RequestBody>,
    headers: Headers,
}

impl Request {
    pub fn new(
        method: HttpMethods,
        uri: RequestUri,
        http_version: HttpVersion,
        headers: Headers,
        body: Option<RequestBody>,
    ) -> Request {
        Request {
            method,
            uri,
            http_version,
            body,
            headers,
        }
    }
}

// impl fmt::Debug for Request {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.debug_tuple("")
//             .field(&String::from_utf8_lossy(&self.body.to_vec()))
//             .finish()
//     }
// }
