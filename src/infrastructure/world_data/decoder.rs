use super::{map_data::MapData, proximity::ProximityData};

const MAGIC: &[u8; 4] = b"WHMP";
const VERSION: u16 = 2;
const HEADER_LENGTH: usize = 36;
const WATER: u16 = u16::MAX;
const NEUTRAL_LAND: u16 = u16::MAX - 1;

pub(super) fn decode_embedded() -> Result<(MapData, ProximityData), String> {
    decode(include_bytes!("../../../assets/world-v2.bin"))
}

fn decode(bytes: &[u8]) -> Result<(MapData, ProximityData), String> {
    if bytes.len() < HEADER_LENGTH || &bytes[0..4] != MAGIC {
        return Err("world map asset has invalid magic or is truncated".to_owned());
    }
    if read_u16(bytes, 4)? != VERSION {
        return Err("world map asset has an unsupported version".to_owned());
    }
    let width = read_u16(bytes, 6)?;
    let height = read_u16(bytes, 8)?;
    let country_count = read_u16(bytes, 10)?;
    if width == 0
        || height == 0
        || country_count == 0
        || read_u16(bytes, 12)? != WATER
        || read_u16(bytes, 14)? != NEUTRAL_LAND
    {
        return Err("world map asset header is invalid".to_owned());
    }
    let cells_length = read_u32(bytes, 16)? as usize;
    let borders_length = read_u32(bytes, 20)? as usize;
    let anchors_length = read_u32(bytes, 24)? as usize;
    let distances_length = read_u32(bytes, 28)? as usize;
    let adjacency_length = read_u32(bytes, 32)? as usize;
    let expected_cells = usize::from(width)
        .checked_mul(usize::from(height))
        .and_then(|count| count.checked_mul(2))
        .ok_or("world map dimensions overflow")?;
    let matrix_entries = usize::from(country_count)
        .checked_mul(usize::from(country_count))
        .ok_or("world proximity country count overflows")?;
    if cells_length != expected_cells
        || borders_length != expected_cells / 2
        || anchors_length != usize::from(country_count) * 4
        || distances_length != matrix_entries * 2
        || adjacency_length != matrix_entries
    {
        return Err("world map asset section lengths are inconsistent".to_owned());
    }
    let end = HEADER_LENGTH
        .checked_add(cells_length)
        .and_then(|offset| offset.checked_add(borders_length))
        .and_then(|offset| offset.checked_add(anchors_length))
        .and_then(|offset| offset.checked_add(distances_length))
        .and_then(|offset| offset.checked_add(adjacency_length))
        .ok_or("world map asset length overflow")?;
    if bytes.len() != end {
        return Err("world map asset has an invalid total length".to_owned());
    }
    let cells = (0..cells_length / 2)
        .map(|index| read_u16(bytes, HEADER_LENGTH + index * 2))
        .collect::<Result<Vec<_>, _>>()?;
    if cells
        .iter()
        .any(|id| *id != WATER && *id != NEUTRAL_LAND && *id >= country_count)
    {
        return Err("world map asset has an unknown country identifier".to_owned());
    }
    let border_start = HEADER_LENGTH + cells_length;
    let borders = bytes[border_start..border_start + borders_length].to_vec();
    if borders.iter().any(|value| *value > 1) {
        return Err("world map asset has an invalid border mask".to_owned());
    }
    let anchor_start = border_start + borders_length;
    let anchors = (0..usize::from(country_count))
        .map(|index| {
            let x = read_u16(bytes, anchor_start + index * 4)?;
            let y = read_u16(bytes, anchor_start + index * 4 + 2)?;
            if x >= width || y >= height {
                return Err("world map asset anchor is out of range".to_owned());
            }
            Ok((x, y))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let distance_start = anchor_start + anchors_length;
    let distances_km = (0..matrix_entries)
        .map(|index| read_u16(bytes, distance_start + index * 2))
        .collect::<Result<Vec<_>, _>>()?;
    let adjacency_start = distance_start + distances_length;
    let adjacency = bytes[adjacency_start..adjacency_start + adjacency_length]
        .iter()
        .map(|value| match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err("world map asset has an invalid adjacency value".to_owned()),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let proximity = ProximityData::new(country_count, distances_km, adjacency)?;
    let self_proximity = proximity
        .between(0, 0)
        .ok_or("world proximity self lookup is missing")?;
    if self_proximity.distance_km != 0 || self_proximity.adjacent {
        return Err("world proximity self lookup is invalid".to_owned());
    }
    Ok((
        MapData::new(width, height, country_count, cells, borders, anchors),
        proximity,
    ))
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, String> {
    bytes
        .get(offset..offset + 2)
        .and_then(|part| part.try_into().ok())
        .map(u16::from_le_bytes)
        .ok_or_else(|| "world map asset is truncated".to_owned())
}
fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    bytes
        .get(offset..offset + 4)
        .and_then(|part| part.try_into().ok())
        .map(u32::from_le_bytes)
        .ok_or_else(|| "world map asset is truncated".to_owned())
}

#[cfg(test)]
mod tests;
