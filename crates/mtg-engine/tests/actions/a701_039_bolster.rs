//! CR 701.39: bolster.

use crate::a701_028_071_common::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn bolster_puts_counters_on_a_creature_with_the_least_toughness() {
    cr!("701.39a");
    ruling!(
        "Dromoka's Gift",
        "You determine which creature to put counters on as the spell or ability that instructs you to bolster resolves."
    );
    supported("Cached Defenses");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let runeclaw = t.battlefield(P0, "Runeclaw Bear");
    // An opponent's creature with less toughness isn't a candidate.
    let theirs = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Forest", 3);
    let spell = t.hand(P0, "Cached Defenses");
    t.cast(P0, spell).go();
    // Chosen as it resolves, among the creatures tied for least toughness.
    choose(&mut t, P0, &[runeclaw]);
    t.resolve_all();
    assert_eq!(t.counters(runeclaw, "+1/+1"), 3);
    assert_eq!(t.counters(bears, "+1/+1"), 0);
    assert_eq!(t.counters(giant, "+1/+1"), 0);
    assert_eq!(t.counters(theirs, "+1/+1"), 0);
    let tied: Vec<Vec<Entity>> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if p == P0 => Some(candidates),
            _ => None,
        })
        .collect();
    assert_eq!(
        tied,
        vec![vec![Entity::Object(bears), Entity::Object(runeclaw)]]
    );
}

#[test]
fn bolster_with_no_creatures_does_nothing() {
    cr!("701.39a");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 3);
    let spell = t.hand(P0, "Cached Defenses");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.counters(theirs, "+1/+1"), 0);
    assert!(t.in_graveyard(P0, "Cached Defenses"));
}
