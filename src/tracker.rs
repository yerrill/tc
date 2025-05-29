use crate::metainfo::*;
use rand::{self, Rng};
use reqwest::get;

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
    fn header_value(&self) -> Option<&'static str> {
        match self {
            Self::Started => Some("started"),
            Self::Completed => Some("completed"),
            Self::Stopped => Some("stopped"),
            Self::Empty => None,
        }
    }
}

pub async fn tracker_get(details: &TrackerDetails<'_>) -> Result<(), ()> {
    let info_hash = details
        .meta
        .info_hash()
        .iter()
        .map(|b| format!("%{:X}", b))
        .collect::<String>();

    let peer_id = generate_peer_id(); // Generate random ID and return with response

    //let body = get()
    //let query_string = format!("?info_hash{}&peer_id{}", metainfo.info_hash(), peer_id);

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
