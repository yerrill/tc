pub mod encoding;
use encoding::{
    BTypes::{self, ByteString, Dict, Integer, List, TextString},
    BencodingError,
};
mod metainfo;
use metainfo::*;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let mut file = File::open("src\\test 1.torrent").unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();

    let info = BTypes::bdecode(&contents).unwrap();

    println!("{:?}", Metainfo::dbencode(info).unwrap());
}
