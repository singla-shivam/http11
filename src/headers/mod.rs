use crate::errors::Error as HttpError;
use crate::grammar::{
    is_cr, is_horizontal_tab, is_space, is_token_char, replace_white_space,
    to_lower_case,
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
mod transfer_encoding;
pub use accept::*;
pub use content_length::*;
pub use extension_header::*;
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

const TRANSFER_ENCODING_HEADER_NAME: &str = "transfer-encoding";
const ACCEPT_HEADER_NAME: &str = "accept";
const CONTENT_LENGTH_HEADER_NAME: &str = "content-length";
const EXTENSION_HEADER_NAME: &str = "extension-header";

macro_rules! get_header {
    ($(
        $(#[$docs:meta])*
        ($name1:ident, $name2:ident);
    )*) => {
        $(
            paste! {
                $(#[$docs])*
                pub fn [<$name1:snake>](&self) -> Option<&[<$name1>]> {
                    let h = self.headers.get($name2);
                    match h {
                        None => None,
                        Some(x) => x.as_any().downcast_ref::<[<$name1>]>(),
                    }
                }
            }
        )*
    };
}

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
    fn get_header_struct(
        buffer: &Vec<u8>,
        header_name_start: usize,
        header_name_end: usize,
        header_value_start: usize,
        header_end: usize,
    ) -> Option<Box<dyn Header>> {
        let header_name_buffer = &buffer[header_name_start..header_name_end];
        let header_value_buffer = &buffer[header_value_start..header_end];

        let name = str::from_utf8(header_name_buffer).unwrap();
        let value = str::from_utf8(header_value_buffer).unwrap();
        let value = replace_white_space(value);
        let value = value.as_str().trim();
        // TODO check grammar for `value`

        if value == "" {
            return None;
        }

        let header: Box<dyn Header> = match name {
            ACCEPT_HEADER_NAME => Box::new(ExtensionHeader::new(name, value)),
            CONTENT_LENGTH_HEADER_NAME => {
                Box::new(ContentLength::try_from(value).unwrap())
            }
            TRANSFER_ENCODING_HEADER_NAME => {
                Box::new(TransferEncoding::try_from(value).unwrap())
            }
            _ => Box::new(ExtensionHeader::new(name, value)),
        };

        println!("value: {:?}", header.value());
        Some(header)
    }

    get_header! {
        (TransferEncoding, TRANSFER_ENCODING_HEADER_NAME);
        (ContentLength, CONTENT_LENGTH_HEADER_NAME);
    }
}

impl<'a> TryFrom<Vec<u8>> for Headers {
    type Error = HttpError;
    fn try_from(mut value: Vec<u8>) -> Result<Self, Self::Error> {
        let length = value.len();
        for i in 0..length {
            value[i] = to_lower_case(value[i]);
        }

        let mut headers = HashMap::new();

        let mut is_looking_name = true;
        let mut header_name_start = 0;
        let mut header_name_end = 0;
        let mut header_value_start = 0;
        let mut value_iter = value.iter().enumerate();

        while let Some((i, byte)) = value_iter.next() {
            let byte = to_lower_case(*byte);

            if is_looking_name {
                match byte {
                    b':' => {
                        header_name_end = i;
                        is_looking_name = false;
                        header_value_start = i + 1;
                        continue;
                    }
                    _ => (),
                }
                if !is_token_char(byte) {
                    return Err(HttpError::InvalidHeaderFieldToken(
                        byte.to_string(),
                    ));
                }
            }

            match byte {
                b'\r' => {
                    let next_byte = value_iter.next();
                    // no more byte after '\r'
                    if next_byte.is_none() {
                        return Err(HttpError::InvalidCrlf(
                            "No character after \\r".to_string(),
                        ));
                    }

                    let (i, next_byte) = next_byte.unwrap();
                    // if there is not \n after \r
                    if *next_byte != b'\n' {
                        let error_string =
                            format!("Char: {}, in body chunk", next_byte);
                        return Err(HttpError::InvalidCrlf(error_string));
                    }

                    let next_byte = value[i + 1];

                    // There was LWS indicating continuation of the field-value
                    // It is deprecated in RFC 7230
                    // https://tools.ietf.org/pdf/rfc7230.pdf#page=26
                    if is_space(next_byte) || is_horizontal_tab(next_byte) {
                        // TODO respond with 400
                        return Err(HttpError::InvalidHeaderFormat(
                            "The line continuation with space or tab is not allowed.".to_string(),
                        ));
                    }

                    let header = Headers::get_header_struct(
                        &value,
                        header_name_start,
                        header_name_end,
                        header_value_start,
                        i - 1, // i points to \n
                    );

                    if let Some(header) = header {
                        // TODO handle multiple header values
                        headers.insert(String::from(header.name()), header);
                    }

                    header_name_start = i + 1;
                    is_looking_name = true;

                    if is_cr(next_byte) {
                        // TODO it was last CRLF- done parsing
                        unimplemented!();
                    }
                }
                _ => continue,
            }
        }

        Ok(Headers { headers })
    }
}

#[cfg(test)]
mod tests {
    use super::Headers;
    use crate::assert_match;
    use crate::errors::Error;
    use std::convert::TryFrom;

    #[test]
    fn test_respond_throw_error_invalid_header_with_space_before_colon() {
        // TODO write integrations test too to check it
        let buffer = b"accept : */*\r\nuser-agent : abc\r\n\r\n";
        let result = Headers::try_from(buffer.to_vec());
        assert!(result.is_err());

        let result = result.err().unwrap();
        let space_string = String::from("32");
        assert_eq!(
            result.to_string(),
            Error::InvalidHeaderFieldToken(space_string).to_string()
        );
    }

    #[test]
    fn test_respond_throw_error_for_header_field_continuation() {
        // TODO write integrations test too to check it
        let buffer = b"accept: */*\r\nuser-agent: abc\r\n\tcontinued\r\n\r\n";
        let result = Headers::try_from(buffer.to_vec());
        assert!(result.is_err());

        let result = result.err().unwrap();
        let space_string = String::from(
            "The line continuation with space or tab is not allowed.",
        );
        assert_eq!(
            result.to_string(),
            Error::InvalidHeaderFormat(space_string).to_string()
        );
    }
}
