/// Renderer-facing world map data. Values are encoded identifiers, not domain values.
#[derive(Debug)]
pub struct MapData {
    width: u16,
    height: u16,
    country_count: u16,
    cells: Vec<u16>,
    borders: Vec<u8>,
    anchors: Vec<(u16, u16)>,
}

impl MapData {
    pub(crate) fn new(
        width: u16,
        height: u16,
        country_count: u16,
        cells: Vec<u16>,
        borders: Vec<u8>,
        anchors: Vec<(u16, u16)>,
    ) -> Self {
        Self {
            width,
            height,
            country_count,
            cells,
            borders,
            anchors,
        }
    }
    pub fn dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }
    pub fn country_count(&self) -> u16 {
        self.country_count
    }
    pub fn cell(&self, x: u16, y: u16) -> Option<u16> {
        (x < self.width && y < self.height)
            .then(|| self.cells[usize::from(y) * usize::from(self.width) + usize::from(x)])
    }
    pub fn is_border(&self, x: u16, y: u16) -> Option<bool> {
        (x < self.width && y < self.height)
            .then(|| self.borders[usize::from(y) * usize::from(self.width) + usize::from(x)] != 0)
    }
    pub fn anchor(&self, country_index: u16) -> Option<(u16, u16)> {
        self.anchors.get(usize::from(country_index)).copied()
    }
}
