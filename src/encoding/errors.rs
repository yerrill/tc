use super::types::BTypes;

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
    KeyNotFound(String),
    NotDict,
    NotList,
    NotInt,
    NotByteStr,
    NotTextStr,
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
            BencodingError::KeyNotFound(k) => write!(f, "Key {k} not found in dict"),
            BencodingError::NotDict => write!(f, "Input or Expected value not dictionary"),
            BencodingError::NotList => write!(f, "Expected value not list"),
            BencodingError::NotInt => write!(f, "Expected value not int"),
            BencodingError::NotByteStr => write!(f, "Expected value not byte string"),
            BencodingError::NotTextStr => write!(f, "Expected value not text string"),
        }
    }
}

impl std::error::Error for BencodingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
