use super::BEncodingError;

pub const BTYPE_PRINT_MAX_ITEMS: usize = 100;

/// Seeks forward in slice to find first byte matching target.
/// Splits slice from `[0, mid)` and `(mid, len)`
pub fn split_on_delimiter(input: &[u8], target_char: u8) -> Result<(&[u8], &[u8]), BEncodingError> {
    // TODO: Replace with Slice::split_once when out of nightly
    for (index, ch) in input.iter().enumerate() {
        if target_char == *ch {
            let (left, right) = input.split_at(index);

            let right = right.get(1..).unwrap_or(&[]);

            return Ok((left, right));
        }
    }

    Err(BEncodingError::CharacterNotFound(target_char as char))
}
