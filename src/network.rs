const BT_HEADER: &'static [u8] = "\x13BitTorrent protocol".as_bytes();

pub enum ProtocolError {
    NoBittorrentHeader,
    UnexpectedEnd,
    HeaderOverflow,
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl std::fmt::Debug for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.to_str())
    }
}

impl std::error::Error for ProtocolError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl ProtocolError {
    fn to_str(&self) -> &'static str {
        match self {
            Self::NoBittorrentHeader => "Protocol Error: No Bittorrent Header in handshake",
            Self::UnexpectedEnd => "Protocol Error: Header ended unexpectedly",
            Self::HeaderOverflow => "Protocol Error: Header length too long",
        }
    }
}

fn match_bytes<'a>(baseline: &'a [u8], criteria: &[u8]) -> Option<&'a [u8]> {
    let mut index = 0;

    loop {
        if index >= criteria.len() {
            break;
        }

        if let (Some(&b), Some(&c)) = (baseline.get(index), criteria.get(index)) {
            if b != c {
                return None;
            }
        } else {
            return None;
        }

        index += 1;
    }

    Some(&baseline[index..])
}

#[derive(PartialEq, Eq, Debug)]
pub struct HandshakeInfo {
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

impl HandshakeInfo {
    pub fn decode(received: Vec<u8>) -> Result<Self, ProtocolError> {
        let rs = received.as_slice();

        // Match Bittorrent header
        let Some(rs) = match_bytes(rs, BT_HEADER) else {
            return Err(ProtocolError::NoBittorrentHeader);
        };

        // Skip blank 8 bytes
        let Some(rs) = rs.get(8..) else {
            return Err(ProtocolError::UnexpectedEnd);
        };

        // Get infohash
        let Some((hash, rs)) = rs.split_at_checked(20) else {
            return Err(ProtocolError::UnexpectedEnd);
        };

        // Get peer id
        let Some((peer, rs)) = rs.split_at_checked(20) else {
            return Err(ProtocolError::UnexpectedEnd);
        };

        if rs.len() > 0 {
            return Err(ProtocolError::HeaderOverflow);
        }

        let mut info_hash: [u8; 20] = [0x00; 20];
        let mut peer_id: [u8; 20] = [0x00; 20];

        for i in 0..20 {
            info_hash[i] = hash[i];
            peer_id[i] = peer[i];
        }

        Ok(Self { info_hash, peer_id })
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        BT_HEADER.iter().for_each(|v| buffer.push(*v));

        (0..8).for_each(|_| buffer.push(0x00));

        self.info_hash.iter().for_each(|v| buffer.push(*v));

        self.peer_id.iter().for_each(|v| buffer.push(*v));

        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_bytes_1() {
        const BASELINE: &[u8] = &[0x1, 0x2, 0x3, 0x4];
        const CRITERIA: &[u8] = &[0x1, 0x2];
        const RESULT: &[u8] = &[0x3, 0x4];
        assert_eq!(match_bytes(BASELINE, CRITERIA).unwrap(), RESULT);
    }

    #[test]
    fn match_bytes_2() {
        const BASELINE: &[u8] = &[0x1, 0x2, 0x3, 0x4];
        const CRITERIA: &[u8] = &[0x1, 0x2, 0x3, 0x3];
        assert_eq!(match_bytes(BASELINE, CRITERIA), None);
    }

    #[test]
    fn match_bytes_3() {
        const BASELINE: &[u8] = &[0x1, 0x2, 0x3];
        const CRITERIA: &[u8] = &[0x1, 0x2, 0x3, 0x4];
        assert_eq!(match_bytes(BASELINE, CRITERIA), None);
    }

    #[test]
    fn header_decode_1() {
        const SAMPLE: [u8; 68] = [
            19, 66, 105, 116, 84, 111, 114, 114, 101, 110, 116, 32, 112, 114, 111, 116, 111, 99,
            111, 108, 0, 0, 0, 0, 0, 16, 0, 0, 58, 66, 135, 44, 34, 168, 10, 90, 45, 167, 115, 85,
            60, 19, 130, 167, 61, 106, 204, 188, 45, 84, 68, 48, 48, 48, 49, 45, 112, 100, 86, 68,
            102, 70, 51, 75, 100, 112, 119, 49,
        ];

        const RESULT: HandshakeInfo = HandshakeInfo {
            info_hash: [
                58, 66, 135, 44, 34, 168, 10, 90, 45, 167, 115, 85, 60, 19, 130, 167, 61, 106, 204,
                188,
            ],
            peer_id: [
                45, 84, 68, 48, 48, 48, 49, 45, 112, 100, 86, 68, 102, 70, 51, 75, 100, 112, 119,
                49,
            ],
        };

        assert_eq!(HandshakeInfo::decode(SAMPLE.to_vec()).unwrap(), RESULT);
    }

    #[test]
    fn header_encode_1() {
        const RESULT: [u8; 68] = [
            19, 66, 105, 116, 84, 111, 114, 114, 101, 110, 116, 32, 112, 114, 111, 116, 111, 99,
            111, 108, 0, 0, 0, 0, 0, 0, 0, 0, 58, 66, 135, 44, 34, 168, 10, 90, 45, 167, 115, 85,
            60, 19, 130, 167, 61, 106, 204, 188, 45, 84, 68, 48, 48, 48, 49, 45, 112, 100, 86, 68,
            102, 70, 51, 75, 100, 112, 119, 49,
        ];

        const SAMPLE: HandshakeInfo = HandshakeInfo {
            info_hash: [
                58, 66, 135, 44, 34, 168, 10, 90, 45, 167, 115, 85, 60, 19, 130, 167, 61, 106, 204,
                188,
            ],
            peer_id: [
                45, 84, 68, 48, 48, 48, 49, 45, 112, 100, 86, 68, 102, 70, 51, 75, 100, 112, 119,
                49,
            ],
        };

        assert_eq!(SAMPLE.encode(), RESULT.to_vec());
    }
}
