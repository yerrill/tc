use super::errors::BencodingError;
use std::{cmp::min, collections::BTreeMap, str::from_utf8};

const BTYPE_PRINT_MAX_ITEMS: usize = 100;

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

    pub fn expect_dict(self) -> Result<DictInner, BencodingError> {
        let BTypes::Dict(d) = self else {
            return Err(BencodingError::NotDict);
        };

        Ok(d)
    }

    pub fn expect_list(self) -> Result<Vec<BTypes>, BencodingError> {
        let BTypes::List(l) = self else {
            return Err(BencodingError::NotList);
        };

        Ok(l)
    }

    pub fn expect_text_str(self) -> Result<String, BencodingError> {
        let BTypes::TextString(t) = self else {
            return Err(BencodingError::NotTextStr);
        };

        Ok(t)
    }

    pub fn expect_byte_str(self) -> Result<Vec<u8>, BencodingError> {
        let BTypes::ByteString(b) = self else {
            return Err(BencodingError::NotByteStr);
        };

        Ok(b)
    }

    pub fn expect_int(self) -> Result<isize, BencodingError> {
        let BTypes::Integer(i) = self else {
            return Err(BencodingError::NotInt);
        };

        Ok(i)
    }

    pub fn keyed_dict(self, key: &str) -> Result<(DictInner, BTypes), BencodingError> {
        let mut d = self.expect_dict()?;

        let Some(value) = d.remove(key) else {
            return Err(BencodingError::KeyNotFound(key.to_owned()));
        };

        let value = value.expect_dict()?;

        Ok((value, BTypes::Dict(d)))
    }

    pub fn keyed_list(self, key: &str) -> Result<(Vec<BTypes>, BTypes), BencodingError> {
        let mut d = self.expect_dict()?;

        let Some(value) = d.remove(key) else {
            return Err(BencodingError::KeyNotFound(key.to_owned()));
        };

        let value = value.expect_list()?;

        Ok((value, BTypes::Dict(d)))
    }

    pub fn keyed_text_str(self, key: &str) -> Result<(String, BTypes), BencodingError> {
        let mut d = self.expect_dict()?;

        let Some(value) = d.remove(key) else {
            return Err(BencodingError::KeyNotFound(key.to_owned()));
        };

        let value = value.expect_text_str()?;

        Ok((value, BTypes::Dict(d)))
    }

    pub fn keyed_byte_str(self, key: &str) -> Result<(Vec<u8>, BTypes), BencodingError> {
        let mut d = self.expect_dict()?;

        let Some(value) = d.remove(key) else {
            return Err(BencodingError::KeyNotFound(key.to_owned()));
        };

        let value = value.expect_byte_str()?;

        Ok((value, BTypes::Dict(d)))
    }

    pub fn keyed_int(self, key: &str) -> Result<(isize, BTypes), BencodingError> {
        let mut d = self.expect_dict()?;

        let Some(value) = d.remove(key) else {
            return Err(BencodingError::KeyNotFound(key.to_owned()));
        };

        let value = value.expect_int()?;

        Ok((value, BTypes::Dict(d)))
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
