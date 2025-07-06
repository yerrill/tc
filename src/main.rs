mod encoding;
mod metainfo;
mod network;
mod tracker;

use encoding::{
    errors::BencodingError,
    types::BTypes::{self, ByteString, Dict, Integer, List, TextString},
};
use metainfo::*;
use network::*;
use std::fs::File;
use std::io::prelude::*;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
use tracker::*;

#[tokio::main]
async fn main() {
    let mut file = File::open("src\\test.torrent").unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();

    let info = Meta::bdecode(BTypes::bdecode(&contents).unwrap()).unwrap();

    let port = 6881;

    let start = tracker_get(&info, port, 0, 0, 0, TrackerEvent::Started).await;

    // let listener = TcpListener::bind("0.0.0.0:6881").await.unwrap();
    let listener = TcpListener::bind("0.0.0.0:6881").await.unwrap();

    let mut counter = 0;

    while counter < 5 {
        let (mut socket, addr) = listener.accept().await.unwrap();
        let mut buffer = Vec::new();
        let _ = socket.read_to_end(&mut buffer).await.unwrap();
        dbg!(&socket);
        dbg!(&buffer);
        let _ = socket.write_all("HELLO".as_bytes());
        counter += 1;
    }

    let end = tracker_get(&info, port, 0, 0, 16384, TrackerEvent::Stopped).await;
}
