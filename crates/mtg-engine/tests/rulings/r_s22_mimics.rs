//! Rulings batch S22 — the Shadowmoor Mimics: "Whenever you cast a spell that's both red
//! and white, this creature has base power and toughness 4/2 until end of turn and gains
//! first strike until end of turn." (Battlegate Mimic). A spell that's both colors
//! triggers it whatever its other colors (CR 105.2, 603.2); each time it triggers, the
//! new effect has a later timestamp (CR 613.7).

use crate::r_s01_common::*;
use crate::r_s06_common::has_kw;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Casts the real card `name` from P0's hand (with the mana for it), targeting `targets`,
/// and resolves the stack.
fn cast(t: &mut TestGame, name: &str, targets: &[Entity]) {
    give_mana_for(t, P0, name);
    let c = t.hand(P0, name);
    for e in targets {
        t.answer_targets(P0, &[*e]);
    }
    t.cast(P0, c).go();
    t.resolve_all();
    t.clear_answers();
}

#[test]
fn battlegate_mimic_triggers_for_a_spell_thats_both_colors_whatever_its_other_colors() {
    cr!("603.2", "105.2", "613.4b");
    ruling!(
        "Battlegate Mimic",
        "The ability triggers whenever you cast a spell that’s both of its listed colors. It doesn’t matter whether that spell also happens to be any other colors."
    );
    supported("Battlegate Mimic");
    supported("Lightning Helix");
    supported("Naya Charm");
    // A red spell that isn't white: no trigger.
    let mut t = TestGame::new(2);
    let mimic = t.battlefield(P0, "Battlegate Mimic");
    cast(&mut t, "Shock", &[Entity::Player(P1)]);
    assert_eq!(t.pt(mimic), (2, 1));
    assert!(!has_kw(&t, mimic, KeywordKind::FirstStrike));
    // Red and white (Lightning Helix): 4/2 and first strike.
    cast(&mut t, "Lightning Helix", &[Entity::Player(P1)]);
    t.g.recompute();
    assert_eq!(t.pt(mimic), (4, 2));
    assert!(has_kw(&t, mimic, KeywordKind::FirstStrike));
    // Red, green, and white (Naya Charm: "Choose one — • Naya Charm deals 3 damage to
    // target creature. • ..."): it triggers too.
    let mut t = TestGame::new(2);
    let mimic = t.battlefield(P0, "Battlegate Mimic");
    let bears = t.battlefield(P1, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Naya Charm");
    let charm = t.hand(P0, "Naya Charm");
    t.cast(P0, charm)
        .modes(&[0])
        .target(Entity::Object(bears))
        .go();
    t.resolve_all();
    t.g.recompute();
    assert_eq!(t.pt(mimic), (4, 2));
    assert!(has_kw(&t, mimic, KeywordKind::FirstStrike));
}

#[test]
fn battlegate_mimic_triggers_again_and_overwrites_earlier_pt_setting_effects() {
    cr!("603.2", "613.4b", "613.7");
    ruling!(
        "Battlegate Mimic",
        "If you cast a spell that’s the two appropriate colors for the second time in a turn, the ability triggers again. The Mimic will once again become the power and toughness stated in its ability, which could overwrite power- and toughness-setting effects that have been applied to it in the meantime."
    );
    supported("Diminish");
    let mut t = TestGame::new(2);
    let mimic = t.battlefield(P0, "Battlegate Mimic");
    cast(&mut t, "Lightning Helix", &[Entity::Player(P1)]);
    t.g.recompute();
    assert_eq!(t.pt(mimic), (4, 2));
    // Diminish: "Target creature has base power and toughness 1/1 until end of turn."
    cast(&mut t, "Diminish", &[Entity::Object(mimic)]);
    t.g.recompute();
    assert_eq!(t.pt(mimic), (1, 1));
    // A second red and white spell: 4/2 again.
    cast(&mut t, "Lightning Helix", &[Entity::Player(P1)]);
    t.g.recompute();
    assert_eq!(t.pt(mimic), (4, 2));
    assert_eq!(t.life(P1), 14);
}
