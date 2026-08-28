use super::{
    COUNTRY_COUNT, CountryGeometry, GeomWithData, ProximityMatrices, RTree, TerritorialPolygon,
    densify_ring, ecef, inverse_distance_m, normalize_polygon, pair_proximity, unwrap_longitude,
    validate,
};
use geographiclib_rs::{Geodesic, InverseGeodesic};

#[test]
fn wraps_antimeridian_longitudes_to_the_short_path() {
    assert_eq!(unwrap_longitude(-179.0, 179.0), 181.0);
}

#[test]
fn densifies_long_boundary_segments() {
    let ring = vec![vec![0.0, 0.0], vec![1.0, 0.0]];
    assert!(densify_ring(&ring).expect("valid segment").len() > 20);
}

#[test]
fn measures_a_wgs84_equatorial_degree() {
    let distance = inverse_distance_m(
        super::Point {
            longitude: 0.0,
            latitude: 0.0,
        },
        super::Point {
            longitude: 1.0,
            latitude: 0.0,
        },
    );
    assert!((distance - 111_319.490_793).abs() < 0.01);
}

#[test]
fn matches_geographiclib_for_a_long_reference_path() {
    let first = super::Point {
        longitude: -118.288_423_7,
        latitude: 34.095_925,
    };
    let second = super::Point {
        longitude: 24.734_164_9,
        latitude: 59.432_343_9,
    };
    let reference: f64 = Geodesic::wgs84().inverse(
        first.latitude,
        first.longitude,
        second.latitude,
        second.longitude,
    );
    assert!((inverse_distance_m(first, second) - reference).abs() < 0.01);
}

#[test]
fn ecef_places_equatorial_prime_meridian_on_the_x_axis() {
    let point = ecef(super::Point {
        longitude: 0.0,
        latitude: 0.0,
    });
    assert!(point[0] > 6_000_000.0);
    assert!(point[1].abs() < 0.001);
    assert!(point[2].abs() < 0.001);
}

#[test]
fn rejects_asymmetric_or_invalid_matrices() {
    let mut matrices = ProximityMatrices {
        country_count: COUNTRY_COUNT,
        distances_km: vec![0; COUNTRY_COUNT * COUNTRY_COUNT],
        adjacency: vec![false; COUNTRY_COUNT * COUNTRY_COUNT],
        boundary_point_count: 0,
    };
    matrices.distances_km[1] = 1;
    assert!(validate(&matrices).is_err());
}

#[test]
fn detects_shared_edges_and_antimeridian_contacts() {
    let west = country("west", vec![square(0.0, 0.0, 1.0, 1.0)]);
    let east = country("east", vec![square(1.0, 0.0, 2.0, 1.0)]);
    assert_eq!(
        pair_proximity(&west, &east).expect("valid territories"),
        (true, 0)
    );

    let east_of_dateline = country("east", vec![square(179.0, 0.0, 180.0, 1.0)]);
    let west_of_dateline = country("west", vec![square(-180.0, 0.0, -179.0, 1.0)]);
    assert_eq!(
        pair_proximity(&east_of_dateline, &west_of_dateline).expect("valid territories"),
        (true, 0)
    );
}

#[test]
fn uses_the_nearest_archipelago_component() {
    let archipelago = country(
        "archipelago",
        vec![square(0.0, 0.0, 1.0, 1.0), square(100.0, 0.0, 101.0, 1.0)],
    );
    let neighbor = country("neighbor", vec![square(1.1, 0.0, 2.0, 1.0)]);
    let (_, distance) = pair_proximity(&archipelago, &neighbor).expect("valid territories");
    assert!(distance < 20);
}

#[test]
fn calculates_finite_proximity_near_a_pole() {
    let first = country("first", vec![square(0.0, 89.0, 1.0, 89.5)]);
    let second = country("second", vec![square(2.0, 89.0, 3.0, 89.5)]);
    let (adjacent, distance) = pair_proximity(&first, &second).expect("valid territories");
    assert!(!adjacent);
    assert!(distance <= 1);
}

fn square(
    minimum_longitude: f64,
    minimum_latitude: f64,
    maximum_longitude: f64,
    maximum_latitude: f64,
) -> Vec<Vec<f64>> {
    vec![
        vec![minimum_longitude, minimum_latitude],
        vec![maximum_longitude, minimum_latitude],
        vec![maximum_longitude, maximum_latitude],
        vec![minimum_longitude, maximum_latitude],
        vec![minimum_longitude, minimum_latitude],
    ]
}

fn country(name: &str, rings: Vec<Vec<Vec<f64>>>) -> CountryGeometry {
    let polygons = rings
        .iter()
        .map(|ring| normalize_polygon(&[ring.clone()]).expect("valid polygon"))
        .collect::<Vec<TerritorialPolygon>>();
    let boundary_points = rings
        .iter()
        .flat_map(|ring| densify_ring(ring).expect("valid boundary"))
        .collect::<Vec<_>>();
    let index = RTree::bulk_load(
        boundary_points
            .iter()
            .copied()
            .map(|point| GeomWithData::new(ecef(point), point))
            .collect(),
    );
    CountryGeometry {
        name: name.to_owned(),
        polygons,
        boundary_points,
        index,
    }
}
