use crate::encoding::BEncodingError;

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

pub fn check_leader(input: &[u8], leader: u8) -> Result<bool, BEncodingError> {
    let Some(first_char) = input.get(0) else {
        return Err(BEncodingError::OutOfBounds);
    };

    Ok(*first_char == leader)
}
