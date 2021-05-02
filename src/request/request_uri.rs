use crate::errors::Error as HttpError;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct RequestUri {
    uri: String,
}

impl RequestUri {
    pub fn uri(&self) -> &String {
        &self.uri
    }
}

impl TryFrom<Vec<u8>> for RequestUri {
    type Error = HttpError;

    fn try_from(v: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(RequestUri {
            uri: String::from_utf8(v).unwrap(),
        })
    }
}
