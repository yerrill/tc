use super::{BEncodeable, BEncodingError, DisplayFormat, common::split_on_delimiter};

#[derive(Clone, PartialEq, Eq)]
pub struct BInteger(pub isize);

impl std::fmt::Display for BInteger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.out(0))
    }
}

impl std::fmt::Debug for BInteger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.out(0))
    }
}

impl DisplayFormat for BInteger {
    fn out(&self, _indent: usize) -> String {
        format!("Integer({})", self.0)
    }
}

impl BEncodeable for BInteger {
    fn bencode(&self) -> Vec<u8> {
        let mut output = Vec::new();
        output.push(b'i');
        output.extend(self.0.to_string().as_bytes());
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

        if *sign != b'i' {
            return Err(BEncodingError::InvalidType((*sign).into()));
        }

        let (slice_numbers, remainder) = split_on_delimiter(remainder, b'e')?;

        let Ok(number_string) = std::str::from_utf8(slice_numbers) else {
            return Err(BEncodingError::CouldNotParseUTF8);
        };

        let number_value = {
            if let Ok(val) = number_string.parse::<isize>() {
                val
            } else {
                return Err(BEncodingError::ParseIntFailure);
            }
        };

        // TODO: Better way to check for bad integer encoding
        // Check for no leading 0s
        if number_string.len() != number_value.to_string().len() {
            return Err(BEncodingError::ParseIntFailure);
        }

        Ok((Self(number_value), remainder))
    }
}

#[cfg(test)]
mod tests {
    use super::{BEncodeable, BInteger};

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
            assert_eq!(BInteger(plain).bencode(), encoded.as_bytes());
        }

        for (plain, encoded) in INTEGER_GOOD {
            assert_eq!(
                BInteger::bdecode(&encoded.as_bytes().to_owned()).unwrap().0,
                BInteger(plain)
            );
        }
    }

    #[test]
    fn btype_integer_bad() {
        const INTEGER_BAD: [&str; 8] = [
            "i00e", "i01e", "i001e", "i-0e", "i--0e", "i-01e", "i---1e", "k0e",
        ];

        for encoded in INTEGER_BAD {
            assert!(BInteger::bdecode(&encoded.as_bytes().to_owned()).is_err());
        }
    }
}
