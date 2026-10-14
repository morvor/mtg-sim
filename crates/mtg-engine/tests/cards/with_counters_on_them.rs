//! "with [kind] counters on them" / "with no counters on them" in a plural filter
//! (`parse_with_suffix` in `src/oracle/phrases.rs`): "no" (or a number) isn't a counter
//! kind.

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn damning_verdict_destroys_only_creatures_with_no_counters() {
    let mut t = TestGame::new(2);
    let bare = t.battlefield(P1, "Grizzly Bears");
    let countered = t.battlefield(P1, "Hill Giant");
    t.g.add_counters(Entity::Object(countered), "+1/+1", 1, None);
    assert!(card("Damning Verdict").unsupported_text().is_empty());
    let v = t.hand(P0, "Damning Verdict");
    t.lands(P0, "Plains", 5);
    t.cast(P0, v).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "{:?}", t.zone(bare));
    assert!(!t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn hazardous_conditions_shrinks_only_creatures_with_no_counters() {
    // "Creatures with no counters on them get -2/-2 until end of turn."
    let mut t = TestGame::new(2);
    let bare = t.battlefield(P1, "Hill Giant");
    let countered = t.battlefield(P1, "Craw Wurm");
    t.g.add_counters(Entity::Object(countered), "+1/+1", 1, None);
    assert!(card("Hazardous Conditions").unsupported_text().is_empty());
    let c = t.hand(P0, "Hazardous Conditions");
    t.lands(P0, "Swamp", 4);
    t.lands(P0, "Forest", 4);
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(t.pt(bare), (1, 1));
    assert_eq!(t.pt(countered), (7, 5));
}
