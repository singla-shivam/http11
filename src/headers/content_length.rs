use crate::errors::Error;
use crate::headers::{EntityHeader, Header};
use std::str;

pub struct ContentLength<'a> {
    length: &'a str,
}

impl<'a> ContentLength<'a> {
    pub fn try_from_vec(
        v: &Vec<u8>,
        start: usize,
        end: usize,
    ) -> Result<ContentLength, Error> {
        let buffer = &v[start..end];
        let str =
            str::from_utf8(buffer).map_err(|_| Error::InvalidUtf8String)?;

        let str = str.trim();
        str.parse::<usize>()
            .map_err(|_| Error::InvalidContentLengthValue)?;

        let length = 1;

        Ok(ContentLength { length: str })
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

    fn header_string(&self) -> String {
        let s = format!("{}: {}", self.name(), self.value());
        return s;
    }
}

impl<'a> EntityHeader<'a> for ContentLength<'a> {}
