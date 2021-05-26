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
pub mod app;
mod connection;
mod http11_server;
mod request;
pub mod response;
pub mod status;

pub use crate::http11_server::*;
pub use app::App;
pub use request::{Request, RequestBuilder};
