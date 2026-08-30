use super::{ProximityData, ProximityRecord};

#[test]
fn looks_up_a_symmetric_adjacent_pair() {
    let proximity = ProximityData::new(2, vec![0, 0, 0, 0], vec![false, true, true, false])
        .expect("valid proximity data");

    assert_eq!(
        proximity.between(0, 1),
        Some(ProximityRecord {
            distance_km: 0,
            adjacent: true,
        })
    );
    assert_eq!(proximity.between(2, 0), None);
}

#[test]
fn rejects_invalid_matrix_invariants() {
    assert!(ProximityData::new(2, vec![0, 1, 0, 0], vec![false; 4]).is_err());
    assert!(ProximityData::new(2, vec![0; 4], vec![true, false, false, false]).is_err());
}
