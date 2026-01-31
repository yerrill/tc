mod collection;
mod common;
mod dict;
mod errors;
mod int;
mod list;
mod strings;

pub use collection::BEncoding;
pub use dict::BDict;
pub use errors::BEncodingError;
pub use int::BInteger;
pub use list::BList;
pub use strings::BString;

pub trait BEncodeable {
    fn bencode(&self) -> Vec<u8>;
    fn bdecode(input: &[u8]) -> Result<(Self, &[u8]), BEncodingError>
    where
        Self: Sized;
}

trait DisplayFormat {
    fn out(&self, indent: usize) -> String;
}
