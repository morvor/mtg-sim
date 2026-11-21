//! CR 612.10: a splice ability is a text-changing effect that adds the rules text of the
//! card with splice to the spell, after the spell's own text.

use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn spell_abilities(t: &TestGame, id: ObjectId) -> Vec<String> {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Spell(_)))
        .map(|a| a.text.to_string())
        .collect()
}

#[test]
fn splice_adds_the_cards_text_after_the_spells_own() {
    // CR 612.10: Glacial Ray ("Glacial Ray deals 2 damage to any target. Splice onto
    // Arcane {1}{R}") spliced onto Lava Spike ("Lava Spike deals 3 damage to target player
    // or planeswalker"): the spell gains Glacial Ray's text after its own, and its own text
    // is unchanged.
    cr!("612.10");
    ruling!(
        "Glacial Ray",
        "You choose all targets for the spell after revealing cards you want to splice"
    );
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let spike = t.hand(P0, "Lava Spike");
    let ray = t.hand(P0, "Glacial Ray");
    assert!(t
        .obj_now(ray)
        .chars
        .keywords()
        .any(|k| k.kind == KeywordKind::Splice));
    let own = spell_abilities(&t, spike);
    assert_eq!(own.len(), 1);
    let spell = t.cast(P0, spike).kicked(true).target(P1).target(bear).go();
    let abilities = spell_abilities(&t, spell);
    assert_eq!(abilities.len(), 2);
    // The spell's own text comes first, unchanged; the spliced text follows it.
    assert_eq!(abilities[0], own[0]);
    assert!(abilities[1].contains("2 damage"), "{abilities:?}");
    let o = t.obj_now(spell);
    assert!(o.chars.rules_text.starts_with("Lava Spike deals 3 damage"));
    assert!(o.chars.rules_text.contains("Glacial Ray deals 2 damage"));
    // Only text is gained (CR 702.47c): the name and mana cost stay the spell's own.
    assert_eq!(o.chars.name, "Lava Spike");
    assert_eq!(o.chars.mana_cost.as_ref().unwrap().to_string(), "{R}");
    // The splice cost was paid as an additional cost; the card stays in hand.
    assert!(t.in_hand(P0, "Glacial Ray"));
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // The spell loses the spliced text once it leaves the stack (CR 702.47e).
    let gy = t.g.find_in_zone(Zone::Graveyard(P0), "Lava Spike")[0];
    assert_eq!(spell_abilities(&t, gy), own);
}

#[test]
fn splice_only_onto_the_named_quality_and_only_if_payable() {
    // CR 612.10 (with 702.47a): Glacial Ray can be spliced only onto an Arcane spell, and
    // only if its splice cost can be paid.
    cr!("612.10");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    t.hand(P0, "Glacial Ray");
    let bolt = t.hand(P0, "Lightning Bolt");
    let spell = t.cast(P0, bolt).kicked(true).target(P1).go();
    assert_eq!(spell_abilities(&t, spell).len(), 1);
    t.resolve();
    assert_eq!(t.life(P1), 17);
    t.clear_answers();
    // Two Mountains left: Lava Spike ({R}) can be cast, but not with the {1}{R} splice
    // cost as well, so splicing isn't offered.
    let spike = t.hand(P0, "Lava Spike");
    let spell = t.cast(P0, spike).kicked(true).target(P1).go();
    assert_eq!(spell_abilities(&t, spell).len(), 1);
    t.resolve();
    assert_eq!(t.life(P1), 14);
}
