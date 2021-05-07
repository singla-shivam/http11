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
        [
            $(
                $(#[$docs1:meta])*
                ($name1:ident, $phrase1:expr);
            )+
        ],
        [
            $(
                $(#[$docs2:meta])*
                ($name2:ident, $type:ty, $phrase2:expr);
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
                $name2($type),
            )+
        }

        impl Error {
            fn desc(&self) -> String {
                match &*self {
                    $(
                        Error::$name1 => String::from($phrase1),
                    )+
                    $(
                        Error::$name2(val) => format!("{}: {:?}", $phrase2, val),
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
        (InvalidHttpVersion, "Invalid http version");
        (RequestNotParsed, "Trying to get request before it is not parsed completely");
        (InvalidCrlf, "Invalid character after \\r.");
        (InvalidUtf8String, "Invalid utf-8 encoding");
        (InvalidContentLengthValue, "Content length field contains non digit characters");
        (NoChunkedCoding, "There was transfer-encoding but the last encoding was not chunked");
    ],
    [
        (InvalidHeaderFormat, String, "Invalid header format");
        (InvalidHeaderFieldToken, String, "Header field contains invalid token character");
    ]
}
