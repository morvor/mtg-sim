//! Combat damage assignment against the assignments the rules allow (CR 510.1c–d, 702.19):
//! see `mtg_engine::properties::damage`.

use mtg_engine::properties::damage::*;

#[test]
fn the_engine_accepts_exactly_the_legal_combat_damage_assignments() {
    let n = std::env::var("MTG_DAMAGE_CASES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(60);
    let (checked, assignments, violations) = match std::env::var("MTG_DAMAGE_SEED") {
        Ok(s) => match check(s.parse().unwrap()) {
            Ok(c) => (1, c.assignments, vec![]),
            Err(e) => (0, 0, vec![e]),
        },
        Err(_) => check_many(1, n),
    };
    eprintln!("{checked} of {n} combats checked, {assignments} assignments compared");
    for v in &violations {
        eprintln!("{v}");
    }
    assert!(violations.is_empty(), "{} violations", violations.len());
}
