use crate::encoding::{
    BEncodeable, BEncodingError, DisplayFormat,
    common::{BTYPE_PRINT_MAX_ITEMS, split_on_delimiter},
};
use std::cmp::min;

#[derive(Clone, PartialEq, Eq)]
pub enum BString {
    TextString(String),
    ByteString(Vec<u8>),
}

impl std::fmt::Display for BString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.out(0))
    }
}

impl std::fmt::Debug for BString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.out(0))
    }
}

impl DisplayFormat for BString {
    fn out(&self, _indent: usize) -> String {
        match self {
            Self::TextString(t) => format!("TextString({})(\"{}\")", t.as_bytes().len(), t),
            Self::ByteString(b) => {
                let items_string = b[..min(b.len(), BTYPE_PRINT_MAX_ITEMS)]
                    .iter()
                    .map(|v| format!("0x{v:x}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("ByteString({})({})", b.len(), items_string)
            }
        }
    }
}

impl BEncodeable for BString {
    fn bencode(&self) -> Vec<u8> {
        let mut output = Vec::new();

        let length = match self {
            Self::TextString(t) => t.as_bytes().len(),
            Self::ByteString(b) => b.len(),
        };

        output.extend(length.to_string().as_bytes());

        output.push(b':');

        output.extend(match self {
            Self::TextString(t) => t.as_bytes(),
            Self::ByteString(b) => b,
        });

        output
    }

    fn bdecode(input: &[u8]) -> Result<(Self, &[u8]), BEncodingError>
    where
        Self: Sized,
    {
        let (string_length_slice, remainder) = split_on_delimiter(input, b':')?;

        let Ok(string_length) = std::str::from_utf8(string_length_slice) else {
            return Err(BEncodingError::CouldNotParseUTF8);
        };

        let Ok(string_length) = string_length.parse::<usize>() else {
            return Err(BEncodingError::ParseIntFailure);
        };

        let mut parsed_chars = Vec::new();
        let mut char_remainder = remainder;

        for (ch, r) in UTF8ByteParser(char_remainder).take(string_length) {
            parsed_chars.push(ch);
            char_remainder = r;
        }

        // Check if successfully parsed all characters as utf-8, return text string
        if parsed_chars.len() == string_length {
            return Ok((
                Self::TextString(parsed_chars.iter().collect()),
                char_remainder,
            ));
        }

        // Else, try to get full length as bytes
        let Some((sl, r)) = remainder.split_at_checked(string_length) else {
            return Err(BEncodingError::OutOfBounds);
        };

        Ok((Self::ByteString(sl.iter().cloned().collect()), r))
    }
}

/// Iterator wrapper for `next_char`.
struct UTF8ByteParser<'a>(&'a [u8]);

impl<'a> Iterator for UTF8ByteParser<'a> {
    type Item = (char, &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        let t = next_char(self.0)?;
        self.0 = t.1;
        Some(t)
    }
}

// I'm weirdly proud of this function
/// Returns the next valid UTF-8 character and the remainder of the slice.
/// "Pops" the next char off the slice.
/// Returns `None` if first 1-4 bytes are invalid UTF-8.
fn next_char(input: &[u8]) -> Option<(char, &[u8])> {
    let b0 = *input.get(0)?;
    let cb0 = b0 as u32;
    let utf8_1_byte = b0 >> 7 == 0x0;

    if utf8_1_byte {
        let codepoint = cb0 & 0b1111111;
        let ch = char::from_u32(codepoint)?;
        return Some((ch, &input[1..]));
    }

    let b1 = *input.get(1)?;
    let cb1 = (b1 & 0b111111) as u32;
    let utf8_2_byte = (b0 >> 5 == 0b110) && (b1 >> 6 == 0b10);

    if utf8_2_byte {
        let codepoint = (cb0 & 0b11111) << 6 | cb1;
        let ch = char::from_u32(codepoint)?;
        return Some((ch, &input[2..]));
    }

    let b2 = *input.get(2)?;
    let cb2 = (b2 & 0b111111) as u32;
    let utf8_3_byte = (b0 >> 4 == 0b1110) && (b1 >> 6 == 0b10) && (b2 >> 6 == 0b10);

    if utf8_3_byte {
        let codepoint = (cb0 & 0b1111) << 12 | cb1 << 6 | cb2;
        let ch = char::from_u32(codepoint)?;
        return Some((ch, &input[3..]));
    }

    let b3 = *input.get(3)?;
    let cb3 = (b3 & 0b111111) as u32;
    let utf8_4_byte =
        (b0 >> 3 == 0b11110) && (b1 >> 6 == 0b10) && (b2 >> 6 == 0b10) && (b3 >> 6 == 0b10);

    if utf8_4_byte {
        let codepoint = (cb0 & 0b111) << 18 | cb1 << 12 | cb2 << 6 | cb3;
        let ch = char::from_u32(codepoint)?;
        return Some((ch, &input[4..]));
    }

    None
}

#[cfg(test)]
mod tests {
    use crate::encoding::{BEncodeable, BString};

    #[test]
    fn string() {
        const STRING_GOOD: [(&str, &str); 4] = [
            ("eggs", "4:eggs"),
            ("bacon", "5:bacon"),
            ("ham", "3:ham"),
            ("coffee", "6:coffee"),
            // Byte example
            // "UTF" that switches to bytes
        ];

        for (plain, encoded) in STRING_GOOD {
            assert_eq!(
                BString::TextString(plain.to_owned()).bencode(),
                encoded.as_bytes()
            );
        }

        for (plain, encoded) in STRING_GOOD {
            assert_eq!(
                BString::bdecode(&encoded.as_bytes().to_owned()).unwrap().0,
                BString::TextString(plain.to_owned())
            )
        }

        let mut test: Vec<u8>;

        test = vec![
            '2' as u8, '0' as u8, ':' as u8, 0xb8, 0x9e, 0xaa, 0xc7, 0xe6, 0x14, 0x17, 0x34, 0x1b,
            0x71, 0x0b, 0x72, 0x77, 0x68, 0x29, 0x4d, 0x0e, 0x6a, 0x27, 0x7b,
        ];
        assert_eq!(
            BString::ByteString(test[3..].to_owned()),
            BString::bdecode(&test).unwrap().0
        );

        test = vec![
            '6' as u8, ':' as u8, 'a' as u8, 'b' as u8, 0xb8, 0x9e, 0xaa, 0xc7,
        ];
        assert_eq!(
            BString::ByteString(test[2..].to_owned()),
            BString::bdecode(&test).unwrap().0
        );

        test = vec![
            '6' as u8, ':' as u8, 0xb8, 0x9e, 0xaa, 0xc7, 'a' as u8, 'b' as u8,
        ];
        assert_eq!(
            BString::ByteString(test[2..].to_owned()),
            BString::bdecode(&test).unwrap().0
        );
    }

    #[test]
    fn btype_string_bad() {
        const STRINGS: [&str; 4] = ["3eggs", "6:bacon", "3ham", ":coffee"];

        for encoded in STRINGS {
            assert!(BString::bdecode(&encoded.as_bytes().to_owned()).is_err());
        }
    }
}
