use crate::errors::Error as HttpError;
use crate::grammar::{is_token, is_token_char};
use crate::headers::Headers;
use crate::helpers::bytes::{Bytes, FragmentedBytes};
use crate::helpers::parser::*;
use crate::request::{Request, RequestBodyBuilder, RequestUri};
use std::collections::LinkedList;
use std::convert::TryFrom;
use std::vec::Vec;
use std::{mem, str};

#[derive(Debug)]
pub enum HttpMethods {
    GET,
    OPTIONS,
    HEAD,
    POST,
    PUT,
    DELETE,
    TRACE,
    CONNECT,
    Extension(String),
}

impl TryFrom<&str> for HttpMethods {
    type Error = HttpError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let bytes = value.as_bytes();
        let is_valid = is_token(bytes);

        if !is_valid {
            return Err(HttpError::InvalidTokenChar(bytes.to_vec()));
        }

        let method_name = value.to_lowercase();

        let method = match &method_name[..] {
            "get" => HttpMethods::GET,
            "options" => HttpMethods::OPTIONS,
            "head" => HttpMethods::HEAD,
            "post" => HttpMethods::POST,
            "put" => HttpMethods::PUT,
            "delete" => HttpMethods::DELETE,
            "trace" => HttpMethods::TRACE,
            "connect" => HttpMethods::CONNECT,
            _ => HttpMethods::Extension(method_name),
        };

        Ok(method)
    }
}

#[derive(Debug)]
pub enum HttpVersion {
    Http11,
}

impl TryFrom<&str> for HttpVersion {
    type Error = HttpError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let version = value.to_lowercase();

        let method_name = value.to_lowercase();

        match &version[..] {
            "http/1.1" => Ok(HttpVersion::Http11),
            _ => Err(HttpError::InvalidHttpVersion(version)),
        }
    }
}

#[derive(Debug)]
pub struct RequestBuilder {
    method: Option<HttpMethods>,
    uri: Option<RequestUri>,
    http_version: Option<HttpVersion>,
    headers: Option<Headers>,
    body: Option<RequestBodyBuilder>,
    fragmented_bytes: FragmentedBytes,
    has_skipped_initial_crlf: bool,
}

impl RequestBuilder {
    pub fn new() -> RequestBuilder {
        RequestBuilder {
            method: None,
            uri: None,
            http_version: None,
            headers: None,
            body: None,
            fragmented_bytes: fragmented_bytes![],
            has_skipped_initial_crlf: false,
        }
    }

    fn get_request_line(&mut self) -> Result<Option<String>, HttpError> {
        let result = look_for_crlf(&mut self.fragmented_bytes);
        if result.is_none() {
            return Ok(None);
        }

        let result = result.unwrap();
        let result = String::from_utf8(result);
        if result.is_err() {
            return Err(HttpError::InvalidRequestLine(
                "not a valid ascii/utf-8 string".into(),
            ));
        }

        Ok(Some(result.unwrap()))
    }

    fn parse_request_line(&mut self) -> Result<&mut Self, HttpError> {
        if self.uri.is_some() {
            return Ok(self);
        }

        let request_line = self.get_request_line()?;
        let request_line = match request_line {
            None => return Ok(self),
            Some(r) => r,
        };

        let mut parts: Vec<&str> = request_line.split(" ").collect();

        if parts.len() != 3 {
            return Err(HttpError::InvalidRequestLine(
                "either request method or request-target or http version is missing".into(),
            ));
        }

        let method = HttpMethods::try_from(parts[0])?;
        self.method = Some(method);

        let version = HttpVersion::try_from(*parts.last().unwrap())?;
        self.http_version = Some(version);

        parts.pop();

        let request_uri = parts[1..].join(" ");
        let request_uri = RequestUri::try_from(request_uri);
        if request_uri.is_err() {
            // TODO invalid request_uri
            return Ok(self);
        }
        self.uri = Some(request_uri.unwrap());

        Ok(self)
    }

    fn get_headers(&mut self) -> Result<Option<String>, HttpError> {
        let result = look_for_double_crlf(&mut self.fragmented_bytes);
        if result.is_none() {
            return Ok(None);
        }

        let headers_vector = result.unwrap();
        let result = String::from_utf8(headers_vector);
        if result.is_err() {
            let error = result.err().unwrap();
            return Err(HttpError::InvalidUtf8String(
                error.as_bytes().to_vec(),
            ));
        }

        Ok(Some(result.unwrap()))
    }

    fn parse_headers(&mut self) -> Result<&mut Self, HttpError> {
        if self.headers.is_some() {
            return Ok(self);
        }

        let headers = match self.get_headers()? {
            None => return Ok(self),
            Some(h) => h,
        };

        let headers = Headers::try_from(headers)?;
        self.headers = Some(headers);
        Ok(self)
    }

    fn create_request_body_builder(&mut self) {
        if !self.are_headers_parsed()
            || !self.can_have_body()
            || self.body.is_some()
        {
            return;
        }

        let x = self.are_headers_valid();

        let fragmented_bytes = mem::take(&mut self.fragmented_bytes);
        self.fragmented_bytes = fragmented_bytes.remaining_bytes();

        let is_chunked = self.is_chunked();
        let body = match is_chunked {
            true => RequestBodyBuilder::new_chunked(),
            false => {
                let headers = self.headers.as_ref().unwrap();
                let content_length = headers.content_length();
                let content_length = content_length.unwrap();
                RequestBodyBuilder::new_whole(content_length.len())
            }
        };
        self.body = Some(body);
    }

    fn are_headers_valid(&self) -> Result<bool, HttpError> {
        let headers = self.headers.as_ref().unwrap();

        let transfer_encoding = headers.transfer_encoding();
        if let Some(transfer_encoding) = transfer_encoding {
            if !transfer_encoding.is_chunked() {
                return Err(HttpError::NoChunkedCoding);
            }

            return Ok(true);
        }

        let content_length = headers.content_length();
        return Ok(true);
    }

    fn is_chunked(&self) -> bool {
        let headers = self.headers.as_ref().unwrap();
        let transfer_encoding = headers.transfer_encoding();

        transfer_encoding.is_some() && transfer_encoding.unwrap().is_chunked()
    }

    fn are_headers_parsed(&self) -> bool {
        self.headers.is_some()
    }

    fn can_have_body(&self) -> bool {
        let headers = self.headers.as_ref().unwrap();

        let transfer_encoding = headers.transfer_encoding();

        if transfer_encoding.is_some() {
            return true;
        }

        let content_length = headers.content_length();
        return content_length.is_some();
    }

    pub(crate) fn parse_body(&mut self) -> &mut RequestBuilder {
        if !self.are_headers_parsed() || !self.can_have_body() {
            return self;
        }

        self.body
            .as_mut()
            .unwrap()
            .parse(&mut self.fragmented_bytes, &self.headers.as_ref().unwrap());

        self
    }

    pub(crate) fn can_parse_more(&self) -> bool {
        // headers are not parsed yet, so we can parse more
        if self.headers.is_none() {
            return true;
        }

        if self.can_have_body() {
            let body = self.body.as_ref();
            return match body {
                None => true,
                Some(b) => !b.is_parsed(),
            };
        }

        return false;
    }

    pub fn parse(
        &mut self,
        vec: Vec<u8>,
        length: usize,
    ) -> Result<&Self, HttpError> {
        let bytes = Bytes::new(vec, length);
        self.fragmented_bytes.push_bytes(bytes);

        if !self.has_skipped_initial_crlf {
            let result = skip_initial_crlf(&mut self.fragmented_bytes)?;
            self.has_skipped_initial_crlf = result;
        }

        if !self.has_skipped_initial_crlf {
            return Ok(self);
        }

        self.parse_request_line()?
            .parse_headers()?
            .create_request_body_builder();

        if self.are_headers_parsed() {
            self.are_headers_valid()?;
            self.create_request_body_builder();
            self.parse_body();
        }

        Ok(self)
    }

    pub(crate) fn build(self) -> Request {
        let RequestBuilder {
            method,
            uri,
            http_version,
            headers,
            body,
            fragmented_bytes: _,
            has_skipped_initial_crlf: _,
        } = self;

        let body = body.map(|f| f.build());

        Request::new(
            method.unwrap(),
            uri.unwrap(),
            http_version.unwrap(),
            headers.unwrap(),
            body,
        )
    }
}

#[cfg(test)]
mod request_tests {
    use super::{HttpMethods, HttpVersion, RequestBuilder};

    #[test]
    fn test_one_pass_parse() {
        let buffer = b"\r\n\r\n\n\nGET /abc HTTP/1.1\r\nAccept: */*\r\nUser-agent: abc\r\n\r\n";
        let mut builder = RequestBuilder::new();
        builder.parse(buffer.to_vec(), buffer.len());

        assert!(builder.method.is_some());
        assert!(builder.uri.is_some());
        assert!(builder.http_version.is_some());
        assert!(builder.headers.is_some());

        assert_match!(
            builder.method,
            Some(HttpMethods::GET),
            "expected method {:?}; found {:?}",
            HttpMethods::GET,
            builder.method
        );

        assert_eq!(builder.uri.unwrap().uri(), &String::from("/abc"));
        assert_match!(
            builder.http_version,
            Some(HttpVersion::Http11),
            "expected version {:?}; found {:?}",
            HttpVersion::Http11,
            builder.http_version
        );
    }

    #[test]
    fn test_many_pass_parse() {
        let buffer = b"GE";
        let mut builder = RequestBuilder::new();
        builder.parse(buffer.to_vec(), buffer.len());

        assert!(builder.method.is_none());
        assert!(builder.uri.is_none());
        assert!(builder.http_version.is_none());

        let buffer = b"T /ab";
        builder.parse(buffer.to_vec(), buffer.len());

        assert!(builder.method.is_none());
        assert!(builder.uri.is_none());
        assert!(builder.http_version.is_none());

        let buffer = b"cd ";
        let empty: Vec<u8> = vec![];
        builder.parse(buffer.to_vec(), buffer.len());

        assert!(builder.method.is_none());
        assert!(builder.uri.is_none());
        assert!(builder.http_version.is_none());

        let buffer = b"HTTP";
        builder.parse(buffer.to_vec(), buffer.len());

        assert!(builder.method.is_none());
        assert!(builder.uri.is_none());
        assert!(builder.http_version.is_none());

        let buffer = b"/1.1\r";
        builder.parse(buffer.to_vec(), buffer.len());
        let buffer = b"\n";
        builder.parse(buffer.to_vec(), buffer.len());

        assert!(builder.method.is_some());
        assert!(builder.uri.is_some());
        assert!(builder.http_version.is_some());

        assert_match!(builder.method, Some(HttpMethods::GET));
        assert_match!(builder.http_version, Some(HttpVersion::Http11));

        let buffer = b"Accept: */*\r";
        builder.parse(buffer.to_vec(), buffer.len());
        let buffer = b"\n";
        builder.parse(buffer.to_vec(), buffer.len());
        let buffer = b"User-agent: abc\r\n";
        builder.parse(buffer.to_vec(), buffer.len());
        let buffer = b"Content-length: 4\r\n";
        builder.parse(buffer.to_vec(), buffer.len());
        let buffer = b"\r";
        builder.parse(buffer.to_vec(), buffer.len());
        let buffer = b"\n";
        builder.parse(buffer.to_vec(), buffer.len());

        assert!(builder.method.is_some());
        assert!(builder.uri.is_some());
        assert!(builder.http_version.is_some());
        assert!(builder.headers.is_some());
    }
}
