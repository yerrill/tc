pub mod encoding;
use encoding::{
    BTypes::{self, ByteString, Dict, Integer, List, TextString},
    BencodingError,
};
mod metainfo;
use metainfo::*;
mod network;
use network::*;
mod tracker;
use std::fs::File;
use std::io::prelude::*;
use tracker::*;

#[tokio::main]
async fn main() {
    let mut file = File::open("src\\test.torrent").unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();

    let info = Meta::bdecode(BTypes::bdecode(&contents).unwrap()).unwrap();

    let start = tracker_get(&info, 6881, 0, 0, 0, TrackerEvent::Started).await;

    let end = tracker_get(&info, 6881, 0, 0, 0, TrackerEvent::Stopped).await;
}
