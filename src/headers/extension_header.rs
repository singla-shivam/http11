use crate::headers::{EntityHeader, Header};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct ExtensionHeader<'a> {
    name: &'a str,
    value: &'a str,
}

impl<'a> ExtensionHeader<'a> {
    pub(crate) fn new(name: &'a str, value: &'a str) -> ExtensionHeader<'a> {
        ExtensionHeader { name, value }
    }
}

impl<'a> Header<'a> for ExtensionHeader<'a> {
    fn name(&self) -> &'a str {
        "extension-header"
    }

    fn value(&self) -> &'a str {
        self.value
    }
}

impl<'a> EntityHeader<'a> for ExtensionHeader<'a> {}
