use crate::headers::{Header, RequestHeader};
use std::any::Any;

pub struct AcceptHeader {
    typ: String,
    sub_type: String, // TODO (@vedant) add other fields
}

impl Header for AcceptHeader {
    fn name(&self) -> &str {
        "accept"
    }

    fn value(&self) -> String {
        // TODO
        unimplemented!();
    }

    fn header_string(&self) -> String {
        let s = format!("{}: {}", self.name(), self.value());
        return s;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl RequestHeader for AcceptHeader {}
