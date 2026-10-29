//! Rulings batch P217 — incubate (CR 701.53): a spell whose only target is illegal as it
//! tries to resolve doesn't resolve, so nobody incubates (CR 608.2b).

use crate::r_s01_common::{give_mana_for, supported};
use crate::r_s05_common::{move_to, tokens_with_subtype};
use crate::r_s07_common::resolved;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// P0 casts `name` targeting P1's Hill Giant, which is returned to its owner's hand in
/// response: the spell doesn't resolve and no Incubator token is created. Then again with
/// the Giant staying: it resolves and `incubator_for` gets an Incubator token with
/// `n` +1/+1 counters.
fn fizzles_without_incubating(name: &str, incubator_for: PlayerId, n: u32) {
    supported(name);
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    give_mana_for(&mut t, P0, name);
    let card = t.hand(P0, name);
    let spell = t.cast(P0, card).target(giant).go();
    move_to(&mut t, giant, Zone::Hand(P1));
    t.resolve_all();
    assert!(!resolved(&t, spell));
    assert!(tokens_with_subtype(&t, P0, "Incubator").is_empty());
    assert!(tokens_with_subtype(&t, P1, "Incubator").is_empty());
    // With the target still legal, the spell resolves and incubates.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    give_mana_for(&mut t, P0, name);
    let card = t.hand(P0, name);
    t.cast(P0, card).target(giant).go();
    t.resolve_all();
    assert_eq!(t.zone(giant), Zone::Exile);
    let incubators = tokens_with_subtype(&t, incubator_for, "Incubator");
    assert_eq!(incubators.len(), 1);
    assert_eq!(t.counters(incubators[0], counters::PLUS1), n);
}

#[test]
fn excise_the_imperfect_with_an_illegal_target_doesnt_incubate() {
    cr!("608.2b", "701.53a");
    ruling!(
        "Excise the Imperfect",
        "If the nonland permanent is an illegal target at the time Excise the Imperfect tries to resolve, it won't resolve and none of its effects will happen. Its controller won't incubate."
    );
    // "Exile target nonland permanent. Its controller incubates X, where X is its mana
    // value." Hill Giant has mana value 4.
    fizzles_without_incubating("Excise the Imperfect", P1, 4);
}

#[test]
fn merciless_repurposing_with_an_illegal_target_doesnt_incubate() {
    cr!("608.2b", "701.53a");
    ruling!(
        "Merciless Repurposing",
        "If the target of Merciless Repurposing is illegal as the spell tries to resolve, it won't resolve and none of its effects will happen. You won't incubate."
    );
    // "Exile target creature. Incubate 3."
    fizzles_without_incubating("Merciless Repurposing", P0, 3);
}
