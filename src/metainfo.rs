use std::collections::BTreeMap;

use crate::encoding::{BTypes::*, *};

pub trait Bencodeable {
    fn bencode(self) -> BTypes;
    fn dbencode(input: BTypes) -> Result<Self, DataParseError>
    where
        Self: Sized;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataParseError {
    ExpectedDict,
    ExpectedInteger,
    ExpectedList,
    ExpectedTextString,
    BadKey(String, Option<BTypes>),
    BadKeyPair(String, Option<BTypes>, String, Option<BTypes>),
    BadPieceLength(isize),
}

impl std::error::Error for DataParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::fmt::Display for DataParseError {
    // IMPROVE: Mixing of debug & display traits
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataParseError::ExpectedDict => write!(f, "Expected Dict, got other"),
            DataParseError::ExpectedInteger => write!(f, "Expected Integer, got other"),
            DataParseError::ExpectedList => write!(f, "Expected List, got other"),
            DataParseError::ExpectedTextString => write!(f, "Expected TextString, got other"),
            DataParseError::BadKey(k, v) => write!(f, "Expected key & value not met, {k:?} {v:?}"),
            DataParseError::BadKeyPair(s1, btypes1, s2, btypes2) => write!(
                f,
                "Expected keys & values not met, {s1:?} {btypes1:?}, {s2:?}, {btypes2:?}"
            ),
            DataParseError::BadPieceLength(n) => write!(f, "Piece length invalid {n}"),
        }
    }
}

/// Metainfo files (also known as .torrent files) are bencoded dictionaries
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metainfo {
    /// The URL of the tracker. All strings in a .torrent file that contains text must be UTF-8 encoded.
    /// Unofficially seems there can be multiple announce keys
    announce: String,

    //info: MetainfoInfo,
    /// Part of the `info` key. The `name` key maps to a UTF-8 encoded string which is the suggested name to save the file (or directory) as.
    /// It is purely advisory.
    /// In the single file case, the name key is the name of a file, in the muliple file case, it's the name of a directory.
    name: String,

    /// Part of the `info` key. `piece_length` maps to the number of bytes in each piece the file is split into.
    /// For the purposes of transfer, files are split into fixed-size pieces which are all the same length except for possibly the last one which may be truncated.
    /// `piece_length` is almost always a power of two, most commonly `2^18 = 256 K` (BitTorrent prior to version 3.2 uses `2 20 = 1 M` as default).
    piece_length: usize,

    /// Part of the `info` key. `pieces` maps to a string whose length is a multiple of `20`. It is to be subdivided into strings of length `20`,
    /// each of which is the SHA1 hash of the piece at the corresponding index.
    pieces: Vec<u8>,

    /// Part of the `info` key. There is also a key length or a key files, but not both or neither.
    /// If length is present then the download represents a single file, otherwise it represents a set of files which go in a directory structure.
    download_type: DownloadTypes,

    /// Any unofficial leftover keys that might be needed for a hash but not functionality
    leftovers: BTypes,
}

impl Bencodeable for Metainfo {
    fn bencode(self) -> BTypes {
        return BTypes::Dict({
            let mut dict: BTreeMap<String, BTypes> = BTreeMap::new();

            dict.insert("announce".to_owned(), TextString(self.announce));

            dict.insert(
                "info".to_owned(),
                Dict({
                    let mut info: BTreeMap<String, BTypes> = BTreeMap::new();

                    info.insert("name".to_owned(), TextString(self.name));

                    info.insert(
                        "piece length".to_owned(),
                        Integer(self.piece_length as isize),
                    );

                    info.insert("pieces".to_owned(), ByteString(self.pieces));

                    match self.download_type {
                        DownloadTypes::Single { .. } => {
                            info.insert("length".to_owned(), self.download_type.bencode());
                        }
                        DownloadTypes::Multiple { .. } => {
                            info.insert("files".to_owned(), self.download_type.bencode());
                        }
                    }

                    info
                }),
            );

            dict
        });
    }

    fn dbencode(input: BTypes) -> Result<Self, DataParseError>
    where
        Self: Sized,
    {
        let BTypes::Dict(mut dict) = input else {
            return Err(DataParseError::ExpectedDict);
        };

        let Some(BTypes::TextString(announce)) = dict.remove("announce") else {
            return Err(DataParseError::BadKey(
                "announce".to_owned(),
                dict.get("announce").cloned(),
            ));
        }; // IMRPOVE: Macro to do this

        let Some(BTypes::Dict(mut info)) = dict.remove("info") else {
            return Err(DataParseError::BadKey(
                "info".to_owned(),
                dict.get("info").cloned(),
            ));
        };

        let Some(BTypes::TextString(name)) = info.remove("name") else {
            return Err(DataParseError::BadKey(
                "info.name".to_owned(),
                info.get("name").cloned(),
            ));
        };

        let Some(BTypes::Integer(piece_length)) = info.remove("piece length") else {
            return Err(DataParseError::BadKey(
                "info.piece length".to_owned(),
                info.get("piece length").cloned(),
            ));
        };

        if piece_length <= 0 {
            return Err(DataParseError::BadPieceLength(piece_length));
        }

        // if 2_isize.pow(piece_length.ilog2()) != piece_length {
        //    return Err(DataParseError::BadPieceLength(piece_length));
        //}

        let Some(BTypes::ByteString(pieces)) = info.remove("pieces") else {
            return Err(DataParseError::BadKey(
                "info.pieces".to_owned(),
                info.get("pieces").cloned(),
            ));
        };

        let (download_types, leftover_info) = DownloadTypes::dbencode(BTypes::Dict(info))?;

        Ok(Self {
            announce,
            name,
            piece_length: piece_length as usize,
            pieces,
            download_type: download_types,
            leftovers: leftover_info,
        })
    }
}

/// Subtype of Metainfo. Splits the two cases of single file download vs multiple file download.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadTypes {
    Single {
        /// In the single file case, length maps to the length of the file in bytes.
        length: usize,
    },
    Multiple {
        /// For the purposes of the other keys, the multi-file case is treated as only having a single file by concatenating the files in the order they appear in the files list.
        /// The files list is the value files maps to, and is a list of `MultipleFileInner`.
        files: Vec<MultipleFileInner>,
    },
}

impl DownloadTypes {
    fn bencode(self) -> BTypes {
        match self {
            Self::Single { length } => BTypes::Integer(length as isize),
            Self::Multiple { files } => {
                BTypes::List(files.iter().map(|v| v.clone().bencode()).collect())
            }
        }
    }

    fn dbencode(input: BTypes) -> Result<(Self, BTypes), DataParseError>
    // IMPROVEMENT: Convert some of these to type aliases
    where
        Self: Sized,
    {
        let BTypes::Dict(mut info) = input else {
            return Err(DataParseError::ExpectedDict);
        };

        let length = info.remove("length");
        let files = info.remove("files");

        let download_type = match (length, files) {
            (None, None) => {
                return Err(DataParseError::BadKey(
                    "info.length and info.files".to_owned(),
                    None,
                ));
            }
            (None, Some(f)) => {
                let BTypes::List(list) = f else {
                    return Err(DataParseError::ExpectedList);
                };

                let mut file_list = Vec::new();

                for item in list {
                    file_list.push(MultipleFileInner::dbencode(item)?);
                }

                Self::Multiple { files: file_list }
            }
            (Some(l), None) => {
                let BTypes::Integer(i) = l else {
                    return Err(DataParseError::ExpectedInteger);
                };
                Self::Single { length: i as usize }
            }
            (Some(l), Some(f)) => {
                return Err(DataParseError::BadKeyPair(
                    "length".to_owned(),
                    Some(l),
                    "files".to_owned(),
                    Some(f),
                ));
            }
        };

        Ok((download_type, BTypes::Dict(info)))
    }
}

/// Dictionary for use in multiple file downloads
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultipleFileInner {
    /// The length of the file, in bytes.
    length: usize,

    /// A list of UTF-8 encoded strings corresponding to subdirectory names, the last of which is the actual file name (a zero length list is an error case).
    path: Vec<String>,
}

impl MultipleFileInner {
    fn bencode(self) -> BTypes {
        BTypes::Dict({
            let mut map = BTreeMap::new();
            map.insert("length".to_owned(), BTypes::Integer(self.length as isize));
            map.insert(
                "path".to_owned(),
                BTypes::List(
                    self.path
                        .iter()
                        .map(|s| BTypes::TextString(s.to_owned()))
                        .collect(),
                ),
            );
            map
        })
    }

    fn dbencode(input: BTypes) -> Result<Self, DataParseError>
    where
        Self: Sized,
    {
        let BTypes::Dict(mut info) = input else {
            return Err(DataParseError::ExpectedDict);
        };

        let Some(BTypes::Integer(length)) = info.remove("length") else {
            return Err(DataParseError::BadKey(
                "length".to_owned(),
                info.get("length").cloned(),
            ));
        };

        let path = {
            let mut path = Vec::new();

            let Some(BTypes::List(path_list)) = info.remove("path") else {
                return Err(DataParseError::BadKey(
                    "path".to_owned(),
                    info.get("path").cloned(),
                ));
            };

            for item in path_list {
                let BTypes::TextString(segment) = item else {
                    return Err(DataParseError::ExpectedTextString);
                };

                path.push(segment);
            }

            path
        };

        Ok(MultipleFileInner {
            length: length as usize,
            path,
        })
    }
}

#[cfg(test)] // IMPROVEMENT: could be significantly expanded
mod tests {
    use super::DownloadTypes::*;
    use super::*;

    #[test]
    fn single() {
        let test_value = Metainfo {
            announce: "www.example.com".to_string(),
            name: "The test file".to_string(),
            piece_length: 4,
            pieces: vec![0x12, 0x43, 0x76, 0xaf],
            download_type: Single { length: 80 },
            leftovers: BTypes::Dict(BTreeMap::new()),
        };

        assert_eq!(
            Ok(test_value.clone()),
            Metainfo::dbencode(test_value.bencode())
        );
    }

    #[test]
    fn multi() {
        let test_value = Metainfo {
            announce: "www.example.com".to_string(),
            name: "The test file".to_string(),
            piece_length: 4,
            pieces: vec![0x12, 0x43, 0x76, 0xaf],
            download_type: Multiple {
                files: vec![
                    MultipleFileInner {
                        length: 15,
                        path: vec!["foo".to_string(), "bar".to_string(), "baz".to_string()],
                    },
                    MultipleFileInner {
                        length: 24,
                        path: vec!["best file ever TM".to_string()],
                    },
                ],
            },
            leftovers: BTypes::Dict(BTreeMap::new()),
        };

        assert_eq!(
            Ok(test_value.clone()),
            Metainfo::dbencode(test_value.bencode())
        );
    }
}
