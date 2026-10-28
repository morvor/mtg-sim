//! Rulings batch P046 — Thrasta, Tempest's Roar's trample over planeswalkers (CR 702.19c)
//! when the planeswalker it attacks is also a creature: excess damage counts from its
//! loyalty, and when that planeswalker blocks it, it must be assigned the greater of its
//! lethal damage and its loyalty before the rest goes to its controller.

use crate::r_s01_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P1's planeswalker `pw`, made an artifact creature with power and toughness equal to
/// its mana value by Mycosynth Lattice and March of the Machines, and P0's Thrasta.
fn creature_planeswalker(pw: &str) -> (TestGame, ObjectId, ObjectId) {
    supported("Thrasta, Tempest's Roar");
    supported("March of the Machines");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mycosynth Lattice");
    t.battlefield(P0, "March of the Machines");
    let w = t.battlefield(P1, pw);
    let thrasta = t.battlefield(P0, "Thrasta, Tempest's Roar");
    t.g.recompute();
    assert!(t.obj_now(w).is(CardType::Creature) && t.obj_now(w).is(CardType::Planeswalker));
    (t, thrasta, w)
}

#[test]
fn excess_damage_counts_from_the_loyalty_of_a_creature_planeswalker() {
    cr!("702.19c", "510.1b");
    ruling!(
        "Thrasta, Tempest's Roar",
        "If a planeswalker that is being attacked by Thrasta is somehow also a creature, Thrasta still deals excess damage based on that planeswalker's loyalty, not its toughness."
    );
    // Garruk Wildspeaker: 4/4 with loyalty 3. Thrasta (7/7) assigns 3 to Garruk, 4 to P1.
    let (mut t, thrasta, garruk) = creature_planeswalker("Garruk Wildspeaker");
    assert_eq!(t.pt(garruk), (4, 4));
    assert_eq!(t.obj_now(garruk).loyalty(), 3);
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![3, 4]));
    t.attack(&[(thrasta, Entity::Object(garruk))], &[]);
    assert_eq!(t.life(P1), 16);
    assert!(t.in_graveyard(P1, "Garruk Wildspeaker"));
}

#[test]
fn a_blocking_creature_planeswalker_needs_the_greater_of_lethal_damage_and_loyalty() {
    cr!("702.19b", "702.19c", "509.1a");
    ruling!(
        "Thrasta, Tempest's Roar",
        "Thrasta's controller must assign at least as much damage to that creature planeswalker equal to its toughness or the number of loyalty counters it has, whichever is greater."
    );
    // Garruk (4/4, loyalty 3) blocks: 4 to Garruk, then 3 to P1.
    let (mut t, thrasta, garruk) = creature_planeswalker("Garruk Wildspeaker");
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![4, 3]));
    t.attack(&[(thrasta, Entity::Object(garruk))], &[(garruk, thrasta)]);
    assert_eq!(t.life(P1), 17);
    assert!(t.in_graveyard(P1, "Garruk Wildspeaker"));
    // Chandra Nalaar (5/5, loyalty 6) blocks: 6 to Chandra, then 1 to P1; 5 isn't enough.
    let (mut t, thrasta, chandra) = creature_planeswalker("Chandra Nalaar");
    assert_eq!((t.pt(chandra), t.obj_now(chandra).loyalty()), ((5, 5), 6));
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![6, 1]));
    t.attack(&[(thrasta, Entity::Object(chandra))], &[(chandra, thrasta)]);
    assert_eq!(t.life(P1), 19);
    assert!(t.in_graveyard(P1, "Chandra Nalaar"));
    // Assigning only its toughness to Chandra isn't allowed: the default assignment (6
    // to Chandra, the rest to P1) is used instead.
    let (mut t, thrasta, chandra) = creature_planeswalker("Chandra Nalaar");
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![5, 2]));
    t.attack(&[(thrasta, Entity::Object(chandra))], &[(chandra, thrasta)]);
    assert_eq!(t.life(P1), 19);
}
