use super::types::BEncoding;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BEncodingError {
    // ADD FROM OPTION<CHAR>
    CharacterNotFound(char),
    ParseIntFailure,
    MalformedString(String),
    IncorrectStartingCharacter(char),
    Nested(String),
    InvalidType(char),
    InvalidBType(BEncoding),
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

impl std::fmt::Display for BEncodingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CharacterNotFound(c) => {
                write!(f, "Encoding Error: CharacterNotFound {c}")
            }
            Self::ParseIntFailure => write!(f, "Encoding Error: ParseIntFailure"),
            Self::MalformedString(s) => write!(f, "Encoding Error: MalformedString {s}"),
            Self::IncorrectStartingCharacter(c) => {
                write!(f, "Encoding Error: IncorrectStartingChar {c}")
            }
            Self::Nested(s) => write!(f, "Encoding Error: Nested {s}"),
            Self::InvalidType(c) => write!(f, "Encoding Error: InvalidType {c}"),
            Self::OutOfBounds => write!(f, "Encoding Error: OutOfBounds"),
            Self::InvalidBType(b) => write!(f, "Encoding Error: InvalidBType {b:?}"),
            Self::MissingInputType(v) => {
                write!(
                    f,
                    "Encoding Error: Decoding input had no type character {v:?}"
                )
            }
            Self::CouldNotParseUTF8 => {
                write!(f, "Encoding Error: Could not parse UTF8 from input")
            }
            Self::KeyNotFound(k) => {
                write!(f, "Encoding Error: Key {k} not found in dict")
            }
            Self::NotDict => {
                write!(f, "Encoding Error: Expected value not dictionary")
            }
            Self::NotList => write!(f, "Encoding Error: Expected value not list"),
            Self::NotInt => write!(f, "Encoding Error: Expected value not int"),
            Self::NotByteStr => {
                write!(f, "Encoding Error: Expected value not byte string")
            }
            Self::NotTextStr => {
                write!(f, "Encoding Error: Expected value not text string")
            }
        }
    }
}

impl std::error::Error for BEncodingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
