use crate::errors::Error as HttpErrors;
use crate::headers::{EntityHeader, Header};
use std::convert::TryFrom;
use std::str;

pub struct ContentLength<'a> {
    length: &'a str,
}

impl<'a> TryFrom<&'a str> for ContentLength<'a> {
    type Error = HttpErrors;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        value
            .parse::<usize>()
            .map_err(|_| HttpErrors::InvalidContentLengthValue)?;

        Ok(ContentLength { length: value })
    }
}

impl<'a> ContentLength<'a> {
    fn content_length(&self) -> usize {
        self.length.parse::<usize>().unwrap()
    }
}

impl<'a> Header<'a> for ContentLength<'a> {
    fn name(&self) -> &'a str {
        "content-length"
    }

    fn value(&self) -> &'a str {
        self.length
    }
}

impl<'a> EntityHeader<'a> for ContentLength<'a> {}
