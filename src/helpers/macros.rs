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
