use std::convert::From;
use std::{fmt, io, str};

impl From<Error> for io::Error {
    fn from(f: Error) -> Self {
        io::Error::new(io::ErrorKind::InvalidData, f)
    }
}

macro_rules! errors {
    (
        [
            $(
                $(#[$docs1:meta])*
                ($name1:ident, $phrase1:expr);
            )+
        ],
        [
            $(
                $(#[$docs2:meta])*
                ($name2:ident, $phrase2:expr);
            )+
        ],
        [
            $(
                $(#[$docs_n:meta])*
                ($name_n:ident, $type_n:ty, $phrase_n:expr);
            )+
        ]
    ) => {
        #[derive(Clone, Debug)]
        pub enum Error {
            $(
                $(#[$docs1])*
                $name1,
            )+
            $(
                $(#[$docs2])*
                $name2(String),
            )+
            $(
                $(#[$docs_n])*
                $name_n($type_n),
            )+
        }

        impl Error {
            fn desc(&self) -> String {
                match &*self {
                    $(
                        Error::$name1 => String::from($phrase1),
                    )+
                    $(
                        Error::$name2(val) => format!("{}: {}", $phrase2, val),
                    )+
                    $(
                        Error::$name_n(val) => format!("{}: {:?}", $phrase_n, val),
                    )+
                }
            }
        }

        impl fmt::Display for Error {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str(&self.desc())
            }
        }

        impl std::error::Error for Error {}
    };
}

errors! {
    [
        (NewLine, "Invalid byte in new line");
        (Token, "Invalid token character");
        (InvalidUri, "Invalid token in Uri");
        (RequestNotParsed, "Trying to get request before it is not parsed completely");
        (InvalidContentLengthValue, "Content length field contains non digit characters");
        (NoChunkedCoding, "There was transfer-encoding but the last encoding was not chunked");
    ],
    [
        (InvalidHttpVersion, "Invalid http version");
        (InvalidRequestLine, "Invalid request line");
        (InvalidCrlf, "Invalid character after \\r.");
        (InvalidHeaderFormat, "Invalid header format");
        (InvalidHeaderField, "Invalid header field");
        (InvalidHeaderFieldValue, "Header field-value contains invalid token character");
        (ParseIntError, "Parse Int Error");
    ],
    [
        (InvalidUtf8String, Vec<u8>, "Invalid utf-8 encoding");
        (InvalidTokenChar, Vec<u8>, "Invalid token character");
    ]
}
