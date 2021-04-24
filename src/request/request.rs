use crate::request::{HttpMethods, HttpVersion, PartialRequest, RequestUri};
use std::marker::PhantomData;

pub struct Request<'buf, T> {
    method: HttpMethods,
    uri: RequestUri,
    http_version: HttpVersion,
    // body: T,
    phantom: PhantomData<&'buf T>,
    partial: PartialRequest,
}
