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

#[cfg(test)]
mod tests {
    use super::TOKEN_CHAR;

    #[test]
    fn test() {
        for i in 65..91 {
            assert_eq!(TOKEN_CHAR[i], true);
        }

        for i in 97..123 {
            assert_eq!(TOKEN_CHAR[i], true);
        }
    }
}
