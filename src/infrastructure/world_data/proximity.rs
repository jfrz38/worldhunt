/// Runtime-only matrix lookup; iteration 005 adapts it to the domain port.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ProximityRecord {
    pub(super) distance_km: u16,
    pub(super) adjacent: bool,
}

pub(crate) struct ProximityData {
    country_count: usize,
    distances_km: Vec<u16>,
    adjacency: Vec<bool>,
}

impl ProximityData {
    pub(crate) fn new(
        country_count: u16,
        distances_km: Vec<u16>,
        adjacency: Vec<bool>,
    ) -> Result<Self, String> {
        let country_count = usize::from(country_count);
        let expected_length = country_count
            .checked_mul(country_count)
            .ok_or("world proximity country count overflows")?;
        if distances_km.len() != expected_length || adjacency.len() != expected_length {
            return Err("world proximity matrix dimensions are invalid".to_owned());
        }
        for first in 0..country_count {
            let diagonal = matrix_index(country_count, first, first)?;
            if distances_km[diagonal] != 0 || adjacency[diagonal] {
                return Err("world proximity matrix diagonal is invalid".to_owned());
            }
            for second in first + 1..country_count {
                let forward = matrix_index(country_count, first, second)?;
                let reverse = matrix_index(country_count, second, first)?;
                if distances_km[forward] != distances_km[reverse]
                    || adjacency[forward] != adjacency[reverse]
                {
                    return Err("world proximity matrices are not symmetric".to_owned());
                }
                if adjacency[forward] && distances_km[forward] != 0 {
                    return Err("adjacent world countries must have zero separation".to_owned());
                }
            }
        }
        Ok(Self {
            country_count,
            distances_km,
            adjacency,
        })
    }

    pub(crate) fn between(&self, first: u16, second: u16) -> Option<ProximityRecord> {
        let index =
            matrix_index(self.country_count, usize::from(first), usize::from(second)).ok()?;
        Some(ProximityRecord {
            distance_km: *self.distances_km.get(index)?,
            adjacent: *self.adjacency.get(index)?,
        })
    }
}

fn matrix_index(country_count: usize, first: usize, second: usize) -> Result<usize, String> {
    if first >= country_count || second >= country_count {
        return Err("world proximity country index is out of range".to_owned());
    }
    first
        .checked_mul(country_count)
        .and_then(|offset| offset.checked_add(second))
        .ok_or_else(|| "world proximity matrix index overflows".to_owned())
}

#[cfg(test)]
mod tests;
