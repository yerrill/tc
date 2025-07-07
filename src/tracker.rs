use crate::encoding::{errors::BencodingError, types::BTypes};
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
) -> [u8; 20] {
    let info_hash = percent_encode(meta.info_hash().as_slice(), NON_ALPHANUMERIC).to_string();

    let peer_id_bytes = generate_peer_id();
    // Generate random ID and return with response, shouldn't need to be percent encoded with current generation method
    // let peer_id = percent_encode(peer_id_bytes.as_slice(), NON_ALPHANUMERIC).to_string();
    let peer_id = "abcdefghijklmnopqrst";

    println!("Peer id: {:?}", &peer_id);
    println!("Peer id bytes: {:?}", &peer_id_bytes);

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

    peer_id_bytes
}

pub fn generate_peer_id() -> [u8; 20] {
    let mut chars = ['A' as u8; 20];
    let mut rng = rand::rng();

    for ch in 0..chars.len() {
        chars[ch] = rng.sample(rand::distr::Alphanumeric) as u8;
    }

    chars
}

fn decode_response(response: String) -> Result<(), BencodingError> {
    let res = BTypes::bdecode(&response.into_bytes())?;

    let (peers, res) = res.keyed_dict("peers")?;

    Ok(())
}
