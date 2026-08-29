use crate::domain::{
    CountryId, Game,
    ports::{CountryCatalog, TargetSelector},
};

pub struct StartGame<'a, Catalog, Selector> {
    catalog: &'a Catalog,
    selector: &'a mut Selector,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartGameError {
    EmptyCatalog,
    InvalidTarget(CountryId),
}

impl<'a, Catalog, Selector> StartGame<'a, Catalog, Selector>
where
    Catalog: CountryCatalog,
    Selector: TargetSelector,
{
    pub fn new(catalog: &'a Catalog, selector: &'a mut Selector) -> Self {
        Self { catalog, selector }
    }

    pub fn dispatch(&mut self) -> Result<Game, StartGameError> {
        let eligible = self.catalog.playable();
        if eligible.is_empty() {
            return Err(StartGameError::EmptyCatalog);
        }
        let target = self.selector.select(eligible);
        if !eligible.contains(&target) {
            return Err(StartGameError::InvalidTarget(target));
        }
        Ok(Game::new(target))
    }
}

#[cfg(test)]
mod tests;
