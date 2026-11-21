//! Rulings batch P203 — commander mechanics: choose a Background (CR 702.124k) on a card
//! that is itself a Background, and commander ninjutsu (CR 702.49d).

use crate::r_s01_common::*;
use crate::r_s03_common::to_blockers;
use crate::r_s12_common::attack_target;
use mtg_engine::card::card;
use mtg_engine::kw::partner::{commanders_problem, ineligible_commander};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn faceless_one_can_be_one_of_two_commanders_either_way_but_not_one_of_three() {
    cr!("702.124k", "702.124g", "903.3");
    ruling!(
        "Faceless One",
        "Faceless One both has choose a Background and is itself a Background. This means that you may have two commanders if one of them is Faceless One and the other is either a Legendary Creature with the choose a background ability OR is a legendary Background enchantment. It does not allow you to have three commanders."
    );
    for name in [
        "Faceless One",
        "Wilson, Refined Grizzly",
        "Raised by Giants",
        "Isamaru, Hound of Konda",
    ] {
        supported(name);
    }
    let faceless = card("Faceless One");
    let wilson = card("Wilson, Refined Grizzly"); // legendary creature, choose a Background
    let giants = card("Raised by Giants"); // legendary Background enchantment
    let isamaru = card("Isamaru, Hound of Konda"); // no partner ability
    let ok = |cs: &[&mtg_engine::card::CardDef]| {
        commanders_problem(cs).is_none() && ineligible_commander(cs, false).is_none()
    };
    // A legendary creature card, it can also be a commander on its own (CR 903.3).
    assert!(ok(&[&faceless]));
    assert!(ok(&[&faceless, &wilson]));
    assert!(ok(&[&wilson, &faceless]));
    assert!(ok(&[&faceless, &giants]));
    assert!(!ok(&[&faceless, &wilson, &giants]));
    assert!(!ok(&[&faceless, &isamaru]));
}

#[test]
fn commander_ninjutsu_from_the_command_zone_or_hand_attacks_what_the_returned_creature_did() {
    cr!("702.49d", "702.49c");
    ruling!(
        "Yuriko, the Tiger's Shadow",
        "Commander ninjutsu is a variant of ninjutsu that can be activated from the command zone as well as from your hand. Just as with regular ninjutsu, the Ninja enters attacking the player or planeswalker that the returned creature was attacking."
    );
    supported("Yuriko, the Tiger's Shadow");
    // Three players; the Bears attack P2's Jace Beleren, unblocked. Yuriko's first
    // activated ability functions from the hand, the second from the command zone.
    for from_command in [true, false] {
        let mut t = TestGame::new(3);
        let jace = t.battlefield(P2, "Jace Beleren");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.lands(P0, "Island", 1);
        t.lands(P0, "Swamp", 1);
        let yuriko = if from_command {
            t.command(P0, "Yuriko, the Tiger's Shadow")
        } else {
            t.hand(P0, "Yuriko, the Tiger's Shadow")
        };
        to_blockers(&mut t, &[(bears, Entity::Object(jace))], &[]);
        t.answer_choose(P0, &[Entity::Object(bears)]);
        t.activate(P0, yuriko, if from_command { 1 } else { 0 }, &[])
            .unwrap();
        t.resolve_all();
        let y = t.named_on_battlefield("Yuriko, the Tiger's Shadow");
        assert_eq!(y.len(), 1, "from command zone: {from_command}");
        assert!(t.g.is_attacking(y[0]));
        assert!(t.obj(y[0]).tapped);
        assert_eq!(attack_target(&t, y[0]), Some(Entity::Object(jace)));
        assert_eq!(t.zone(bears), Zone::Hand(P0));
    }
}
