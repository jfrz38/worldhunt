fn main() -> std::io::Result<()> {
    let world_data = worldhunt::infrastructure::world_data::WorldData::decode_embedded()
        .map_err(std::io::Error::other)?;
    let mut selector = worldhunt::infrastructure::random::RandomTargetSelector::new();
    worldhunt::infrastructure::tui::run(&world_data, &world_data, &mut selector)
}
