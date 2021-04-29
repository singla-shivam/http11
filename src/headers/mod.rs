use crate::errors::Error as HttpError;
use crate::grammar::{
    is_cr, is_horizontal_tab, is_space, is_token_char, to_lower_case,
};
use crate::headers::{ContentLength, ExtensionHeader};
use std::convert::TryFrom;
use std::fmt::Debug;
use std::str;
// use std::marker::PhantomData;

pub trait Header<'a> {
    fn name(&self) -> &'a str;
    fn value(&self) -> &'a str;
    fn header_string(&self) -> String;
}

pub trait GeneralHeader<'a>: Header<'a> {}

pub trait RequestHeader<'a>: Header<'a> {}

pub trait ResponseHeader<'a>: Header<'a> {}

pub trait EntityHeader<'a>: Header<'a> {}

mod accept;
mod content_length;
mod extension_header;
pub use accept::*;
pub use content_length::*;
pub use extension_header::*;

use std::rc::Rc;

#[derive(Debug)]
pub struct Headers {}

impl Debug for dyn Header<'_> {
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
    ) {
        let header_name_buffer = &buffer[header_name_start..header_name_end];
        let header_value_buffer = &buffer[header_value_start..header_end];
        // println!("get_header_struct: start");
        // println!("{:?}", header_name_buffer);
        // println!("{:?}", header_value_buffer);
        // println!("{}", String::from_utf8_lossy(header_name_buffer));
        // println!("{}", String::from_utf8_lossy(header_value_buffer));

        let header: Box<dyn Header> = match header_name_buffer {
            b"accept" => {
                let name = str::from_utf8(header_name_buffer).unwrap();
                let value = str::from_utf8(header_value_buffer).unwrap();
                Box::new(ExtensionHeader::new(name, value))
            }
            b"content-length" => Box::new(
                ContentLength::try_from_vec(
                    buffer,
                    header_value_start,
                    header_end,
                )
                .unwrap(),
            ),
            _ => {
                let name = str::from_utf8(header_name_buffer).unwrap();
                let value = str::from_utf8(header_value_buffer).unwrap();
                Box::new(ExtensionHeader::new(name, value))
            }
        };

        println!("value: {:?}", header.value());
        // println!("get_header_struct: end\n");
    }
}

impl TryFrom<Vec<u8>> for Headers {
    type Error = HttpError;
    fn try_from(mut value: Vec<u8>) -> Result<Self, Self::Error> {
        let length = value.len();
        for i in 0..length {
            value[i] = to_lower_case(value[i]);
        }

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
                    return Err(HttpError::InvalidHeaderFieldToken);
                }
            }

            match byte {
                b'\r' => {
                    let next_byte = value_iter.next();
                    // no more byte after '\r'
                    if next_byte.is_none() {
                        return Err(HttpError::InvalidCrlf);
                    }

                    let (i, next_byte) = next_byte.unwrap();
                    // if there is not \n after \r
                    if *next_byte != b'\n' {
                        return Err(HttpError::InvalidCrlf);
                    }

                    let next_byte = value[i + 1];

                    // There was LWS indicating continuation of the field-value
                    // RFC 2616 https://www.rfc-editor.org/pdfrfc/rfc2616.txt.pdf#page=16
                    if is_space(next_byte) || is_horizontal_tab(next_byte) {
                        continue;
                    }

                    Headers::get_header_struct(
                        &value,
                        header_name_start,
                        header_name_end,
                        header_value_start,
                        i - 1, // i points to \n
                    );
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

        Ok(Headers {})
    }
}
