use crate::errors::Error;

#[derive(Debug)]
pub struct RequestUri {
    uri: String,
}

impl RequestUri {
    pub fn from_vector(v: Vec<u8>) -> Result<RequestUri, Error> {
        // TODO (singla-shivam) parse URI correctly RFC2396
        Ok(RequestUri {
            uri: String::from_utf8(v).unwrap(),
        })
    }

    pub fn uri(&self) -> &String {
        &self.uri
    }
}
