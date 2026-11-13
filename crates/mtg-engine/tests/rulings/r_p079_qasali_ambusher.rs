//! Rulings batch P079 — Qasali Ambusher: "If a creature is attacking you and you control a
//! Forest and a Plains, you may cast this creature without paying its mana cost and as
//! though it had flash." (an alternative cost with flash, CR 118.9, 601.3c).

use crate::r_p079_common::*;
use crate::r_s01_common::{attack_with, supported};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P1 controls a Forest and a Plains (and no other mana) and has Qasali Ambusher in hand;
/// P0's Grizzly Bears attack `target` (P1 or P1's planeswalker), or nothing attacks.
fn setup(attack: Option<bool>) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    t.lands(P1, "Forest", 1);
    t.lands(P1, "Plains", 1);
    let pw = t.battlefield(P1, "Jace Beleren");
    let bears = t.battlefield(P0, "Grizzly Bears");
    match attack {
        Some(at_player) => {
            let to = if at_player {
                Entity::Player(P1)
            } else {
                obj(pw)
            };
            attack_with(&mut t, &[(bears, to)]);
        }
        None => t.set_step(P0, Step::BeginningOfCombat),
    }
    let q = t.hand(P1, "Qasali Ambusher");
    (t, q)
}

/// The alternative-cost casting methods offered for the card now.
fn alternative_methods(t: &mut TestGame, card: ObjectId) -> Vec<CastMethod> {
    crate::r_s08_common::legal_cast_methods(t, P1, card)
        .into_iter()
        .filter(|m| matches!(m, CastMethod::Alternative(_)))
        .collect()
}

#[test]
fn qasali_ambusher_needs_a_creature_attacking_you() {
    cr!("118.9", "601.3c", "506.2");
    ruling!(
        "Qasali Ambusher",
        "For you to cast Qasali Ambusher as described in its second ability, a creature has to be attacking *you*. If creatures are attacking only planeswalkers, you won’t be able to cast Qasali Ambusher this way."
    );
    supported("Qasali Ambusher");
    let (mut t, q) = setup(Some(true));
    assert!(
        can_cast_now(&mut t, P1, q),
        "attacked: free, at instant speed"
    );
    let (mut t, q) = setup(Some(false));
    assert!(
        !can_cast_now(&mut t, P1, q),
        "only the planeswalker is attacked"
    );
    assert!(alternative_methods(&mut t, q).is_empty());
    let (mut t, q) = setup(None);
    assert!(!can_cast_now(&mut t, P1, q), "nothing attacking");
}

#[test]
fn qasali_ambusher_cast_this_way_uses_the_stack() {
    cr!("601.2", "701.6a");
    ruling!(
        "Qasali Ambusher",
        "Casting Qasali Ambusher as described in its second ability still uses the stack. The spell can be countered."
    );
    let (mut t, q) = setup(Some(true));
    let method = alternative_methods(&mut t, q)
        .into_iter()
        .next()
        .expect("the alternative cost is offered");
    let spell = t.cast(P1, q).method(method).go();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P1), 0, "paid nothing");
    cast_card(&mut t, P0, "Cancel", &[obj(spell)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Qasali Ambusher"), "countered");
}
