//! Irencrag Feat (hand-written, `src/cards/irencrag_feat.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn only_one_more_spell_after_it_resolves() {
    cr!("601.3");
    ruling!("Irencrag Feat", "doesn't care how many spells you've cast so far");
    let mut t = TestGame::new(2);
    // A spell cast earlier this turn doesn't count.
    t.lands(P0, "Mountain", 5);
    let first = t.hand(P0, "Lightning Bolt");
    t.cast(P0, first).target(P1).go();
    t.resolve();
    let feat = t.hand(P0, "Irencrag Feat");
    t.cast(P0, feat).go();
    t.resolve();
    assert_eq!(t.g.player(P0).mana_pool.total(), 7);
    let b1 = t.hand(P0, "Lightning Bolt");
    let b2 = t.hand(P0, "Lightning Bolt");
    t.cast(P0, b1).target(P1).go();
    t.resolve();
    assert!(t.cast(P0, b2).target(P1).try_go().is_err());
    assert_eq!(t.life(P1), 14);
}

#[test]
fn abilities_and_lands_are_not_limited() {
    cr!("601.3");
    ruling!("Irencrag Feat", "doesn't stop you from activating any abilities or from playing lands");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let feat = t.hand(P0, "Irencrag Feat");
    t.cast(P0, feat).go();
    t.resolve();
    let b1 = t.hand(P0, "Lightning Bolt");
    t.cast(P0, b1).target(P1).go();
    t.resolve();
    let land = t.hand(P0, "Mountain");
    assert!(t.play_land(P0, land).is_ok());
}
