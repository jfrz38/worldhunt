use super::{
    HEIGHT, WATER, WIDTH, anchor_in_raster, contains_polygon, find_anchor, mark_transition,
    point_in_ring, project, rasterize,
};
use crate::validation::load_validated_repository;
use std::path::Path;

#[test]
fn recognizes_an_outer_ring_and_a_hole() {
    let polygon = vec![
        vec![
            vec![0.0, 0.0],
            vec![4.0, 0.0],
            vec![4.0, 4.0],
            vec![0.0, 4.0],
            vec![0.0, 0.0],
        ],
        vec![
            vec![1.0, 1.0],
            vec![3.0, 1.0],
            vec![3.0, 3.0],
            vec![1.0, 3.0],
            vec![1.0, 1.0],
        ],
    ];
    assert!(contains_polygon(&polygon, 0.5, 0.5));
    assert!(!contains_polygon(&polygon, 2.0, 2.0));
    assert!(point_in_ring(&polygon[0], 3.5, 3.5));
}

#[test]
fn marks_each_territory_transition_once() {
    let mut borders = vec![0; 4];
    mark_transition(&mut borders, 0, 1, 0, 1);
    mark_transition(&mut borders, 2, 3, WATER, 0);

    assert_eq!(borders, vec![1, 0, 0, 1]);
}

#[test]
fn recognizes_a_polygon_across_the_antimeridian() {
    let ring = vec![
        vec![179.0, 0.0],
        vec![-179.0, 0.0],
        vec![-179.0, 2.0],
        vec![179.0, 2.0],
        vec![179.0, 0.0],
    ];

    assert!(point_in_ring(&ring, 179.5, 1.0));
    assert!(point_in_ring(&ring, -179.5, 1.0));
    assert!(!point_in_ring(&ring, 0.0, 1.0));
}

#[test]
fn projects_wrapped_longitudes_to_the_same_cell() {
    assert_eq!(project(181.0, 0.0), project(-179.0, 0.0));
}

#[test]
fn moves_a_source_anchor_to_the_nearest_owned_raster_cell() {
    let mut cells = vec![WATER; usize::from(WIDTH) * usize::from(HEIGHT)];
    cells[100 * usize::from(WIDTH) + 11] = 7;

    assert_eq!(anchor_in_raster(7, (10, 100), &cells), Some((11, 100)));
}

#[test]
fn anchors_the_largest_connected_component() {
    let mut cells = vec![WATER; usize::from(WIDTH) * usize::from(HEIGHT)];
    for x in 10_u16..15 {
        cells[usize::from(100_u16) * usize::from(WIDTH) + usize::from(x)] = 7;
    }
    for x in 300_u16..302 {
        for y in 100_u16..102 {
            cells[usize::from(y) * usize::from(WIDTH) + usize::from(x)] = 7;
        }
    }

    assert!(matches!(find_anchor(7, &cells), Some((11 | 12, 100))));
}

#[test]
fn treats_components_across_the_antimeridian_as_connected() {
    let mut cells = vec![WATER; usize::from(WIDTH) * usize::from(HEIGHT)];
    cells[usize::from(100_u16) * usize::from(WIDTH)] = 7;
    cells[usize::from(100_u16) * usize::from(WIDTH) + usize::from(WIDTH - 1)] = 7;

    assert!(matches!(find_anchor(7, &cells), Some((0 | 719, 100))));
}

#[test]
fn anchors_russia_in_its_mainland() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repository root exists");
    let validated = load_validated_repository(root).expect("committed data is valid");
    let russia = validated
        .catalog
        .countries
        .iter()
        .position(|country| country.iso3 == "RUS")
        .expect("Russia is catalogued") as u16;
    let (x, y) = rasterize(&validated)
        .expect("rasterizes committed data")
        .anchors[usize::from(russia)];

    assert!(
        (500..=650).contains(&x),
        "unexpected Russia longitude cell {x}"
    );
    assert!(
        (35..=80).contains(&y),
        "unexpected Russia latitude cell {y}"
    );
}

#[test]
fn anchors_pacific_countries_in_their_source_regions() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repository root exists");
    let validated = load_validated_repository(root).expect("committed data is valid");
    let anchors = rasterize(&validated)
        .expect("rasterizes committed data")
        .anchors;

    for (iso3, expected_x, expected_y) in [
        ("NZL", 660..=719, 220..=299),
        ("FJI", 680..=719, 180..=240),
        ("WSM", 0..=50, 180..=240),
    ] {
        let country = validated
            .catalog
            .countries
            .iter()
            .position(|country| country.iso3 == iso3)
            .expect("country is catalogued");
        let (x, y) = anchors[country];
        assert!(
            expected_x.contains(&x),
            "unexpected {iso3} longitude cell {x}"
        );
        assert!(
            expected_y.contains(&y),
            "unexpected {iso3} latitude cell {y}"
        );
    }
}
