use crate::headers::{GeneralHeader, Header};

enum TransferEncodingValue<'a> {
    Chunked,
    Extension(&'a str),
}

impl<'a> TransferEncodingValue<'a> {
    fn get_str(&self) -> &'a str {
        match self {
            TransferEncodingValue::Chunked => "chunked",
            TransferEncodingValue::Extension(str) => str,
        }
    }
}

pub struct TransferEncoding<'a> {
    value: TransferEncodingValue<'a>,
}

impl<'a> TransferEncoding<'a> {
    pub(crate) fn new(value: &'a str) -> TransferEncoding<'a> {
        let value = value.trim();
        match value {
            "chunked" => TransferEncoding {
                value: TransferEncodingValue::Chunked,
            },
            _ => TransferEncoding {
                value: TransferEncodingValue::Chunked,
            },
        }
    }
}

impl<'a> Header<'a> for TransferEncoding<'a> {
    fn name(&self) -> &'a str {
        "transfer-encoding"
    }

    fn value(&self) -> &'a str {
        self.value.get_str()
    }
}

impl<'a> GeneralHeader<'a> for TransferEncoding<'a> {}
