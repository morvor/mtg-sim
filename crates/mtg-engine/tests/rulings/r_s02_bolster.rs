//! Rulings batch S02 — bolster (CR 701.39): "Choose a creature with the least toughness
//! among creatures you control and put N +1/+1 counters on it."

use crate::r_s01_common::*;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// P0 casts the real instant or sorcery `name` (with lands for it) and resolves it.
fn cast_spell(t: &mut TestGame, name: &str, targets: &[Entity]) -> ObjectId {
    supported(name);
    give_mana_for(t, P0, name);
    let c = t.hand(P0, name);
    t.cast(P0, c).targets(targets).go()
}

#[test]
fn a_white_bolster_ability_puts_counters_on_a_creature_with_protection_from_white() {
    cr!("701.39a", "702.16b", "115.1");
    ruling!(
        "Aven Tactician",
        "Bolster itself doesn’t target any creature, though some spells and abilities that bolster may have other effects that target creatures. For example, you could put counters on a creature with protection from white with Aven Tactician’s bolster ability."
    );
    supported("Aven Tactician");
    let mut t = TestGame::new(2);
    // Black Knight: 2/2, first strike, protection from white.
    let knight = t.battlefield(P0, "Black Knight");
    t.battlefield(P0, "Hill Giant");
    // Aven Tactician (a white 2/3): "When this creature enters, bolster 1."
    let tactician = t.enter(P0, "Aven Tactician");
    t.resolve_all();
    assert_eq!(t.counters(knight, counters::PLUS1), 1);
    assert_eq!(t.counters(tactician, counters::PLUS1), 0);
    assert_eq!(t.pt(knight), (3, 3));
}

#[test]
fn a_white_bolster_spell_puts_counters_on_a_creature_with_protection_from_white() {
    cr!("701.39a", "702.16b");
    ruling!(
        "Honor's Reward",
        "Bolster itself doesn’t target any creature, though some spells and abilities that bolster may have other effects that target creatures. For example, you could put counters on a creature with protection from white with Abzan Skycaptain’s bolster ability."
    );
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Black Knight");
    t.battlefield(P0, "Hill Giant");
    // Honor's Reward (white): "You gain 4 life. Bolster 2."
    cast_spell(&mut t, "Honor's Reward", &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.counters(knight, counters::PLUS1), 2);
    assert_eq!(t.pt(knight), (4, 4));
}

#[test]
fn abzan_skycaptains_bolster_puts_counters_on_a_creature_with_protection_from_white() {
    cr!("701.39a", "702.16b");
    ruling!(
        "Abzan Skycaptain",
        "Bolster itself doesn't target any creature, though some spells and abilities that bolster may have other effects that target creatures. For example, you could put counters on a creature with protection from white with Abzan Skycaptain's bolster ability."
    );
    supported("Abzan Skycaptain");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Black Knight");
    t.battlefield(P0, "Hill Giant");
    // Abzan Skycaptain (white 2/2 flier): "When this creature dies, bolster 2."
    let skycaptain = t.battlefield(P0, "Abzan Skycaptain");
    t.g.destroy(skycaptain, None);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Abzan Skycaptain"));
    assert_eq!(t.counters(knight, counters::PLUS1), 2);
    assert_eq!(t.pt(knight), (4, 4));
}

#[test]
fn scale_blessings_bolster_puts_a_counter_on_a_creature_with_protection_from_white() {
    cr!("701.39a", "702.16b");
    ruling!(
        "Scale Blessing",
        "Bolster itself doesn't target any creature, though some spells and abilities that bolster may have other effects that target creatures. For example, you could put counters on a creature with protection from white with Aven Tactician's bolster ability."
    );
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Black Knight");
    let giant = t.battlefield(P0, "Hill Giant");
    // Scale Blessing (white): "Bolster 1, then put a +1/+1 counter on each creature you
    // control with a +1/+1 counter on it."
    cast_spell(&mut t, "Scale Blessing", &[]);
    t.resolve_all();
    assert_eq!(t.counters(knight, counters::PLUS1), 2);
    assert_eq!(t.counters(giant, counters::PLUS1), 0);
}

#[test]
fn the_creature_with_the_least_toughness_is_determined_as_the_spell_resolves() {
    cr!("701.39a", "608.2h");
    ruling!(
        "Honor's Reward",
        "You determine which creature to put counters on as the spell or ability that instructs you to bolster resolves."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    // When Honor's Reward is cast, the Bears have the least toughness.
    cast_spell(&mut t, "Honor's Reward", &[]);
    // In response, the Bears get +3/+3: now the Giant has the least toughness.
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(bears).go();
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::PLUS1), 2);
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
}

#[test]
fn a_bolster_ability_can_put_counters_on_its_own_source() {
    cr!("701.39a", "608.2h");
    ruling!(
        "Anafenza, Kin-Tree Spirit",
        "You determine which creature to put counters on as the spell or ability that instructs you to bolster resolves. That could be the creature with the bolster ability, if it's still under your control and has the least toughness."
    );
    supported("Anafenza, Kin-Tree Spirit");
    let mut t = TestGame::new(2);
    // Anafenza (2/2): "Whenever another nontoken creature you control enters, bolster 1."
    let anafenza = t.battlefield(P0, "Anafenza, Kin-Tree Spirit");
    let courser = t.enter(P0, "Centaur Courser");
    t.resolve_all();
    assert_eq!(t.counters(anafenza, counters::PLUS1), 1);
    assert_eq!(t.counters(courser, counters::PLUS1), 0);
    assert_eq!(t.pt(anafenza), (3, 3));
}
