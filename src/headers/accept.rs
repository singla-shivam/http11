use crate::headers::{Header, RequestHeader};

pub struct AcceptHeader<'a> {
    typ: &'a str,
    sub_type: &'a str, // TODO (@vedant) add other fields
}

impl<'a> Header<'a> for AcceptHeader<'a> {
    fn name(&self) -> &'a str {
        "accept"
    }

    fn value(&self) -> &'a str {
        // TODO
        unimplemented!();
    }

    fn header_string(&self) -> String {
        let s = format!("{}: {}", self.name(), self.value());
        return s;
    }
}

impl<'a> RequestHeader<'a> for AcceptHeader<'a> {}
