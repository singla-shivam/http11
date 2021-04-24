pub struct RequestUri {
    uri: String,
}

impl RequestUri {
    pub fn from_vector(v: Vec<u8>) -> RequestUri {
        RequestUri {
            uri: String::from_utf8(v).unwrap(),
        }
    }

    pub fn uri(&self) -> &String {
        &self.uri
    }
}
