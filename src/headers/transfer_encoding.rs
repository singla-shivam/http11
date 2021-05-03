use crate::errors::Error as HttpErrors;
use crate::headers::{GeneralHeader, Header, TRANSFER_ENCODING_HEADER_NAME};
use std::convert::TryFrom;
use std::any::Any;

pub struct TransferEncoding {
    encodings: Vec<TransferEncodingValue>,
}

impl TransferEncoding {
    /// Check if the final encoding is chunked or not
    pub fn is_chunked(&self) -> bool {
        // if not encoding is present
        if !self.has_transfer_encoding() {
            return false;
        }

        let last_encoding = self.encodings.last().unwrap();

        match last_encoding {
            TransferEncodingValue::Chunked => true,
            _ => false,
        }
    }

    pub fn has_transfer_encoding(&self) -> bool {
        self.encodings.len() > 0
    }

    pub fn clone() -> TransferEncoding {
        TransferEncoding { encodings: vec![] }
    }
}

impl TryFrom<&str> for TransferEncoding {
    type Error = HttpErrors;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value: Vec<&str> = value.split(",").collect();
        let value = value
            .iter()
            .map(|v| v.trim())
            .map(TransferEncodingValue::from)
            .collect();

        Ok(TransferEncoding { encodings: value })
    }
}

impl Header for TransferEncoding {
    fn name(&self) -> &str {
        TRANSFER_ENCODING_HEADER_NAME
    }

    fn value(&self) -> String {
        let list: Vec<&str> =
            self.encodings.iter().map(|e| e.to_string()).collect();

        list.join(", ")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl GeneralHeader for TransferEncoding {}

macro_rules! transfer_encoding_values {
    ($(
        $(#[$docs:meta])*
        ($name:ident, $phrase:expr);
    )+) => {
        pub(crate) enum TransferEncodingValue {
            $(
                $(#[$docs])*
                $name,
            )+
            Extension(String),
        }

        impl From<&str> for TransferEncodingValue {
            fn from(value: &str) -> Self {
                match value {
                    $(
                        $phrase => TransferEncodingValue::$name,
                    )+
                    _ => TransferEncodingValue::Extension(String::from(value))
                }
            }
        }

        impl TransferEncodingValue {
            fn to_string(&self) -> &str {
                match self {
                    $(
                        TransferEncodingValue::$name => $phrase,
                    )+
                    TransferEncodingValue::Extension(s) => s
                }
            }
        }
    };
}

transfer_encoding_values! {
    (Chunked, "chunked");
}
