fn byte_to_bool(bytes: [u8; 256]) -> [bool; 256] {
    let mut result = [false; 256];

    for i in 0..256 {
        result[i] = bytes[i] != 0;
    }

    result
}

lazy_static! {
    /// CHAR = <any US-ASCII character (octets 0 - 127)>
    static ref CHAR: [bool; 256] = {
        let mut bytes = [false; 256];

        for i in 0..128 {
            bytes[i] = true;
        }

        bytes
    };

    /// CTL = <any US-ASCII control character
    ///         (octets 0 - 31) and DEL (127)>
    static ref CTL: [bool; 256] = {
        let mut bytes = [false; 256];

        for i in 0..32 {
            bytes[i] = true;
        }

        bytes[127] = true;
        bytes
    };

    /// separators = "(" | ")" | "<" | ">" | "@"
    ///               | "," | ";" | ":" | "\" | <">
    ///               | "/" | "[" | "]" | "?" | "="
    ///               | "{" | "}" | SP | HT
    static ref SEPARATOR: [bool; 256] = {
        let mut result = [false; 256];
        let separators = "()<>@,;:\\\"/[]?={} \t";

        for s in separators.chars() {
            result[s as usize] = true;
        }

        result
    };

    /// token = 1*<any CHAR except CTLs or separators>
    static ref TOKEN_CHAR: [bool; 256] = {
        let mut result = [false; 256];

        for i in 0..256 {
            result[i] = CHAR[i] && !(CTL[i] || SEPARATOR[i]);
        }

        result
    };

    /// token = 1*<any CHAR except CTLs or separators>
    static ref UPPER_ALPHA: [bool; 256] = {
        let mut result = [false; 256];

        for i in 65..91 {
            result[i] = true;
        }

        result
    };
}

#[inline]
pub fn is_token_char(byte: u8) -> bool {
    TOKEN_CHAR[byte as usize]
}

pub fn is_token(bytes: &[u8]) -> bool {
    for byte in bytes {
        if !is_token_char(*byte) {
            return false;
        }
    }

    true
}

#[inline]
pub fn is_cr(byte: u8) -> bool {
    byte == 13
}

#[inline]
pub fn is_lf(byte: u8) -> bool {
    byte == 10
}

#[inline]
pub fn is_space(byte: u8) -> bool {
    byte == 32
}

#[inline]
pub fn is_horizontal_tab(byte: u8) -> bool {
    byte == 32
}

#[inline]
pub fn is_upper_alpha(byte: u8) -> bool {
    UPPER_ALPHA[byte as usize]
}

#[inline]
pub fn to_lower_case(byte: u8) -> u8 {
    match is_upper_alpha(byte) {
        true => byte | 0x20,
        false => byte,
    }
}

#[cfg(test)]
mod tests {
    use super::TOKEN_CHAR;

    #[test]
    fn test_token_char() {
        for i in 65..91 {
            assert_eq!(TOKEN_CHAR[i], true);
        }

        for i in 97..123 {
            assert_eq!(TOKEN_CHAR[i], true);
        }
    }
}
