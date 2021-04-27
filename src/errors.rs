use std::convert::From;
use std::{fmt, io};

impl From<Error> for io::Error {
    fn from(f: Error) -> Self {
        io::Error::new(io::ErrorKind::InvalidData, f)
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
}
