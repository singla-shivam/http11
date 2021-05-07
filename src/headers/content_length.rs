use crate::errors::Error as HttpErrors;
use crate::headers::{EntityHeader, Header, CONTENT_LENGTH_HEADER_NAME};
use std::any::Any;
use std::convert::TryFrom;
use std::str;

pub struct ContentLength {
    length: String,
}

impl TryFrom<&str> for ContentLength {
    type Error = HttpErrors;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value
            .parse::<usize>()
            .map_err(|_| HttpErrors::InvalidContentLengthValue)?;

        Ok(ContentLength {
            length: String::from(value),
        })
    }
}

impl ContentLength {
    pub fn content_length(&self) -> usize {
        self.length.parse::<usize>().unwrap()
    }
}

impl Header for ContentLength {
    fn name(&self) -> &str {
        CONTENT_LENGTH_HEADER_NAME
    }

    fn value(&self) -> String {
        self.length.clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl EntityHeader for ContentLength {}
