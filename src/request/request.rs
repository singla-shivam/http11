use crate::request::request_builder::{HttpMethods, HttpVersion, PartialRequest};
use crate::request::RequestUri;
use std::marker::PhantomData;

pub struct Request<'buf, T> {
    method: HttpMethods,
    uri: RequestUri,
    http_version: HttpVersion,
    // body: T,
    phantom: PhantomData<&'buf T>,
    partial: PartialRequest,
}
