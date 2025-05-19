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

fn main() {
    let mut file = File::open("src\\test.torrent").unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();

    let info = Meta::bdecode(BTypes::bdecode(&contents).unwrap()).unwrap();

    //info.info.pieces = Vec::new();

    //println!("{:#?}", info);

    let infohash = info.info_hash().iter().map(|b| format!("%{:X}", b)).collect::<String>();

    println!("{}", infohash);
}
