use crate::encoding::{
    BEncodeable, BEncodingError, BString, DisplayFormat, collection::BEncoding,
    common::check_leader,
};
use std::collections::BTreeMap;

#[derive(Clone, PartialEq, Eq)]
pub struct BDict(pub BTreeMap<String, BEncoding>);

impl std::fmt::Display for BDict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.out(0))
    }
}

impl std::fmt::Debug for BDict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.out(0))
    }
}

impl DisplayFormat for BDict {
    fn out(&self, indent: usize) -> String {
        let items_string = self
            .0
            .iter()
            .map(|(k, v)| {
                format!(
                    "{}\"{}\": {}",
                    repeat(' ', indent + 2),
                    k,
                    v.out(indent + 2)
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "Dict({})(\n{}\n{})",
            self.0.len(),
            items_string,
            repeat(' ', indent)
        )
    }
}

impl BEncodeable for BDict {
    fn bencode(&self) -> Vec<u8> {
        let mut output = Vec::new();

        output.push(b'd');

        for (key, value) in self.0.iter() {
            output.extend(BString::TextString(key.to_owned()).bencode());
            output.extend(&value.bencode());
        }

        output.push(b'e');

        output
    }

    fn bdecode(input: &[u8]) -> Result<(Self, &[u8]), BEncodingError>
    where
        Self: Sized,
    {
        let Some((sign, remainder)) = input.split_first() else {
            return Err(BEncodingError::MissingInputType(input.to_owned()));
        };

        if *sign != b'd' {
            return Err(BEncodingError::InvalidType((*sign).into()));
        }

        let mut map: BTreeMap<String, BEncoding> = BTreeMap::new();
        let mut remainder = remainder;

        loop {
            if check_leader(remainder, 'e' as u8)? {
                remainder = remainder.get(1..).unwrap_or(&[]);
                break;
            }

            let (key, key_remainder) = BEncoding::bdecode(remainder)?;
            let (value, value_remainder) = BEncoding::bdecode(key_remainder)?;

            remainder = value_remainder;

            if let BEncoding::String(BString::TextString(k)) = key {
                map.insert(k, value);
            } else {
                return Err(BEncodingError::InvalidBType(key));
            }
        }

        Ok((BDict(map), remainder))
    }
}

fn repeat(ch: char, count: usize) -> String {
    (0..count).map(|_| ch).collect()
}

#[cfg(test)]
mod tests {
    use crate::encoding::{BDict, BEncodeable, BEncoding, BInteger, BList, BString};
    use std::collections::BTreeMap;

    #[test]
    fn btype_dict() {
        let dict = BDict({
            let mut map: BTreeMap<String, BEncoding> = BTreeMap::new();

            map.insert(
                "Breakfast".to_owned(),
                BEncoding::String(BString::TextString("Beans".to_owned())),
            );
            map.insert("Servings".to_owned(), BEncoding::Integer(BInteger(5)));

            map
        });

        let encoded = "d9:Breakfast5:Beans8:Servingsi5ee";

        assert_eq!(dict.bencode(), encoded.as_bytes().to_owned());
        assert_eq!(
            BEncoding::bdecode(&encoded.as_bytes().to_owned())
                .unwrap()
                .0,
            BEncoding::Dict(dict)
        );

        let dict = BDict({
            let mut map: BTreeMap<String, BEncoding> = BTreeMap::new();

            map.insert(
                "Breakfast".to_owned(),
                BEncoding::String(BString::TextString("Beans".to_owned())),
            );
            map.insert(
                "Servings".to_owned(),
                BEncoding::List(BList(vec![
                    BEncoding::Integer(BInteger(-1)),
                    BEncoding::String(BString::TextString("eggs".to_owned())),
                    BEncoding::Integer(BInteger(-20)),
                ])),
            );

            map
        });

        let encoded = "d9:Breakfast5:Beans8:Servingsli-1e4:eggsi-20eee";

        assert_eq!(dict.bencode(), encoded.as_bytes().to_owned());
        assert_eq!(
            BEncoding::bdecode(&encoded.as_bytes().to_owned())
                .unwrap()
                .0,
            BEncoding::Dict(dict)
        );
    }
}
