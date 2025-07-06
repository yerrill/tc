pub mod errors;
pub mod types;

#[cfg(test)]
mod tests {
    use super::types::{
        BTypes,
        BTypes::{ByteString, Dict, Integer, List, TextString},
    };
    use std::collections::BTreeMap;

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
