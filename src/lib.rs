#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_must_use, unused_variables)
)]

#[macro_use]
extern crate lazy_static;

pub mod errors;
mod grammar;
pub mod headers;
#[macro_use]
mod helpers;
mod http11;
mod request;
pub mod request_uri;
pub mod response;
pub mod status;

pub use crate::http11::*;
pub use request::{Request, RequestBuilder};

