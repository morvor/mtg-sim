//! Rulings batch P217 — hexproof from [quality] (CR 702.11d, 702.11e): "This permanent
//! can't be the target of [quality] spells your opponents control or abilities your
//! opponents control from [quality] sources."

use crate::r_s01_common::supported;
use crate::r_s04_common::{ability_targets, add_mana, spell_targets};
use crate::r_s06_common::activate_containing;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Whether `p` could target `id` with the spell `spell` (put into `p`'s hand) now.
fn spell_can_target(t: &mut TestGame, p: PlayerId, spell: &str, id: ObjectId) -> bool {
    spell_targets(t, p, spell).contains(&Entity::Object(id))
}

/// Whether the first activated ability of the permanent `name` (put onto the battlefield
/// under `p`'s control) could target `id` now.
fn ability_can_target(t: &mut TestGame, p: PlayerId, name: &str, id: ObjectId) -> bool {
    let source = t.battlefield(p, name);
    ability_targets(t, source, 0).contains(&Entity::Object(id))
}

/// `name` has "hexproof from `color`": the opponent's spells and abilities of that color
/// can't target it, the opponent's others can, and its controller's can.
fn hexproof_from(name: &str, colored_spell: &str, colored_source: &str) {
    supported(name);
    let mut t = TestGame::new(2);
    let it = t.battlefield(P0, name);
    // The opponent's spell / ability of that color can't target it.
    assert!(!spell_can_target(&mut t, P1, colored_spell, it));
    assert!(!ability_can_target(&mut t, P1, colored_source, it));
    // Its spells and abilities of other colors can: Lightning Bolt (red), and Prodigal
    // Sorcerer (blue) or Master Decoy (white), whichever isn't that color.
    assert!(spell_can_target(&mut t, P1, "Lightning Bolt", it));
    let other_source = if colored_source == "Prodigal Sorcerer" {
        "Master Decoy"
    } else {
        "Prodigal Sorcerer"
    };
    assert!(ability_can_target(&mut t, P1, other_source, it));
    // Its controller's spells and abilities of that color can.
    assert!(spell_can_target(&mut t, P0, colored_spell, it));
    assert!(ability_can_target(&mut t, P0, colored_source, it));
}

#[test]
fn hexproof_from_white_is_white_spells_and_abilities_of_white_sources_opponents_control() {
    cr!("702.11d", "702.11e", "115.4");
    ruling!(
        "Knight of Malice",
        "\"Hexproof from white\" means \"This permanent can't be the target of white spells your opponents control or abilities of white sources your opponents control.\""
    );
    // Swords to Plowshares (white spell), Master Decoy ("{W}, {T}: Tap target creature.").
    hexproof_from("Knight of Malice", "Swords to Plowshares", "Master Decoy");
}

#[test]
fn hexproof_from_blue_is_blue_spells_and_abilities_of_blue_sources_opponents_control() {
    cr!("702.11d", "702.11e", "115.4");
    ruling!(
        "Sporeweb Weaver",
        "Hexproof from blue means that Sporeweb Weaver can't be the target of blue spells your opponents control or abilities your opponents control of blue sources."
    );
    // Unsummon (blue spell), Prodigal Sorcerer ("{T}: This creature deals 1 damage to any
    // target.").
    hexproof_from("Sporeweb Weaver", "Unsummon", "Prodigal Sorcerer");
}

#[test]
fn knight_of_malice_gets_plus_one_while_any_player_controls_a_white_permanent() {
    cr!("611.3a", "105.2");
    supported("Knight of Malice");
    supported("Knight of Grace");
    let mut t = TestGame::new(2);
    let malice = t.battlefield(P0, "Knight of Malice");
    let grace = t.battlefield(P1, "Knight of Grace");
    // Each is the other's color: each gets +1/+0.
    assert_eq!(t.pt(malice), (3, 2));
    assert_eq!(t.pt(grace), (3, 2));
    t.g.sacrifice(grace, P1);
    t.g.recompute();
    assert_eq!(t.pt(malice), (2, 2));
    // A white permanent its own controller controls counts too.
    t.battlefield(P0, "Savannah Lions");
    t.g.recompute();
    assert_eq!(t.pt(malice), (3, 2));
}

#[test]
fn losing_hexproof_or_being_targetable_as_though_it_didnt_applies_to_hexproof_from_white() {
    cr!("702.11e", "702.11f", "613.1f");
    ruling!(
        "Knight of Malice",
        "If an effect says that a creature loses hexproof or can be targeted as though it didn't have hexproof, this applies to hexproof from white as well."
    );
    supported("Glaring Spotlight");
    supported("Shadowspear");
    // Glaring Spotlight: "Creatures your opponents control with hexproof can be the
    // targets of spells and abilities you control as though they didn't have hexproof."
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Knight of Malice");
    assert!(!spell_can_target(&mut t, P1, "Swords to Plowshares", knight));
    t.battlefield(P1, "Glaring Spotlight");
    assert!(spell_can_target(&mut t, P1, "Swords to Plowshares", knight));
    assert!(ability_can_target(&mut t, P1, "Master Decoy", knight));
    // Shadowspear: "{1}: Permanents your opponents control lose hexproof and
    // indestructible until end of turn."
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Knight of Malice");
    let spear = t.battlefield(P1, "Shadowspear");
    assert!(!spell_can_target(&mut t, P1, "Swords to Plowshares", knight));
    add_mana(&mut t, P1, ManaType::C, 1);
    activate_containing(&mut t, P1, spear, "lose hexproof").expect("activate");
    t.resolve_all();
    assert!(spell_can_target(&mut t, P1, "Swords to Plowshares", knight));
    assert!(ability_can_target(&mut t, P1, "Master Decoy", knight));
}
