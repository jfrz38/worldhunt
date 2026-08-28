//! Read-only access to the embedded, generated map asset.

mod decoder;
mod map_data;

pub use decoder::decode_embedded;
pub use map_data::MapData;
