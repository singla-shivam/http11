use crate::helpers::bytes::FragmentedBytes;

pub(crate) fn look_for_delimiter(
    bytes: &mut FragmentedBytes,
    delimiter: &Vec<u8>,
) -> Option<Vec<u8>> {
    let delimiter_len = delimiter.len();
    let mut outstanding_buffer = Vec::with_capacity(delimiter_len);

    let mut bytes_iter = bytes.iter();
    let mut delimiter_end = 0;

    while let Some(byte) = bytes_iter.next() {
        push_to_buffer(&mut outstanding_buffer, byte);

        if delimiter == &outstanding_buffer {
            delimiter_end = bytes_iter.current_pos() - 1;
            break;
        }
    }

    if delimiter_end != 0 {
        let buffer = bytes.copy_buffer(delimiter_end - delimiter_len);
        bytes.set_read_pos(delimiter_end + 1);
        return Some(buffer);
    }

    None
}

pub(crate) fn look_for_crlf(bytes: &mut FragmentedBytes) -> Option<Vec<u8>> {
    return look_for_delimiter(bytes, &b"\r\n".to_vec());
}

fn push_to_buffer(buf: &mut Vec<u8>, byte: u8) {
    let len = buf.len();

    if len != buf.capacity() {
        buf.push(byte);
        return;
    }

    for i in 1..len {
        buf[i - 1] = buf[i]
    }

    buf[len - 1] = byte;
}

#[cfg(test)]
mod tests_parser {
    use super::{look_for_crlf, look_for_delimiter};
    use crate::helpers::bytes::{Bytes, FragmentedBytes};

    #[test]
    pub fn test_look_for_crlf_one_buffer() {
        let bytes = Bytes::new(vec![1, 2, 3, b'\r', b'\n'], 5);
        let mut bytes = fragmented_bytes![bytes];

        let buffer = look_for_crlf(&mut bytes);
        assert!(buffer.is_some());
        assert_eq!(vec![1, 2, 3], buffer.unwrap());
    }

    #[test]
    pub fn test_look_for_crlf_multiple_buffer() {
        let bytes1 = Bytes::new(vec![1, 2, 3], 3);
        let bytes2 = Bytes::new(vec![1, 2, 3, 4, 5], 5);
        let bytes3 =
            Bytes::new(vec![1, 2, 3, 6, 7, 8, b'\r', b'\n', 9, 10], 10);

        let mut bytes = fragmented_bytes![bytes1, bytes2, bytes3];

        let buffer = look_for_crlf(&mut bytes);
        assert!(buffer.is_some());
        assert_eq!(
            vec![1, 2, 3, 1, 2, 3, 4, 5, 1, 2, 3, 6, 7, 8],
            buffer.unwrap()
        );
    }

    #[test]
    pub fn test_look_for_crlf_with_no_crlf() {
        let bytes1 = Bytes::new(vec![1, 2, 3], 3);
        let bytes2 = Bytes::new(vec![1, 2, 3, 4, 5], 5);

        let mut bytes = fragmented_bytes![bytes1, bytes2];

        let buffer = look_for_crlf(&mut bytes);
        assert!(buffer.is_none());

        let bytes3 = Bytes::new(vec![1, 2, 3, 6, 7, 8, b'\r', 9, 10], 9);
        bytes.push_bytes(bytes3);

        let buffer = look_for_crlf(&mut bytes);
        assert!(buffer.is_none());

        let bytes4 =
            Bytes::new(vec![1, 2, 3, 6, 7, 8, b'\r', b'\n', 9, 10], 10);
        bytes.push_bytes(bytes4);

        let buffer = look_for_crlf(&mut bytes);
        assert!(buffer.is_some());
        assert_eq!(
            vec![
                1, 2, 3, 1, 2, 3, 4, 5, 1, 2, 3, 6, 7, 8, b'\r', 9, 10, 1, 2,
                3, 6, 7, 8
            ],
            buffer.unwrap()
        );
    }

    #[test]
    fn test_look_for_other_delimiter_sequentially() {
        let delimiter = vec![4, 5];
        let bytes1 = Bytes::new(vec![1, 2, 3, 4, 5], 5);
        let bytes2 = Bytes::new(vec![11, 12, 13, 4, 5], 5);
        let bytes3 = Bytes::new(vec![21, 22, 23, 4, 5], 5);

        let mut bytes = fragmented_bytes![bytes1, bytes2, bytes3];

        let buffer = look_for_delimiter(&mut bytes, &delimiter);
        assert!(buffer.is_some());
        assert_eq!(vec![1, 2, 3], buffer.unwrap());

        let buffer = look_for_delimiter(&mut bytes, &delimiter);
        assert!(buffer.is_some());
        assert_eq!(vec![11, 12, 13], buffer.unwrap());

        let buffer = look_for_delimiter(&mut bytes, &delimiter);
        assert!(buffer.is_some());
        assert_eq!(vec![21, 22, 23], buffer.unwrap());
    }
}
