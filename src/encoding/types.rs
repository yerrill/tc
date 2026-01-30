use crate::encoding::DisplayFormat;

use super::{BEncodeable, BInteger, BList, BString, errors::BEncodingError};
use std::{cmp::min, collections::BTreeMap, str::from_utf8};

#[derive(Clone, PartialEq, Eq)]
pub enum BEncoding {
    Integer(BInteger),
    String(BString),
    List(BList),
    Dict(BTreeMap<String, BEncoding>),
}

impl std::fmt::Display for BEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string(0))
    }
}

impl std::fmt::Debug for BEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string(0))
    }
}

impl BEncodeable for BEncoding {
    fn bencode(&self) -> Vec<u8> {
        let mut output: Vec<u8> = Vec::new();

        match self {
            BEncoding::Integer(int) => {
                output.extend(int.bencode());
            }
            BEncoding::String(str) => {
                output.extend(str.bencode());
            }
            BEncoding::List(items) => {
                output.extend(items.bencode());
            }
            BEncoding::Dict(btree_map) => {
                output.push('d' as u8);

                for (key, value) in btree_map {
                    output.extend(BEncoding::TextString(key.to_owned()).bencode());
                    output.extend(&value.bencode());
                }

                output.push('e' as u8);
            }
        };

        output
    }

    fn bdecode(input: &[u8]) -> Result<(Self, &[u8]), BEncodingError>
    where
        Self: Sized,
    {
        match bdecode(&input.as_slice()) {
            Ok((v, _)) => Ok(v),
            Err(e) => Err(e),
        }
    }
}

impl DisplayFormat for BEncoding {
    fn out(&self, indent: usize) -> String {
        let mut output: String = String::new();

        match self {
            BEncoding::Integer(i) => {
                output = format!("{output}");
            }
            BEncoding::TextString(s) => {
                output = format!("{output}TextString({})(\"{}\")", s.len(), s);
            }
            BEncoding::ByteString(items) => {
                let items_string = items[..min(items.len(), BTYPE_PRINT_MAX_ITEMS)]
                    .iter()
                    .map(|v| format!("0x{v:x}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                output = format!("{output}ByteString({})({})", items.len(), items_string);
            }
            BEncoding::List(items) => {
                let items_string = items[..min(items.len(), BTYPE_PRINT_MAX_ITEMS)]
                    .iter()
                    .map(|v| v.to_string(indent))
                    .collect::<Vec<_>>()
                    .join(", ");
                output = format!("{output}List[{items_string}]");
            }
            BEncoding::Dict(btree_map) => {
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

impl BEncoding {
    pub fn expect_dict(self) -> Result<BTreeMap<String, BEncoding>, BEncodingError> {
        let BEncoding::Dict(d) = self else {
            return Err(BEncodingError::NotDict);
        };

        Ok(d)
    }

    pub fn expect_list(self) -> Result<Vec<BEncoding>, BEncodingError> {
        let BEncoding::List(l) = self else {
            return Err(BEncodingError::NotList);
        };

        Ok(l)
    }

    pub fn expect_text_str(self) -> Result<String, BEncodingError> {
        let BEncoding::TextString(t) = self else {
            return Err(BEncodingError::NotTextStr);
        };

        Ok(t)
    }

    pub fn expect_byte_str(self) -> Result<Vec<u8>, BEncodingError> {
        let BEncoding::ByteString(b) = self else {
            return Err(BEncodingError::NotByteStr);
        };

        Ok(b)
    }

    pub fn expect_int(self) -> Result<isize, BEncodingError> {
        let BEncoding::Integer(i) = self else {
            return Err(BEncodingError::NotInt);
        };

        Ok(i)
    }

    pub fn keyed_dict(
        self,
        key: &str,
    ) -> Result<(BTreeMap<String, BEncoding>, BEncoding), BEncodingError> {
        let mut d = self.expect_dict()?;

        let Some(value) = d.remove(key) else {
            return Err(BEncodingError::KeyNotFound(key.to_owned()));
        };

        let value = value.expect_dict()?;

        Ok((value, BEncoding::Dict(d)))
    }

    pub fn keyed_list(self, key: &str) -> Result<(Vec<BEncoding>, BEncoding), BEncodingError> {
        let mut d = self.expect_dict()?;

        let Some(value) = d.remove(key) else {
            return Err(BEncodingError::KeyNotFound(key.to_owned()));
        };

        let value = value.expect_list()?;

        Ok((value, BEncoding::Dict(d)))
    }

    pub fn keyed_text_str(self, key: &str) -> Result<(String, BEncoding), BEncodingError> {
        let mut d = self.expect_dict()?;

        let Some(value) = d.remove(key) else {
            return Err(BEncodingError::KeyNotFound(key.to_owned()));
        };

        let value = value.expect_text_str()?;

        Ok((value, BEncoding::Dict(d)))
    }

    pub fn keyed_byte_str(self, key: &str) -> Result<(Vec<u8>, BEncoding), BEncodingError> {
        let mut d = self.expect_dict()?;

        let Some(value) = d.remove(key) else {
            return Err(BEncodingError::KeyNotFound(key.to_owned()));
        };

        let value = value.expect_byte_str()?;

        Ok((value, BEncoding::Dict(d)))
    }

    pub fn keyed_int(self, key: &str) -> Result<(isize, BEncoding), BEncodingError> {
        let mut d = self.expect_dict()?;

        let Some(value) = d.remove(key) else {
            return Err(BEncodingError::KeyNotFound(key.to_owned()));
        };

        let value = value.expect_int()?;

        Ok((value, BEncoding::Dict(d)))
    }
}

fn repeat(ch: char, count: usize) -> String {
    (0..count).map(|_| ch).collect()
}

fn bdecode(text: &[u8]) -> Result<(BEncoding, &[u8]), BEncodingError> {
    let Some((type_char, _)) = text.split_first() else {
        return Err(BEncodingError::MissingInputType(text.to_owned()));
    };

    match *type_char as char {
        'i' => parse_integer(text),
        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => parse_string(text),
        'l' => parse_list(text),
        'd' => parse_dictionary(text),
        _ => Err(BEncodingError::InvalidType(*type_char as char)),
    }
}

fn parse_dictionary(text: &[u8]) -> Result<(BEncoding, &[u8]), BEncodingError> {
    let Some((_, remainder)) = text.split_first() else {
        return Err(BEncodingError::MissingInputType(text.to_owned()));
    };

    let mut map: BTreeMap<String, BEncoding> = BTreeMap::new();
    let mut remainder = remainder;

    loop {
        if check_leader(remainder, 'e' as u8)? {
            remainder = remainder.get(1..).unwrap_or(&[]);
            break;
        }

        let (key, key_remainder) = bdecode(remainder)?;
        let (value, value_remainder) = bdecode(key_remainder)?;

        remainder = value_remainder;

        if let BEncoding::TextString(k) = key {
            map.insert(k, value);
        } else {
            return Err(BEncodingError::InvalidBType(key));
        }
    }

    Ok((BEncoding::Dict(map), remainder))
}
