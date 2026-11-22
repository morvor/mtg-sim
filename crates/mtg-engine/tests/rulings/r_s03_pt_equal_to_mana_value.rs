//! "[Artifact] becomes / is an artifact creature with power and toughness each equal to its
//! mana value" (Karn's Touch, March of the Machines, Animate Artifact): the phrase compiled
//! for the rulings S03 work (living metal and March of the Machines), in one-shot effects
//! and in static abilities.

use crate::r_s01_common::*;
use crate::r_s02_common::{can_activate, can_attack};
use crate::r_s03_common::in_hand_with_mana;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// P0 casts Karn's Touch ("Target noncreature artifact becomes an artifact creature with
/// power and toughness each equal to its mana value until end of turn.") on `target`.
fn karns_touch(t: &mut TestGame, target: ObjectId) {
    supported("Karn's Touch");
    let touch = in_hand_with_mana(t, P0, "Karn's Touch");
    t.cast(P0, touch).target(target).go();
    t.resolve_all();
    t.g.recompute();
}

#[test]
fn karns_touch_makes_a_creature_with_its_mana_value_subject_to_summoning_sickness() {
    cr!("613.4b", "205.1b", "302.6");
    ruling!(
        "Karn's Touch",
        "A noncreature permanent that turns into a creature can attack, and its {T} abilities can be activated, only if its controller has continuously controlled that permanent since the beginning of their most recent turn. It doesn’t matter how long the permanent has been a creature."
    );
    // Mind Stone ({2}): "{T}: Add {C}." and "{1}, {T}, Sacrifice this artifact: Draw a
    // card." P0 has controlled it since the turn began: a 2/2 artifact creature that can
    // attack.
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "Mind Stone");
    karns_touch(&mut t, stone);
    assert!(t.obj(stone).is(CardType::Creature) && t.obj(stone).is(CardType::Artifact));
    assert_eq!(t.pt(stone), (2, 2));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, stone));
    // One that came under P0's control this turn: its {T} ability could be activated
    // while it wasn't a creature, but not once it is one, and it can't attack.
    let mut t = TestGame::new(2);
    let stone = t.battlefield_sick(P0, "Mind Stone");
    let wastes = t.battlefield(P0, "Wastes");
    assert!(can_activate(&mut t, P0, stone));
    karns_touch(&mut t, stone);
    assert!(!t.obj(wastes).tapped);
    assert_eq!(t.pt(stone), (2, 2));
    assert!(!can_activate(&mut t, P0, stone));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, stone));
    // The effect ends at end of turn.
    t.advance_to(P1, Step::Upkeep);
    t.g.recompute();
    assert!(!t.obj(stone).is(CardType::Creature));
}

#[test]
fn march_of_the_machines_keeps_other_types_and_artifact_lands_are_0_0() {
    cr!("613.4b", "205.1b", "704.5f");
    ruling!(
        "March of the Machines",
        "If a noncreature artifact is also another card type, such as enchantment or land, it will retain those types in addition to being an artifact creature."
    );
    ruling!(
        "March of the Machines",
        "Each artifact land has a mana value of 0. March of the Machines makes them 0/0 creatures, which are put into the graveyard as a state-based action."
    );
    supported("March of the Machines");
    // "Each noncreature artifact is an artifact creature with power and toughness each
    // equal to its mana value." Bow of Nylea ({1}{G}{G}, legendary enchantment artifact),
    // Mind Stone ({2}) and Seat of the Synod (artifact land).
    let mut t = TestGame::new(2);
    let bow = t.battlefield(P0, "Bow of Nylea");
    let seat = t.battlefield(P1, "Seat of the Synod");
    let stone = t.battlefield(P1, "Mind Stone");
    t.battlefield(P0, "March of the Machines");
    t.settle();
    let o = t.obj(bow);
    assert!(o.is(CardType::Creature) && o.is(CardType::Artifact) && o.is(CardType::Enchantment));
    assert_eq!(t.pt(bow), (3, 3));
    assert_eq!(t.pt(stone), (2, 2));
    assert!(!t.on_battlefield(seat));
    assert!(t.in_graveyard(P1, "Seat of the Synod"));
}

#[test]
fn animate_artifact_doesnt_change_an_artifact_creature() {
    cr!("613.4b");
    ruling!(
        "Animate Artifact",
        "If it is enchanting an artifact that's already a creature, it won't change its power and toughness."
    );
    supported("Animate Artifact");
    // "As long as enchanted artifact isn't a creature, it's an artifact creature with power
    // and toughness each equal to its mana value." On Mind Stone: a 2/2. On Ornithopter
    // (a 0/2 artifact creature with mana value 0): still 0/2.
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "Mind Stone");
    let thopter = t.battlefield(P0, "Ornithopter");
    for target in [stone, thopter] {
        let aura = in_hand_with_mana(&mut t, P0, "Animate Artifact");
        t.cast(P0, aura).target(target).go();
        t.resolve_all();
    }
    t.g.recompute();
    assert!(t.obj(stone).is(CardType::Creature));
    assert_eq!(t.pt(stone), (2, 2));
    assert!(t.on_battlefield(thopter));
    assert_eq!(t.pt(thopter), (0, 2));
}
