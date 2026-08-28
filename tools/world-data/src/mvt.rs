use flate2::read::GzDecoder;
use prost::Message;
use std::io::Read;

#[derive(Clone, PartialEq, Message)]
pub(crate) struct Tile {
    #[prost(message, repeated, tag = "3")]
    pub(crate) layers: Vec<Layer>,
}

#[derive(Clone, PartialEq, Message)]
pub(crate) struct Layer {
    #[prost(string, required, tag = "1")]
    pub(crate) name: String,
    #[prost(message, repeated, tag = "2")]
    pub(crate) features: Vec<Feature>,
    #[prost(uint32, optional, tag = "5")]
    pub(crate) extent: Option<u32>,
    #[prost(uint32, required, tag = "15")]
    pub(crate) version: u32,
}

#[derive(Clone, PartialEq, Message)]
pub(crate) struct Feature {
    #[prost(uint32, repeated, packed = "true", tag = "4")]
    pub(crate) geometry: Vec<u32>,
}

pub(crate) fn decode(bytes: &[u8]) -> Result<Tile, String> {
    let mut decoded = Vec::new();
    GzDecoder::new(bytes)
        .read_to_end(&mut decoded)
        .map_err(|error| format!("could not decompress zoom-0 vector tile: {error}"))?;
    Tile::decode(decoded.as_slice())
        .map_err(|error| format!("could not decode zoom-0 vector tile: {error}"))
}

pub(crate) fn decode_geometry(commands: &[u32]) -> Vec<Vec<(i32, i32)>> {
    let mut paths = Vec::new();
    let mut path = Vec::new();
    let mut index = 0;
    let mut x = 0;
    let mut y = 0;

    while index < commands.len() {
        let command = commands[index] & 0x7;
        let count = (commands[index] >> 3) as usize;
        index += 1;
        match command {
            1 => {
                for _ in 0..count {
                    if index + 1 >= commands.len() {
                        break;
                    }
                    x += zigzag(commands[index]);
                    y += zigzag(commands[index + 1]);
                    index += 2;
                    if !path.is_empty() {
                        paths.push(std::mem::take(&mut path));
                    }
                    path.push((x, y));
                }
            }
            2 => {
                for _ in 0..count {
                    if index + 1 >= commands.len() {
                        break;
                    }
                    x += zigzag(commands[index]);
                    y += zigzag(commands[index + 1]);
                    index += 2;
                    path.push((x, y));
                }
            }
            7 => {
                if let Some(&first) = path.first() {
                    path.push(first);
                }
            }
            _ => {}
        }
    }
    if !path.is_empty() {
        paths.push(path);
    }
    paths
}

fn zigzag(value: u32) -> i32 {
    ((value >> 1) as i32) ^ -((value & 1) as i32)
}
