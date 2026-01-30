mod common;
mod errors;
mod int;
mod list;
mod strings;
pub mod types;

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

#[cfg(test)]
mod tests {
    use super::types::{
        BEncoding,
        BEncoding::{ByteString, Dict, Integer, List, TextString},
    };
    use std::collections::BTreeMap;

    #[test]
    fn btype_dict() {
        let dict = Dict({
            let mut map: BTreeMap<String, BEncoding> = BTreeMap::new();

            map.insert("Breakfast".to_owned(), TextString("Beans".to_owned()));
            map.insert("Servings".to_owned(), Integer(5));

            map
        });

        let encoded = "d9:Breakfast5:Beans8:Servingsi5ee";

        assert_eq!(dict.bencode(), encoded.as_bytes().to_owned());
        assert_eq!(BEncoding::bdecode(&encoded.as_bytes().to_owned()), Ok(dict));

        let dict = Dict({
            let mut map: BTreeMap<String, BEncoding> = BTreeMap::new();

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
        assert_eq!(BEncoding::bdecode(&encoded.as_bytes().to_owned()), Ok(dict));
    }
}
