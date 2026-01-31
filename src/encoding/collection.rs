use crate::encoding::{
    BDict, BEncodeable, BEncodingError, BInteger, BList, BString, DisplayFormat,
};
use std::collections::BTreeMap;

#[derive(Clone, PartialEq, Eq)]
pub enum BEncoding {
    Integer(BInteger),
    String(BString),
    List(BList),
    Dict(BDict),
}

impl std::fmt::Display for BEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.out(0))
    }
}

impl std::fmt::Debug for BEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.out(0))
    }
}

impl DisplayFormat for BEncoding {
    fn out(&self, indent: usize) -> String {
        match self {
            Self::Integer(i) => i.out(indent),
            Self::String(s) => s.out(indent),
            Self::List(l) => l.out(indent),
            Self::Dict(d) => d.out(indent),
        }
    }
}

impl BEncodeable for BEncoding {
    fn bencode(&self) -> Vec<u8> {
        let mut output: Vec<u8> = Vec::new();

        match self {
            Self::Integer(int) => {
                output.extend(int.bencode());
            }
            Self::String(str) => {
                output.extend(str.bencode());
            }
            Self::List(items) => {
                output.extend(items.bencode());
            }
            Self::Dict(btree_map) => {
                output.extend(btree_map.bencode());
            }
        };

        output
    }

    fn bdecode(input: &[u8]) -> Result<(Self, &[u8]), BEncodingError>
    where
        Self: Sized,
    {
        let Some((type_char, _)) = input.split_first() else {
            return Err(BEncodingError::MissingInputType(input.to_owned()));
        };

        let pair = match *type_char as char {
            'i' => {
                let (value, remainder) = BInteger::bdecode(input)?;
                (Self::Integer(value), remainder)
            }
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                let (value, remainder) = BString::bdecode(input)?;
                (Self::String(value), remainder)
            }
            'l' => {
                let (value, remainder) = BList::bdecode(input)?;
                (Self::List(value), remainder)
            }
            'd' => {
                let (value, remainder) = BDict::bdecode(input)?;
                (Self::Dict(value), remainder)
            }
            _ => {
                return Err(BEncodingError::InvalidType(*type_char as char));
            }
        };

        Ok(pair)
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
