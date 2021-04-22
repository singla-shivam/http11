use std::num::NonZeroU16;

#[derive(Clone, Copy, Debug)]
pub struct StatusCode(NonZeroU16);

macro_rules! status_code {
    (
        $(
            ($code:expr, $name:ident, $phrase:expr);
        )+
    ) => {
        impl StatusCode {
            $(
                #[allow(dead_code)]
                pub const $name: StatusCode = StatusCode(
                    unsafe { NonZeroU16::new_unchecked($code) }
                );
            )+

            #[allow(dead_code)]
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
}
