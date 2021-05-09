#[macro_export]
macro_rules! assert_match {
    ($expression:expr, $( $pattern:pat )|+ $( if $guard: expr )? $(,)?) => {
        assert!(match $expression {
            $( $pattern )|+ $( if $guard )? => true,
            _ => false
        })
    };
    ($expression:expr, $( $pattern:pat )|+ $( if $guard: expr )? $(,)?, $($arg:tt)*) => {
        assert!(match $expression {
            $( $pattern )|+ $( if $guard )? => true,
            _ => false
        }, $($arg)*)
    }
}

#[macro_export]
macro_rules! assert_match_error {
    ($expression1:expr, $expression2:expr) => {
        assert_eq!($expression1.to_string(), $expression2.to_string());
    };
    ($expression1:expr, $expression2:expr, $($arg:tt)+) => {
        assert_eq!($expression1.to_string(), $expression2.to_string(), $($arg)+);
    };
}
