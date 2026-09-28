//! Rulings batch S29 — effects that set a creature's base power and toughness (layer 7b)
//! and the effects and counters that modify it (layers 7c and 7d, CR 613.4): the
//! modifications apply no matter when they started to take effect (CR 613.4, 613.7).

use crate::r_s01_common::supported;
use crate::r_s06_common::attach_new;
use crate::r_s29_common::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0's Grizzly Bears (2/2) with a +1/+1 counter, pumped by the real spell `pump` and
/// then switched by Twisted Image ("Switch target creature's power and toughness until
/// end of turn. Draw a card.") if `switch`.
fn modified_bears(t: &mut TestGame, pump: &str, switch: bool) -> ObjectId {
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_counters(t, bears, counters::PLUS1, 1);
    cast_and_resolve(t, P0, pump, &[Entity::Object(bears)]);
    if switch {
        supported("Twisted Image");
        cast_and_resolve(t, P0, "Twisted Image", &[Entity::Object(bears)]);
    }
    bears
}

#[test]
fn retro_mutations_base_0_1_keeps_earlier_pumps_counters_and_switches() {
    cr!("613.4", "613.4b", "613.4c", "613.4d", "613.7");
    ruling!(
        "Retro-Mutation",
        "Effects that modify the creature's power and/or toughness, such as the effect of Karai's Technique, will apply to the creature no matter when they started to take effect. The same is true for counters that change its power and/or toughness and effects that switch its power and toughness."
    );
    supported("Retro-Mutation");
    // "Enchanted creature is a Turtle with base power and toughness 0/1. It can't attack
    // and loses all abilities."
    let mut t = TestGame::new(2);
    // 2/2 + counter + Giant Growth = 6/6, switched (still 6/6).
    let bears = modified_bears(&mut t, "Giant Growth", true);
    assert_eq!(t.pt(bears), (6, 6));
    cast_and_resolve(&mut t, P0, "Retro-Mutation", &[Entity::Object(bears)]);
    assert!(t.obj_now(bears).chars.has_subtype("Turtle"));
    // Base 0/1, +1/+1 counter, +3/+3 from Giant Growth: 4/5, then switched: 5/4.
    assert_eq!(t.pt(bears), (5, 4));
    // Mind Transfer Protocol: "Until end of turn, target artifact or creature becomes an
    // artifact creature with base power and toughness 4/5."
    supported("Mind Transfer Protocol");
    let mut t = TestGame::new(2);
    let bears = modified_bears(&mut t, "Giant Growth", false);
    cast_and_resolve(
        &mut t,
        P0,
        "Mind Transfer Protocol",
        &[Entity::Object(bears)],
    );
    assert!(t.obj_now(bears).chars.is(CardType::Artifact));
    assert_eq!(t.pt(bears), (4 + 1 + 3, 5 + 1 + 3));
}

#[test]
fn will_kenriths_base_0_3_keeps_earlier_pumps_and_counters() {
    cr!("613.4", "613.4b", "613.4c", "613.4d");
    ruling!(
        "Will Kenrith",
        "Effects that raise or lower a creature's power and/or toughness, such as the effect of Titanic Growth, will apply to the creature no matter when they started to take effect. The same is true for any counters that change its power and/or toughness and effects that switch its power and toughness."
    );
    supported("Will Kenrith");
    supported("Titanic Growth");
    // Will Kenrith's +2: "Until your next turn, up to two target creatures each have base
    // power and toughness 0/3 and lose all abilities."
    let mut t = TestGame::new(2);
    let bears = modified_bears(&mut t, "Titanic Growth", true);
    // 2/2 + 1/1 + 4/4 = 7/7, switched.
    assert_eq!(t.pt(bears), (7, 7));
    let will = t.battlefield(P0, "Will Kenrith");
    t.activate(P0, will, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    // Base 0/3 + 1/1 + 4/4 = 5/8, switched: 8/5.
    assert_eq!(t.pt(bears), (8, 5));
}

#[test]
fn startling_developments_base_4_4_keeps_earlier_pumps_and_counters() {
    cr!("613.4b", "613.4c", "613.4d");
    ruling!(
        "Startling Development",
        "Effects that modify the affected creature’s power or toughness without setting it will apply no matter when they started to take effect. The same is true for counters that change the creature’s power or toughness."
    );
    supported("Startling Development");
    // "Until end of turn, target creature becomes a blue Serpent with base power and
    // toughness 4/4."
    let mut t = TestGame::new(2);
    let bears = modified_bears(&mut t, "Giant Growth", false);
    cast_and_resolve(
        &mut t,
        P0,
        "Startling Development",
        &[Entity::Object(bears)],
    );
    assert!(t.obj_now(bears).chars.has_subtype("Serpent"));
    assert_eq!(t.pt(bears), (4 + 1 + 3, 4 + 1 + 3));
    // A counter put on it afterwards applies too.
    put_counters(&mut t, bears, counters::PLUS1, 1);
    assert_eq!(t.pt(bears), (9, 9));
}

#[test]
fn wrecking_ball_arms_base_7_7_keeps_earlier_pumps_counters_and_switches() {
    cr!("613.4", "613.4b", "613.4c", "613.4d", "613.7");
    ruling!(
        "Wrecking Ball Arm",
        "Effects that modify the creature's power and/or toughness without setting base power and/or toughness will apply to the creature no matter when they started to take effect. The same is true for counters that change its power and/or toughness and effects that switch its power and toughness."
    );
    supported("Wrecking Ball Arm");
    // "Equipped creature has base power and toughness 7/7 ..." Giant Growth (+3/+3) then
    // a -1/-1 counter make the difference visible through the switch.
    let mut t = TestGame::new(2);
    let bears = modified_bears(&mut t, "Giant Growth", true);
    put_counters(&mut t, bears, counters::MINUS1, 1);
    // 2/2 + 1/1 + 3/3 - 1/1 = 5/5.
    assert_eq!(t.pt(bears), (5, 5));
    attach_new(&mut t, P0, "Wrecking Ball Arm", bears);
    // Base 7/7, counters (+1/+1 and -1/-1), +3/+3: 10/10.
    assert_eq!(t.pt(bears), (10, 10));
    // Eye of Nidhogg: "Enchanted creature is a black Dragon with base power and
    // toughness 4/2 ...": base 4/2 + 3/3 = 7/5, switched: 5/7.
    supported("Eye of Nidhogg");
    let mut t = TestGame::new(2);
    let bears = modified_bears(&mut t, "Giant Growth", true);
    put_counters(&mut t, bears, counters::MINUS1, 1);
    attach_new(&mut t, P0, "Eye of Nidhogg", bears);
    assert_eq!(t.pt(bears), (5, 7));
}

#[test]
fn a_land_creature_with_three_counters_becomes_a_7_7() {
    cr!("613.4b", "613.4c", "613.1d", "701.66a");
    ruling!(
        "Elemental Uprising",
        "You may target a land that’s already a creature. For example, if you target a land that’s also a 0/0 creature and has three +1/+1 counters on it, the resulting land creature will be 7/7."
    );
    supported("Elemental Uprising");
    supported("Vengeant Earth");
    supported("Sandbenders' Storm");
    for spell in ["Elemental Uprising", "Vengeant Earth"] {
        let mut t = TestGame::new(2);
        let land = t.battlefield(P0, "Wastes");
        // Sandbenders' Storm, second mode: "Earthbend 3." (a 0/0 land creature with three
        // +1/+1 counters).
        choose_modes(&mut t, P0, &[1]);
        cast_and_resolve(&mut t, P0, "Sandbenders' Storm", &[Entity::Object(land)]);
        assert_eq!(t.pt(land), (3, 3));
        // "Target land you control becomes a 4/4 Elemental creature ...".
        cast_and_resolve(&mut t, P0, spell, &[Entity::Object(land)]);
        let o = t.obj_now(land);
        assert!(o.chars.is(CardType::Land) && o.chars.is(CardType::Creature));
        assert!(o.chars.has_subtype("Elemental"));
        assert_eq!(t.pt(land), (7, 7), "{spell}");
    }
}
