use crate::headers::{EntityHeader, Header, EXTENSION_HEADER_NAME};
use std::any::Any;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct ExtensionHeader {
    name: String,
    value: String,
}

impl ExtensionHeader {
    pub(crate) fn new(name: &str, value: &str) -> ExtensionHeader {
        ExtensionHeader {
            name: String::from(name),
            value: String::from(value),
        }
    }
}

impl Header for ExtensionHeader {
    fn name(&self) -> &str {
        EXTENSION_HEADER_NAME
    }

    fn value(&self) -> String {
        self.value.clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl EntityHeader for ExtensionHeader {}
