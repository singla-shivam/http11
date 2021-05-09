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

impl TryFrom<String> for RequestUri {
    type Error = HttpError;

    fn try_from(v: String) -> Result<Self, Self::Error> {
        Ok(RequestUri { uri: v })
    }
}
