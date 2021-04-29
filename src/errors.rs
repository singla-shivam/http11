use std::convert::From;
use std::{fmt, io, str};

impl From<Error> for io::Error {
    fn from(f: Error) -> Self {
        io::Error::new(io::ErrorKind::InvalidData, f)
    }
}

impl From<str::Utf8Error> for Error {
    fn from(f: str::Utf8Error) -> Self {
        Error::InvalidUtf8String
    }
}

macro_rules! errors {
    (
        $(
            $(#[$docs:meta])*
            ($name:ident, $phrase:expr);
        )+
    ) => {
        #[derive(Copy, Clone, Debug)]
        pub enum Error {
            $(
                $(#[$docs])*
                $name,
            )+
        }

        impl Error {
            fn desc(&self) -> &'static str {
                match *self {
                    $(
                        Error::$name => $phrase,
                    )+
                }
            }
        }

        impl fmt::Display for Error {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str(self.desc())
            }
        }

        impl std::error::Error for Error {
            fn description(&self) -> &str {
                self.desc()
            }
        }
    };
}

errors! {
    (NewLine, "Invalid byte in new line");
    (Token, "Invalid token character");
    (InvalidUri, "Invalid token in Uri");
    (InvalidHttpVersion, "Invalid http version");
    (RequestNotParsed, "Trying to get request before it is not parsed completely");
    (InvalidHeaderFieldToken, "Header field contains invalid token character");
    (InvalidCrlf, "Invalid character after \\r.");
    (InvalidUtf8String, "Invalid utf-8 encoding");
    (InvalidContentLengthValue, "Content length field contains non digit characters");
}
