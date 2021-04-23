use crate::headers::{Header, RequestHeader};

pub struct AcceptHeader<'a> {
    typ: &'a str,
    sub_type: &'a str, // TODO (@vedant) add other fields
}

impl<'a> Header<'a> for AcceptHeader<'a> {
    const NAME: &'a str = "accept";

    fn get_value() -> String {
        return String::from("abc");
    }
}

impl<'a> RequestHeader<'a> for AcceptHeader<'a> {}
