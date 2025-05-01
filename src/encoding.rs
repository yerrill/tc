use std::{cmp::min, collections::BTreeMap, str::from_utf8};

const BTYPE_PRINT_MAX_ITEMS: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BencodingError {
    // ADD FROM OPTION<CHAR>
    CharacterNotFound(char),
    ParseIntFailure,
    MalformedString(String),
    IncorrectStartingCharacter(char),
    Nested(String),
    InvalidType(char),
    InvalidBType(BTypes),
    OutOfBounds,
    MissingInputType(Vec<u8>),
    CouldNotParseUTF8,
}

impl std::fmt::Display for BencodingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BencodingError::CharacterNotFound(c) => {
                write!(f, "BencodingError::CharacterNotFound {c}")
            }
            BencodingError::ParseIntFailure => write!(f, "BencodingError::ParseIntFailure"),
            BencodingError::MalformedString(s) => write!(f, "BencodingError::MalformedString {s}"),
            BencodingError::IncorrectStartingCharacter(c) => {
                write!(f, "BencodingError::IncorrectStartingChar {c}")
            }
            BencodingError::Nested(s) => write!(f, "BencodingError::Nested {s}"),
            BencodingError::InvalidType(c) => write!(f, "BencodingError::InvalidType {c}"),
            BencodingError::OutOfBounds => write!(f, "BencodingError::OutOfBounds"),
            BencodingError::InvalidBType(b) => write!(f, "BencodingError::InvalidBType {b:?}"),
            BencodingError::MissingInputType(v) => {
                write!(f, "Decoding input had no type character {v:?}")
            }
            BencodingError::CouldNotParseUTF8 => write!(f, "Could not parse UTF8 from input"),
        }
    }
}

impl std::error::Error for BencodingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub type DictInner = BTreeMap<String, BTypes>;

#[derive(Clone, PartialEq, Eq)]
pub enum BTypes {
    Integer(isize),
    TextString(String),
    ByteString(Vec<u8>),
    List(Vec<BTypes>),
    Dict(DictInner),
}

impl std::fmt::Display for BTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string(0))
    }
}

impl std::fmt::Debug for BTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string(0))
    }
}

impl BTypes {
    pub fn bencode(&self) -> Vec<u8> {
        let mut output: Vec<u8> = Vec::new();

        match self {
            BTypes::Integer(int) => {
                output.push('i' as u8);
                output.extend(int.to_string().as_bytes());
                output.push('e' as u8);
            }
            BTypes::TextString(st) => {
                output.extend(st.len().to_string().as_bytes());
                output.push(':' as u8);
                output.extend(st.as_bytes());
            }
            BTypes::ByteString(st) => {
                output.extend(st.len().to_string().as_bytes());
                output.push(':' as u8);
                output.extend(st);
            }
            BTypes::List(items) => {
                output.push('l' as u8);

                for item in items {
                    output.extend(item.bencode());
                }

                output.push('e' as u8);
            }
            BTypes::Dict(btree_map) => {
                output.push('d' as u8);

                for (key, value) in btree_map {
                    output.extend(BTypes::TextString(key.to_owned()).bencode());
                    output.extend(&value.bencode());
                }

                output.push('e' as u8);
            }
        };

        output
    }

    pub fn bdecode(input: &Vec<u8>) -> Result<Self, BencodingError> {
        match bdecode(&input.as_slice()) {
            Ok((v, _)) => Ok(v),
            Err(e) => Err(e),
        }
    }

    pub fn to_string(&self, indent: usize) -> String {
        let mut output: String = String::new();

        match self {
            BTypes::Integer(i) => {
                output = format!("{output}Integer({i})");
            }
            BTypes::TextString(s) => {
                output = format!("{output}TextString({})(\"{}\")", s.len(), s);
            }
            BTypes::ByteString(items) => {
                let items_string = items[..min(items.len(), BTYPE_PRINT_MAX_ITEMS)]
                    .iter()
                    .map(|v| format!("0x{v:x}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                output = format!("{output}ByteString({})({})", items.len(), items_string);
            }
            BTypes::List(items) => {
                let items_string = items[..min(items.len(), BTYPE_PRINT_MAX_ITEMS)]
                    .iter()
                    .map(|v| v.to_string(indent))
                    .collect::<Vec<_>>()
                    .join(", ");
                output = format!("{output}List[{items_string}]");
            }
            BTypes::Dict(btree_map) => {
                let items_string = btree_map
                    .iter()
                    .map(|(k, v)| {
                        format!(
                            "{}\"{}\": {}",
                            repeat(' ', indent + 2),
                            k,
                            v.to_string(indent + 2)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                output = format!(
                    "{output}Dict({})(\n{}\n{})",
                    btree_map.len(),
                    items_string,
                    repeat(' ', indent)
                );
            }
        };

        output
    }
}

fn repeat(ch: char, count: usize) -> String {
    (0..count).map(|_| ch).collect()
}

fn bdecode(text: &[u8]) -> Result<(BTypes, &[u8]), BencodingError> {
    let Some((type_char, _)) = text.split_first() else {
        return Err(BencodingError::MissingInputType(text.to_owned()));
    };

    match *type_char as char {
        'i' => parse_integer(text),
        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => parse_string(text),
        'l' => parse_list(text),
        'd' => parse_dictionary(text),
        _ => Err(BencodingError::InvalidType(*type_char as char)),
    }
}

fn parse_integer(text: &[u8]) -> Result<(BTypes, &[u8]), BencodingError> {
    let Some((_, remainder)) = text.split_first() else {
        return Err(BencodingError::MissingInputType(text.to_owned()));
    };

    let (slice_numbers, remainder) = split_on_delimiter(remainder, 'e' as u8)?;

    let Ok(number_string) = from_utf8(slice_numbers) else {
        return Err(BencodingError::CouldNotParseUTF8);
    };

    let number_value = {
        if let Ok(val) = number_string.parse::<isize>() {
            val
        } else {
            return Err(BencodingError::ParseIntFailure);
        }
    };

    // IMPROVEMENT: Better way to check for bad integer encoding

    if number_string.len() != number_value.to_string().len() {
        return Err(BencodingError::ParseIntFailure);
    }

    Ok((BTypes::Integer(number_value), remainder))
}

fn parse_string(text: &[u8]) -> Result<(BTypes, &[u8]), BencodingError> {
    let (string_length_slice, remainder) = split_on_delimiter(text, ':' as u8)?;

    let Ok(string_length) = from_utf8(string_length_slice) else {
        return Err(BencodingError::CouldNotParseUTF8);
    };

    let Ok(string_length) = string_length.parse::<usize>() else {
        return Err(BencodingError::ParseIntFailure);
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
            BTypes::TextString(parsed_chars.iter().collect()),
            char_remainder,
        ));
    }

    // Else, try to get full lenght as bytes
    let Some((sl, r)) = remainder.split_at_checked(string_length) else {
        return Err(BencodingError::OutOfBounds);
    };

    Ok((BTypes::ByteString(sl.iter().cloned().collect()), r))
}

fn parse_list(text: &[u8]) -> Result<(BTypes, &[u8]), BencodingError> {
    let Some((_, remainder)) = text.split_first() else {
        return Err(BencodingError::MissingInputType(text.to_owned()));
    };

    let mut values: Vec<BTypes> = Vec::new();
    let mut remainder = remainder;

    loop {
        if check_leader(remainder, 'e' as u8)? {
            remainder = remainder.get(1..).unwrap_or(&[]);
            break;
        }

        let (value, new_remainder) = bdecode(remainder)?;
        values.push(value);
        remainder = new_remainder;
    }

    Ok((BTypes::List(values), remainder))
}

fn parse_dictionary(text: &[u8]) -> Result<(BTypes, &[u8]), BencodingError> {
    let Some((_, remainder)) = text.split_first() else {
        return Err(BencodingError::MissingInputType(text.to_owned()));
    };

    let mut map: BTreeMap<String, BTypes> = BTreeMap::new();
    let mut remainder = remainder;

    loop {
        if check_leader(remainder, 'e' as u8)? {
            remainder = remainder.get(1..).unwrap_or(&[]);
            break;
        }

        let (key, key_remainder) = bdecode(remainder)?;
        let (value, value_remainder) = bdecode(key_remainder)?;

        remainder = value_remainder;

        if let BTypes::TextString(k) = key {
            map.insert(k, value);
        } else {
            return Err(BencodingError::InvalidBType(key));
        }
    }

    Ok((BTypes::Dict(map), remainder))
}

/// Seeks forward in slice to find first byte matching target.
/// Splits slice from `[0, mid)` and `(mid, len)`
fn split_on_delimiter(input: &[u8], target_char: u8) -> Result<(&[u8], &[u8]), BencodingError> {
    // IMPROVEMENT: Replace with Slice::split_once when out of nightly
    for (index, ch) in input.iter().enumerate() {
        if target_char == *ch {
            let (left, right) = input.split_at(index);

            let right = right.get(1..).unwrap_or(&[]);

            return Ok((left, right));
        }
    }

    Err(BencodingError::CharacterNotFound(target_char as char))
}

fn check_leader(input: &[u8], leader: u8) -> Result<bool, BencodingError> {
    let Some(first_char) = input.get(0) else {
        return Err(BencodingError::OutOfBounds);
    };

    Ok(*first_char == leader)
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
    use super::BTypes::{ByteString, Dict, Integer, List, TextString};
    use super::*;

    #[test]
    fn btype_string() {
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
                BTypes::TextString(plain.to_owned()).bencode(),
                encoded.as_bytes()
            );
        }

        for (plain, encoded) in STRING_GOOD {
            assert_eq!(
                BTypes::bdecode(&encoded.as_bytes().to_owned()),
                Ok(BTypes::TextString(plain.to_owned()))
            )
        }

        let mut test: Vec<u8>;

        test = vec![
            '2' as u8, '0' as u8, ':' as u8, 0xb8, 0x9e, 0xaa, 0xc7, 0xe6, 0x14, 0x17, 0x34, 0x1b,
            0x71, 0x0b, 0x72, 0x77, 0x68, 0x29, 0x4d, 0x0e, 0x6a, 0x27, 0x7b,
        ];
        assert_eq!(Ok(ByteString(test[3..].to_owned())), BTypes::bdecode(&test));

        test = vec![
            '6' as u8, ':' as u8, 'a' as u8, 'b' as u8, 0xb8, 0x9e, 0xaa, 0xc7,
        ];
        assert_eq!(Ok(ByteString(test[2..].to_owned())), BTypes::bdecode(&test));

        test = vec![
            '6' as u8, ':' as u8, 0xb8, 0x9e, 0xaa, 0xc7, 'a' as u8, 'b' as u8,
        ];
        assert_eq!(Ok(ByteString(test[2..].to_owned())), BTypes::bdecode(&test));
    }

    #[test]
    fn btype_string_bad() {
        const STRINGS: [&str; 4] = ["3eggs", "6:bacon", "3ham", ":coffee"];

        for encoded in STRINGS {
            assert!(BTypes::bdecode(&encoded.as_bytes().to_owned()).is_err());
        }
    }

    #[test]
    fn btype_integer() {
        const INTEGER_GOOD: [(isize, &str); 6] = [
            (1, "i1e"),
            (0, "i0e"),
            (-1, "i-1e"),
            (10, "i10e"),
            (12, "i12e"),
            (-20, "i-20e"),
        ];

        for (plain, encoded) in INTEGER_GOOD {
            assert_eq!(BTypes::Integer(plain).bencode(), encoded.as_bytes());
        }

        for (plain, encoded) in INTEGER_GOOD {
            assert_eq!(
                BTypes::bdecode(&encoded.as_bytes().to_owned()),
                Ok(BTypes::Integer(plain))
            );
        }
    }

    #[test]
    fn btype_integer_bad() {
        const INTEGER_BAD: [&str; 7] = ["i00e", "i01e", "i-0e", "i--0e", "i-01e", "i---1e", "k0e"];

        for encoded in INTEGER_BAD {
            assert!(BTypes::bdecode(&encoded.as_bytes().to_owned()).is_err());
        }
    }

    #[test]
    fn btype_list() {
        let lists: Vec<(BTypes, &str)> = vec![
            (
                List(vec![Integer(-1), Integer(10), Integer(12), Integer(-20)]),
                "li-1ei10ei12ei-20ee",
            ),
            (
                List(vec![
                    TextString(String::from("eggs")),
                    TextString(String::from("bacon")),
                    TextString(String::from("ham")),
                    TextString(String::from("coffee")),
                ]),
                "l4:eggs5:bacon3:ham6:coffeee",
            ),
            (
                List(vec![
                    Integer(-1),
                    TextString(String::from("eggs")),
                    Integer(-20),
                ]),
                "li-1e4:eggsi-20ee",
            ),
            // IMPROVEMENT: Add Malformed List test, Add nested list/dictionary test
        ];

        for (plain, encoded) in lists {
            assert_eq!(plain.bencode(), encoded.as_bytes().to_owned());
            assert_eq!(BTypes::bdecode(&encoded.as_bytes().to_owned()), Ok(plain));
        }
    }

    #[test]
    fn btype_dict() {
        let dict = Dict({
            let mut map: BTreeMap<String, BTypes> = BTreeMap::new();

            map.insert("Breakfast".to_owned(), TextString("Beans".to_owned()));
            map.insert("Servings".to_owned(), Integer(5));

            map
        });

        let encoded = "d9:Breakfast5:Beans8:Servingsi5ee";

        assert_eq!(dict.bencode(), encoded.as_bytes().to_owned());
        assert_eq!(BTypes::bdecode(&encoded.as_bytes().to_owned()), Ok(dict));

        let dict = Dict({
            let mut map: BTreeMap<String, BTypes> = BTreeMap::new();

            map.insert("Breakfast".to_owned(), TextString("Beans".to_owned()));
            map.insert(
                "Servings".to_owned(),
                List(vec![
                    Integer(-1),
                    TextString("eggs".to_owned()),
                    Integer(-20),
                ]),
            );

            map
        });

        let encoded = "d9:Breakfast5:Beans8:Servingsli-1e4:eggsi-20eee";

        assert_eq!(dict.bencode(), encoded.as_bytes().to_owned());
        assert_eq!(BTypes::bdecode(&encoded.as_bytes().to_owned()), Ok(dict));
    }
}
