use std::num::NonZeroU16;

#[derive(Clone, Copy, Debug)]
pub struct StatusCode(NonZeroU16);

pub struct InvalidStatusCode();

impl StatusCode {
    fn from_u16(code: u16) -> Result<StatusCode, InvalidStatusCode> {
        if code < 100 || code >= 1000 {
            return Err(InvalidStatusCode());
        }

        NonZeroU16::new(code)
            .map(StatusCode)
            .ok_or_else(InvalidStatusCode)
    }
}

macro_rules! status_code {
    (
        $(
            ($code:expr, $name:ident, $phrase:expr);
        )+
    ) => {
        impl StatusCode {
            $(
                pub const $name: StatusCode = StatusCode(
                    unsafe { NonZeroU16::new_unchecked($code) }
                );
            )+

            pub fn reason(num: u16) -> Option<&'static str> {
                match num {
                    $(
                        $code => Some($phrase),
                    )+
                    _ => None
                }
            }
        }
    };
}

status_code! {
    (100, CONTINUE, "Continue");
    (200, OK, "OK");
    (400, BAD_REQUEST, "Bad Request");
    (404, NOT_FOUND, "Not Found");
    (500, INTERNAL_SERVER_ERROR, "Internal Server Error");
}
