use worldhunt::infrastructure::{random::RandomTargetSelector, tui, world_data::WorldData};

#[test]
fn production_wiring_decodes_the_embedded_asset_and_renders_an_initial_frame() {
    let world_data = WorldData::decode_embedded().expect("embedded world data is valid");
    let mut selector = RandomTargetSelector::seeded(7);

    tui::render_initial_frame(&world_data, &world_data, &mut selector, 100, 30)
        .expect("initial frame renders without a TTY");
}
