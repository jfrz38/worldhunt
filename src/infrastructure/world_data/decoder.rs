use super::map_data::MapData;

const MAGIC: &[u8; 4] = b"WHMP";
const VERSION: u16 = 1;
const HEADER_LENGTH: usize = 32;
const WATER: u16 = u16::MAX;
const NEUTRAL_LAND: u16 = u16::MAX - 1;

pub fn decode_embedded() -> Result<MapData, String> {
    decode(include_bytes!("../../../assets/world-v1.bin"))
}

fn decode(bytes: &[u8]) -> Result<MapData, String> {
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
    let expected_cells = usize::from(width)
        .checked_mul(usize::from(height))
        .and_then(|count| count.checked_mul(2))
        .ok_or("world map dimensions overflow")?;
    if cells_length != expected_cells
        || borders_length != expected_cells / 2
        || anchors_length != usize::from(country_count) * 4
    {
        return Err("world map asset section lengths are inconsistent".to_owned());
    }
    let end = HEADER_LENGTH
        .checked_add(cells_length)
        .and_then(|offset| offset.checked_add(borders_length))
        .and_then(|offset| offset.checked_add(anchors_length))
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
    Ok(MapData::new(
        width,
        height,
        country_count,
        cells,
        borders,
        anchors,
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
mod tests {
    use super::decode;

    #[test]
    fn rejects_corrupt_assets() {
        assert!(decode(b"WHMP").is_err());
    }

    #[test]
    fn rejects_invalid_versions_lengths_and_identifiers() {
        let original = include_bytes!("../../../assets/world-v1.bin");

        let mut version = original.to_vec();
        version[4] = 2;
        assert!(decode(&version).is_err());

        let mut length = original.to_vec();
        length[17] = 0;
        assert!(decode(&length).is_err());

        let mut identifier = original.to_vec();
        identifier[32] = 250;
        identifier[33] = 0;
        assert!(decode(&identifier).is_err());
    }
    #[test]
    fn decodes_the_committed_asset() {
        assert_eq!(
            super::decode_embedded()
                .expect("asset should decode")
                .dimensions(),
            (720, 300)
        );
    }
}
