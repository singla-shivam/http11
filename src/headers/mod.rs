pub trait Header<'a> {
    const NAME: &'a str;
    fn get_value() -> String;
}

pub trait GeneralHeader<'a>: Header<'a> {}

pub trait RequestHeader<'a>: Header<'a> {}

pub trait ResponseHeader<'a>: Header<'a> {}

mod accept;
pub use accept::*;
