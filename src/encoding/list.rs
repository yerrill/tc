use crate::encoding::{
    BEncodeable, BEncodingError, DisplayFormat,
    collection::BEncoding,
    common::{BTYPE_PRINT_MAX_ITEMS, check_leader},
};
use std::cmp::min;

#[derive(Clone, PartialEq, Eq)]
pub struct BList(pub Vec<BEncoding>);

impl std::fmt::Display for BList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.out(0))
    }
}

impl std::fmt::Debug for BList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.out(0))
    }
}

impl DisplayFormat for BList {
    fn out(&self, indent: usize) -> String {
        let items_string = self.0[..min(self.0.len(), BTYPE_PRINT_MAX_ITEMS)]
            .iter()
            .map(|v| v.out(indent))
            .collect::<Vec<_>>()
            .join(", ");

        format!("List[{items_string}]")
    }
}

impl BEncodeable for BList {
    fn bencode(&self) -> Vec<u8> {
        let mut output = Vec::new();

        output.push(b'l');

        for item in self.0.iter() {
            output.extend(item.bencode());
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

        if *sign != b'l' {
            return Err(BEncodingError::InvalidType((*sign).into()));
        }

        let mut values: Vec<BEncoding> = Vec::new();
        let mut remainder = remainder;

        loop {
            if check_leader(remainder, b'e')? {
                remainder = remainder.get(1..).unwrap_or(&[]);
                break;
            }

            let (value, new_remainder) = BEncoding::bdecode(remainder)?;
            values.push(value);
            remainder = new_remainder;
        }

        Ok((BList(values), remainder))
    }
}

#[cfg(test)]
mod tests {
    use crate::encoding::{BEncodeable, BEncoding, BInteger, BList, BString};

    #[test]
    fn list() {
        let lists: Vec<(BList, &str)> = vec![
            (
                BList(vec![
                    BEncoding::Integer(BInteger(-1)),
                    BEncoding::Integer(BInteger(10)),
                    BEncoding::Integer(BInteger(12)),
                    BEncoding::Integer(BInteger(-20)),
                ]),
                "li-1ei10ei12ei-20ee",
            ),
            (
                BList(vec![
                    BEncoding::String(BString::TextString(String::from("eggs"))),
                    BEncoding::String(BString::TextString(String::from("bacon"))),
                    BEncoding::String(BString::TextString(String::from("ham"))),
                    BEncoding::String(BString::TextString(String::from("coffee"))),
                ]),
                "l4:eggs5:bacon3:ham6:coffeee",
            ),
            (
                BList(vec![
                    BEncoding::Integer(BInteger(-1)),
                    BEncoding::String(BString::TextString(String::from("eggs"))),
                    BEncoding::Integer(BInteger(-20)),
                ]),
                "li-1e4:eggsi-20ee",
            ),
            // IMPROVEMENT: Add Malformed List test, Add nested list/dictionary test
        ];

        for (plain, encoded) in lists {
            assert_eq!(plain.bencode(), encoded.as_bytes().to_owned());
            assert_eq!(
                BEncoding::bdecode(&encoded.as_bytes().to_owned())
                    .unwrap()
                    .0,
                BEncoding::List(plain)
            );
        }
    }
}
