use crate::errors::Error as HttpError;
use crate::grammar::{
    is_cr, is_horizontal_tab, is_space, is_token, is_token_char,
    is_vchar_sequence_with_white_space, replace_white_space, to_lower_case,
};
use crate::headers::{ContentLength, ExtensionHeader};
use paste::paste;
use std::any::Any;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::Debug;
use std::str;

mod accept;
mod content_length;
mod extension_header;
mod trailer;
mod transfer_encoding;
pub use accept::*;
pub use content_length::*;
pub use extension_header::*;
pub use trailer::*;
pub use transfer_encoding::*;

pub trait Header {
    fn name(&self) -> &str;
    fn value(&self) -> String;
    fn as_any(&self) -> &dyn Any;
    fn header_string(&self) -> String {
        format!("{}: {}", self.name(), self.value())
    }
}

pub trait GeneralHeader: Header {}

pub trait RequestHeader: Header {}

pub trait ResponseHeader: Header {}

pub trait EntityHeader: Header {}

macro_rules! apply_header_names {
    ($macro_name:ident) => {
        $macro_name! {
            "transfer-encoding";
            "content-length";
            "trailer";
        }
    };
}

macro_rules! header_names_constants {
    ($(
        $(#[$docs:meta])*
        $name:expr;
    )*) => {
        $(
            paste! {
                const [<$name:snake:upper _HEADER_NAME>]: &str = $name;
            }
        )*
    };
}

macro_rules! get_header {
    ($(
        $(#[$docs:meta])*
        $name:expr;
    )*) => {
        $(
            paste! {
                $(#[$docs])*
                pub fn [<$name:snake>](&self) -> Option<&[<$name:camel>]> {
                    let h = self.headers.get([<$name:snake:upper _HEADER_NAME>]);
                    match h {
                        None => None,
                        Some(x) => x.as_any().downcast_ref::<[<$name:camel>]>(),
                    }
                }
            }
        )*
    };
}

macro_rules! valid_headers {
    ($(
        $(#[$docs:meta])*
        $name:expr;
    )*) => {
        fn is_valid_header_name(field_name: &str) -> bool {
            paste! {
                match field_name {
                    $(
                        [<$name:snake:upper _HEADER_NAME>] => true,
                    )*
                    _ => false
                }
            }
        }
    };
}

macro_rules! get_header_struct {
    ($(
        $(#[$docs:meta])*
        $name:expr;
    )*) => {
        paste! {
            fn get_header_struct(name: &str, value: &str) -> Result<Option<Box<dyn Header>>, HttpError> {
                if value == "" {
                    return Ok(None);
                }
                let header: Box<dyn Header> = match name {
                    $(
                        [<$name:snake:upper _HEADER_NAME>] => {
                            let header = [<$name:camel>]::try_from(value)?;
                            Box::new(header)
                        },
                    )*
                    _ => Box::new(ExtensionHeader::new(name, value)),
                };
                Ok(Some(header))
            }
        }
    };
}

apply_header_names!(header_names_constants);
const ACCEPT_HEADER_NAME: &str = "accept";
const EXTENSION_HEADER_NAME: &str = "extension-header";

#[derive(Debug)]
pub struct Headers {
    headers: HashMap<String, Box<dyn Header>>,
}

impl Debug for dyn Header {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.header_string())
    }
}

impl Headers {
    apply_header_names!(get_header);
    apply_header_names!(valid_headers);
    apply_header_names!(get_header_struct);
}

impl<'a> TryFrom<String> for Headers {
    type Error = HttpError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut headers = HashMap::new();

        let parts = value.split("\r\n").filter(|p| p != &"");

        for part in parts {
            if is_continued_field(part) {
                return Err(HttpError::InvalidHeaderFormat(format!(
                    "The line continuation with space or tab is not allowed:- {}",
                    part
                )));
            }

            let colon_index = part.find(":");
            if colon_index.is_none() {
                return Err(HttpError::InvalidHeaderFormat(format!(
                    "The header field line does not contain colon:- {}",
                    part
                )));
            }

            let colon_index = colon_index.unwrap();
            let (name, value) = part.split_at(colon_index);
            let name = name.to_lowercase();
            let value = &value[1..];
            let value = replace_white_space(value.trim());
            let value = value.as_str();

            if !is_token(name.as_bytes()) {
                return Err(HttpError::InvalidHeaderField(format!(
                    "The header field-name has invalid character:- {}",
                    name
                )));
            }

            if !is_vchar_sequence_with_white_space(value.as_bytes()) {
                return Err(HttpError::InvalidHeaderFieldValue(format!(
                    "{}",
                    value
                )));
            }

            let header = Headers::get_header_struct(name.as_str(), value)?;
            if let Some(header) = header {
                headers.insert(header.name().to_string(), header);
            }
        }

        Ok(Headers { headers })
    }
}

fn is_continued_field(field: &str) -> bool {
    let space_index = field.find(" ");
    let tab_index = field.find("\t");

    match (space_index, tab_index) {
        (None, None) => false,
        (Some(0), _) | (_, Some(0)) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests_header {
    use super::Headers;
    use crate::errors::Error;
    use crate::{assert_match, assert_match_error};
    use std::convert::TryFrom;
    use std::str;

    #[test]
    fn test_no_header() {
        let buffer = "";
        let result = Headers::try_from(buffer.to_string());
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.headers.len(), 0);
        assert_eq!(result.headers.capacity(), 0);
    }

    #[test]
    fn test_respond_throw_error_invalid_header_with_space_before_colon() {
        // TODO write integrations test too to check it
        let buffer = "accept : */*\r\nuser-agent : abc";
        let result = Headers::try_from(buffer.to_string());
        assert!(result.is_err());

        let result = result.err().unwrap();
        let expected_error = Error::InvalidHeaderField(format!(
            "The header field-name has invalid character:- accept "
        ));
        assert_match_error!(result, expected_error);
    }

    #[test]
    fn test_throw_error_for_header_field_continuation() {
        // TODO write integrations test too to check it
        let buffer = "accept: */*\r\nuser-agent: abc\r\n\tcontinued";
        let result = Headers::try_from(buffer.to_string());
        assert!(result.is_err());

        let result = result.err().unwrap();
        let expected_error = Error::InvalidHeaderFormat(format!(
            "The line continuation with space or tab is not allowed:- \tcontinued"
        ));
        assert_match_error!(result, expected_error);
    }

    #[test]
    fn test_when_no_colon_in_header_field() {
        let buffer = "accept */*\r\nuser-agent: abc";
        let result = Headers::try_from(buffer.to_string());
        assert!(result.is_err());

        let result = result.err().unwrap();
        let expected_error = Error::InvalidHeaderFormat(format!(
            "The header field line does not contain colon:- accept */*"
        ));
        assert_match_error!(result, expected_error);
    }

    #[test]
    fn test_invalid_char_in_value() {
        let buffer = "accept: abcdfd\u{12}";
        let result = Headers::try_from(buffer.to_string());
        assert!(result.is_err());

        let result = result.err().unwrap();
        let expected_error =
            Error::InvalidHeaderFieldValue(format!("abcdfd\u{12}"));
        assert_match_error!(result, expected_error);
    }

    #[test]
    fn test_valid_char_in_value() {
        let mut buffer = "accept: ab cd\t".to_string();
        for i in 33..=126 {
            buffer += str::from_utf8(&[i]).unwrap();
        }

        let result = Headers::try_from(buffer);
        assert!(result.is_ok());
    }
}
