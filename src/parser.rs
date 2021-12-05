use nom::{
    bytes::complete::take,
    character::complete::{crlf, not_line_ending},
    multi::count,
    sequence::terminated,
    IResult,
};


/// Enum for types defined in RESP specification.
/// Its variants contain Vec<u8> or Option<Vec<u8>> for optional types (i.e. Bulk Strings and Arrays).
pub enum Resp {
    /// Simple string in RESP.
    ///
    /// # Examples
    /// ```
    /// let simple_string = respirator::resp(&b"+OK\r\n"[..]);
    /// if let (_, respirator::Resp::SimpleString(value)) = simple_string.unwrap() {
    ///   assert_eq!(value, b"OK".to_vec());
    /// }
    /// ```
    SimpleString(Vec<u8>),
    /// Integer in RESP. Contains i64 value.
    ///
    /// # Examples
    /// ```
    /// let integer = respirator::resp(&b":8\r\n"[..]);
    /// if let (_, respirator::Resp::Integer(value)) = integer.unwrap() {
    ///   assert_eq!(value, 8);
    /// }
    /// ```
    Integer(i64),
    /// Error in RESP.
    ///
    /// # Examples
    /// ```
    /// let error = respirator::resp(&b"-ERROR\r\n"[..]);
    /// if let (_, respirator::Resp::Error(value)) = error.unwrap() {
    ///   assert_eq!(value, b"ERROR".to_vec());
    /// }
    /// ```
    Error(Vec<u8>),
    /// Bulk String in RESP, contains None if encounters empty string.
    ///
    /// # Examples
    /// ```
    /// /// Bulk String
    /// let bulk_string = respirator::resp(&b"$3\r\nstr\r\n"[..]);
    /// if let (_, respirator::Resp::BulkString(Some(value))) = bulk_string.unwrap() {
    ///   assert_eq!(value, b"str".to_vec());
    /// }
    ///
    /// use std::matches;
    /// /// Empty Bulk String
    /// let empty_bulk_string = respirator::resp(&b"$0\r\n"[..]);
    /// assert!(matches!(respirator::Resp::BulkString(None), empty_bulk_string));
    /// ```
    BulkString(Option<Vec<u8>>),
    /// Array in RESP, contains None if encounters empty array.
    ///
    /// # Examples
    /// ```
    /// /// Bulk String
    /// let bulk_string = respirator::resp(&b"$3\r\nstr\r\n"[..]);
    /// if let (_, respirator::Resp::BulkString(Some(value))) = bulk_string.unwrap() {
    ///   assert_eq!(value, b"str".to_vec());
    /// }
    ///
    /// use std::matches;
    /// /// Empty Bulk String
    /// let empty_bulk_string = respirator::resp(&b"$0\r\n"[..]);
    /// assert!(matches!(respirator::Resp::BulkString(None), empty_bulk_string));
    /// ```
    Array(Option<Vec<Resp>>),
}

/// Main function for RESP parsing, conforming nom's contract.
///
/// # Arguments
///
/// * `input` - a byte slice to be parsed
///
/// # Examples
/// ```
/// use respirator::{Resp, resp};
/// use std::matches;
///
/// let input = &b"*2\r\n$2\r\nOK\r\n$4\r\nResp\r\n"[..];
/// let (_, parsed) = resp(input).unwrap();
/// if let Resp::Array(Some(values)) = parsed {
///   let simple_string = &values[0];
///   let bulk_string = &values[1];
///   assert!(matches!(Resp::SimpleString(b"OK".to_vec()), simple_string));
///   assert!(matches!(Resp::BulkString(Some(b"Resp".to_vec())), bulk_string));
/// }
/// ```
pub fn resp(input: &[u8]) -> IResult<&[u8], Resp> {
    let (input, val) = take(1usize)(input)?;
    match val[0] {
        b'+' => simple_string(input),
        b':' => integer(input),
        b'-' => error(input),
        b'$' => bulk_string(input),
        b'*' => array(input),
        _ => panic!("Unknown type byte: {:?}", val),
    }
}

fn simple_string(input: &[u8]) -> IResult<&[u8], Resp> {
    let (input, val) = terminated(not_line_ending, crlf)(input)?;
    Ok((input, Resp::SimpleString(val.to_vec())))
}

fn integer(input: &[u8]) -> IResult<&[u8], Resp> {
    let (input, val) = terminated(not_line_ending, crlf)(input)?;
    Ok((
        input,
        Resp::Integer(String::from_utf8_lossy(val).parse::<i64>().unwrap()),
    ))
}

fn error(input: &[u8]) -> IResult<&[u8], Resp> {
    let (input, val) = terminated(not_line_ending, crlf)(input)?;
    Ok((input, Resp::Error(val.to_vec())))
}

fn bulk_string(input: &[u8]) -> IResult<&[u8], Resp> {
    let (input, len) = length(input)?;
    if len == 0 {
        return Ok((input, Resp::BulkString(None)));
    }
    let (input, val) = terminated(take(len), crlf)(input)?;

    Ok((input, Resp::BulkString(Some(val.to_vec()))))
}

fn length(input: &[u8]) -> IResult<&[u8], usize> {
    let (input, len) = terminated(not_line_ending, crlf)(input)?;
    Ok((input, String::from_utf8_lossy(len).parse().unwrap()))
}

fn array(input: &[u8]) -> IResult<&[u8], Resp> {
    let (input, len) = length(input)?;
    if len == 0 {
        return Ok((input, Resp::Array(None)));
    }
    let (input, res) = count(resp, len)(input)?;
    Ok((input, Resp::Array(Some(res))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_strings() {
        let input = &b"+OK - seems good.\r\n"[..];
        let parsed = resp(input).unwrap();
        if let Resp::SimpleString(parsed) = parsed.1 {
            assert_eq!(parsed, b"OK - seems good.".to_vec());
        } else {
            panic!("Error parsing SimpleString");
        }
    }

    #[test]
    #[should_panic]
    fn fails_on_corrupted_simple_string() {
        let corrupted_input = &b"+OK - seems bad.\r"[..];
        resp(corrupted_input).unwrap();
    }

    #[test]
    fn parses_integers() {
        let input = &b":12345\r\n"[..];
        let parsed = resp(input).unwrap();
        if let Resp::Integer(parsed) = parsed.1 {
            assert_eq!(parsed, 12345);
        } else {
            panic!("Error parsing Integer");
        }
    }

    #[test]
    #[should_panic]
    fn fails_on_corrupted_integer() {
        let corrupted_input = &b":OK - seems bad.\r\n"[..];
        resp(corrupted_input).unwrap();
    }

    #[test]
    fn parses_errors() {
        let input = &b"-this is an error\r\n"[..];
        let parsed = resp(input).unwrap();
        if let Resp::Error(parsed) = parsed.1 {
            assert_eq!(parsed, b"this is an error".to_vec());
        } else {
            panic!("Error parsing Error");
        }
    }

    #[test]
    #[should_panic]
    fn fails_on_corrupted_error() {
        let corrupted_input = &b"-an error - seems bad.\n"[..];
        resp(corrupted_input).unwrap();
    }

    #[test]
    fn parses_bulk_string() {
        let input = &b"$4\r\ngood\r\n"[..];
        let parsed = resp(input).unwrap();
        if let Resp::BulkString(Some(parsed)) = parsed.1 {
            assert_eq!(parsed, b"good".to_vec());
        } else {
            panic!("Error parsing BulkString");
        }
    }

    #[test]
    #[should_panic]
    fn fails_on_corrupted_bulk_string() {
        let corrupted_input = &b"$4\r\nbad\r\n"[..];
        resp(corrupted_input).unwrap();
    }

    #[test]
    fn parses_array() {
        let input = &b"*2\r\n$2\r\nOK\r\n$4\r\nResp\r\n"[..];
        let parsed = resp(input).unwrap();

        if let Resp::Array(Some(parsed)) = parsed.1 {
            if let [Resp::BulkString(Some(str1)), Resp::BulkString(Some(str2))] = &parsed[..] {
                assert_eq!(*str1, b"OK".to_vec());
                assert_eq!(*str2, b"Resp".to_vec());
            } else {
                panic!("Error parsing Array");
            }
        } else {
            panic!("Error parsing Array");
        }
    }

    #[test]
    fn parses_empty_array() {
        let input = &b"*0\r\n"[..];
        let parsed = resp(input).unwrap();

        if let Resp::Array(None) = parsed.1 {
        } else {
            panic!("Error parsing Array");
        }
    }

    #[test]
    fn parses_multiple() {
        let input = &b"$4\r\ngood\r\n:8\r\n+OK\r\n"[..];

        let (input, parsed) = resp(input).unwrap();
        if let Resp::BulkString(Some(parsed)) = parsed {
            assert_eq!(parsed, b"good".to_vec());
        } else {
            panic!("Error parsing BulkString");
        }

        let (input, parsed) = resp(input).unwrap();
        if let Resp::Integer(parsed) = parsed {
            assert_eq!(parsed, 8);
        } else {
            panic!("Error parsing Integer");
        }

        let (input, parsed) = resp(input).unwrap();
        if let Resp::SimpleString(parsed) = parsed {
            assert_eq!(parsed, b"OK".to_vec());
        } else {
            panic!("Error parsing SimpleString");
        }

        assert_eq!(input, &[]);
    }
}
