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

async fn connection(info: Meta) {
    let port = 6881;

    let peer_id = tracker_get(&info, port, 0, 0, 20, TrackerEvent::Started).await;

    // let listener = TcpListener::bind("0.0.0.0:6881").await.unwrap();
    let listener = TcpListener::bind("0.0.0.0:6881").await.unwrap();

    let mut counter = 0;

    while counter < 5 {
        let (mut socket, addr) = listener.accept().await.unwrap();
        let mut buffer = Vec::new();
        let _ = socket.read_to_end(&mut buffer).await.unwrap();
        println!("{:?}", &buffer);

        // let _ = socket.write_all("HELLO".as_bytes());
        let in_header = HandshakeInfo::decode(buffer).unwrap();
        println!("{:?}", &in_header.encode());

        let out_header = HandshakeInfo {
            peer_id: *b"abcdefghijklmnopqrst",
            info_hash: info.info_hash(),
        };
        println!("{:?}", &out_header.encode());

        if in_header.info_hash == out_header.info_hash {
            let _ = socket.write_all(&out_header.encode().as_slice()).await;
        } else {
            dbg!("oof");
        }

        counter += 1;
    }

    let end = tracker_get(&info, port, 0, 0, 16384, TrackerEvent::Stopped).await;
}

#[tokio::main]
async fn main() {
    let mut file = File::open("src\\test.torrent").unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();
    let info = Meta::bdecode(BTypes::bdecode(&contents).unwrap()).unwrap();

    const HEADER: &'static [u8] = "\x13BitTorrent protocol".as_bytes();

    connection(info).await;
}
