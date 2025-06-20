use crate::metainfo::*;
use percent_encoding::{NON_ALPHANUMERIC, percent_encode};
use rand::{self, Rng};
use reqwest::get;

// Refactor this into a trait for different tracker protocols

pub struct TrackerDetails<'a> {
    meta: &'a Meta,
    peer_id: String,
    port: usize,
    uploaded: usize,
    downloaded: usize,
    left: usize,
    event: TrackerEvent,
}

#[derive(Clone, Copy)]
pub enum TrackerEvent {
    Started,
    Completed,
    Stopped,
    Empty,
}

impl TrackerEvent {
    fn header_value(&self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::Completed => "completed",
            Self::Stopped => "stopped",
            Self::Empty => "empty",
        }
    }
}

pub async fn tracker_get(
    // IP not here rn come back later
    meta: &Meta,
    port: u16,
    uploaded: usize,
    downloaded: usize,
    left: usize,
    event: TrackerEvent,
) -> Result<(), ()> {
    let info_hash = percent_encode(meta.info_hash().as_slice(), NON_ALPHANUMERIC).to_string();

    println!("{}", info_hash);

    // Generate random ID and return with response, shouldn't need to be percent encoded with current generation method
    let peer_id = percent_encode(generate_peer_id().as_bytes(), NON_ALPHANUMERIC).to_string();

    let query_string = format!(
        "info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&event={}",
        info_hash,
        peer_id,
        port,
        uploaded,
        downloaded,
        left,
        event.header_value()
    );

    let response = match reqwest::Client::new()
        .get(format!("{}?{}", meta.announce, query_string))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => panic!("{:?}", e),
    };

    println!("{:?}", response);
    println!("{:?}", response.text().await);

    Ok(())
}

pub fn generate_peer_id() -> String {
    let mut chars = ['A'; 20];
    let mut rng = rand::rng();

    for ch in 0..chars.len() {
        chars[ch] = rng.sample(rand::distr::Alphanumeric) as char;
    }

    chars.iter().collect()
}

// For consistency, percent-encoded octets in the ranges of ALPHA (%41-%5A and %61-%7A),
// DIGIT (%30-%39), hyphen (%2D), period (%2E), underscore (%5F), or tilde (%7E) should
// not be created by URI producers and, when found in a URI, should be decoded to their
// corresponding unreserved characters by URI normalizers.
fn url_encode(data: &[u8]) -> String {
    const RESERVED: [char; 18] = [
        ':', '/', '?', '#', '[', ']', '@', '!', '$', '&', '\'', '(', ')', '*', '+', ',', ';', '=',
    ];

    let mut output = String::new();

    for ch in data {
        dbg!(*ch as char);
        if RESERVED.contains(&(*ch as char)) {
            output.push_str(format!("%{:02X}", *ch).as_str());
        } else {
            output.push(*ch as char);
        }
    }

    output
}
