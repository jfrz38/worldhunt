use std::io::Read;

use flate2::read::GzDecoder;
use prost::Message;

#[derive(Clone, PartialEq, Message)]
pub(super) struct Tile {
    #[prost(message, repeated, tag = "3")]
    pub(super) layers: Vec<Layer>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct Layer {
    #[prost(string, required, tag = "1")]
    pub(super) name: String,
    #[prost(message, repeated, tag = "2")]
    pub(super) features: Vec<Feature>,
    #[prost(string, repeated, tag = "3")]
    pub(super) keys: Vec<String>,
    #[prost(message, repeated, tag = "4")]
    pub(super) values: Vec<Value>,
    #[prost(uint32, optional, tag = "5")]
    pub(super) extent: Option<u32>,
    #[prost(uint32, required, tag = "15")]
    pub(super) version: u32,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct Feature {
    #[prost(uint32, repeated, packed = "true", tag = "2")]
    pub(super) tags: Vec<u32>,
    #[prost(uint32, repeated, packed = "true", tag = "4")]
    pub(super) geometry: Vec<u32>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct Value {
    #[prost(uint32, optional, tag = "5")]
    uint_value: Option<u32>,
    #[prost(sint64, optional, tag = "6")]
    sint_value: Option<i64>,
    #[prost(bool, optional, tag = "7")]
    bool_value: Option<bool>,
}

pub(super) fn decode(bytes: &[u8]) -> Result<Tile, String> {
    let mut decoded = Vec::new();
    GzDecoder::new(bytes)
        .read_to_end(&mut decoded)
        .map_err(|error| format!("could not decompress vector tile: {error}"))?;
    Tile::decode(decoded.as_slice())
        .map_err(|error| format!("could not decode vector tile: {error}"))
}

pub(super) fn decode_geometry(commands: &[u32]) -> Vec<Vec<(i32, i32)>> {
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

pub(super) fn unsigned_property(layer: &Layer, feature: &Feature, name: &str) -> Option<u32> {
    feature.tags.chunks_exact(2).find_map(|tag| {
        let key = layer.keys.get(tag[0] as usize)?;
        (key == name).then(|| {
            let value = layer.values.get(tag[1] as usize)?;
            value
                .uint_value
                .or_else(|| value.sint_value.and_then(|value| u32::try_from(value).ok()))
        })?
    })
}

pub(super) fn boolean_property(layer: &Layer, feature: &Feature, name: &str) -> Option<bool> {
    feature.tags.chunks_exact(2).find_map(|tag| {
        let key = layer.keys.get(tag[0] as usize)?;
        (key == name).then(|| {
            let value = layer.values.get(tag[1] as usize)?;
            value
                .bool_value
                .or_else(|| value.uint_value.map(|value| value != 0))
                .or_else(|| value.sint_value.map(|value| value != 0))
        })?
    })
}

fn zigzag(value: u32) -> i32 {
    ((value >> 1) as i32) ^ -((value & 1) as i32)
}
