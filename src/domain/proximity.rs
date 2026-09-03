/// Geographic clue between a guess and the target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Proximity {
    distance_km: u16,
    adjacent: bool,
}

impl Proximity {
    pub const fn new(distance_km: u16, adjacent: bool) -> Self {
        Self {
            distance_km,
            adjacent,
        }
    }

    pub const fn distance_km(self) -> u16 {
        self.distance_km
    }

    pub const fn is_adjacent(self) -> bool {
        self.adjacent
    }
}
