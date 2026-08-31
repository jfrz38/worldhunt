use super::{
    INITIAL_ZOOM, Map, MapState, NORTH, SOUTH, SPAIN_CENTER_X, SPAIN_CENTER_Y, dominant_country,
    fill_geographic_country, status_line, visible_countries, visible_rows,
};
use crate::domain::{CountryId, Game, Proximity};
use crate::infrastructure::tui::theme::{ColorMode, Theme};
use ratatui::{Terminal, backend::TestBackend, style::Color};

#[test]
fn loads_centered_on_spain_at_zoom_one() {
    let map = Map::load().expect("map assets should load");

    assert_eq!(map.zoom, INITIAL_ZOOM);
    assert_eq!(map.center_x, SPAIN_CENTER_X);
    assert_eq!(map.center_y, SPAIN_CENTER_Y);
}

#[test]
fn chooses_the_most_represented_country_in_a_braille_cell() {
    let countries = [1, 1, 1, 2, 1, 2, 3, 3];

    assert_eq!(dominant_country(&countries, 2, 0, 0), 1);
}

#[test]
fn breaks_country_ties_by_stable_catalog_index() {
    let countries = [4, 4, 4, 4, 2, 2, 2, 2];

    assert_eq!(dominant_country(&countries, 2, 0, 0), 2);
}

#[test]
fn identifies_only_countries_that_survive_braille_downsampling() {
    let countries = [1, 1, 1, 1, 1, 1, 1, 2];

    assert_eq!(
        visible_countries(&countries, 2, 1, 1, 3),
        vec![false, true, false]
    );
}

#[test]
fn preserves_a_country_sample_when_the_other_braille_dots_are_water() {
    let countries = [
        7,
        u16::MAX,
        u16::MAX,
        u16::MAX,
        u16::MAX,
        u16::MAX,
        u16::MAX,
        u16::MAX,
    ];

    assert_eq!(dominant_country(&countries, 2, 0, 0), 7);
}

#[test]
fn geographic_details_preserve_their_country_identity() {
    let mut countries = vec![u16::MAX; 16];
    let polygon = [(-2.0, -2.0), (2.0, -2.0), (2.0, 2.0), (-2.0, 2.0)];

    fill_geographic_country(&mut countries, 4, 4, &polygon, 7, 0.5, 0.5, 100.0);

    assert!(countries.contains(&7));
}

#[test]
fn canary_details_rasterize_with_spains_country_identity() {
    let details =
        super::MapDetails::decode(include_bytes!("../../../../assets/map-details-v1.bin"))
            .expect("map details decode");
    let spain = details.islands[0].country;
    let mut dots = vec![0; 200 * 100];
    let mut countries = vec![u16::MAX; 200 * 100];

    details.draw(
        &mut dots,
        &mut countries,
        super::Viewport {
            width: 200,
            height: 100,
            center_x: 0.455,
            center_y: 0.418,
            scale: 10_000.0,
        },
    );

    assert!(countries.contains(&spain));
    assert!((0..25).any(|row| {
        (0..100).any(|column| dominant_country(&countries, 200, row, column) == spain)
    }));
}

#[test]
fn navigation_clamps_latitude_wraps_longitude_and_bounds_zoom() {
    let mut map = Map::load().expect("map assets should load");

    for _ in 0..16 {
        map.zoom_in();
    }
    assert_eq!(map.zoom, 1.99);
    map.pan(0.0, -100.0);
    assert_eq!(map.center_y, NORTH);
    map.pan(100.0, 0.0);
    assert!((0.0..1.0).contains(&map.center_x));

    for _ in 0..16 {
        map.zoom_out();
    }
    assert_eq!(map.zoom, 0.0);
    map.pan(0.0, 100.0);
    assert_eq!(map.center_y, SOUTH);
}

#[test]
fn centering_uses_an_anchor_without_changing_zoom() {
    let mut map = Map::load().expect("map assets should load");
    let zoom = map.zoom;

    assert!(map.center_on(CountryId::new(0)));

    assert_eq!(map.zoom, zoom);
}

#[test]
fn status_includes_openstreetmap_attribution() {
    assert!(status_line(INITIAL_ZOOM).contains("OpenStreetMap contributors"));
}

#[test]
fn skips_polygons_outside_the_viewport() {
    let rings = [vec![(20, 20), (30, 20), (30, 30), (20, 30), (20, 20)]];

    assert_eq!(visible_rows(&rings, 10, 10), None);
}

#[test]
fn rasterizes_only_the_rows_covered_by_a_polygon() {
    let rings = [vec![(2, 3), (7, 3), (7, 8), (2, 8), (2, 3)]];

    assert_eq!(visible_rows(&rings, 10, 10), Some(3..9));
}

#[test]
fn renders_a_deterministic_game_state_with_test_backend() {
    let map = Map::load().expect("map assets should load");
    let mut game = Game::new(CountryId::new(0));
    game.submit(CountryId::new(0), Proximity::new(0, false))
        .expect("target guess is accepted");
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).expect("test terminal starts");
    let theme = Theme::new(ColorMode::Ansi256);

    terminal
        .draw(|frame| {
            map.render_with_guesses(frame.area(), frame.buffer_mut(), game.guesses(), theme)
        })
        .expect("first frame renders");
    let first = terminal.backend().buffer().clone();
    terminal
        .draw(|frame| {
            map.render_with_guesses(frame.area(), frame.buffer_mut(), game.guesses(), theme)
        })
        .expect("second frame renders");

    assert_eq!(first, *terminal.backend().buffer());
    assert!(first.content().iter().any(|cell| cell.symbol() != " "));
    assert!(
        first
            .content()
            .iter()
            .any(|cell| cell.style().bg == Some(Color::Indexed(35)))
    );
}

#[test]
fn displays_resize_message_below_minimum_size() {
    let map = Map::load().expect("map assets should load");
    let mut terminal = Terminal::new(TestBackend::new(19, 7)).expect("test terminal starts");

    terminal
        .draw(|frame| map.render(frame.area(), frame.buffer_mut()))
        .expect("small frame renders");

    let text = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    assert!(text.contains("Resize terminal"));
}

#[test]
fn keeps_every_accepted_guess_in_the_presentation_state() {
    let mut game = Game::new(CountryId::new(2));
    game.submit(CountryId::new(0), Proximity::new(4_000, false))
        .expect("first guess is accepted");
    game.submit(CountryId::new(1), Proximity::new(500, false))
        .expect("second guess is accepted");

    let state = MapState::from_guesses(game.guesses(), 196, Theme::new(ColorMode::Ansi256));

    assert!(state.country_styles[0].is_some());
    assert!(state.country_styles[1].is_some());
}
