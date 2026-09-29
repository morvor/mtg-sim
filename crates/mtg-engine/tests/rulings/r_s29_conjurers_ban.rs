//! Rulings batch S29 — Conjurer's Ban: "Choose a card name. Until your next turn, spells
//! with the chosen name can't be cast and lands with the chosen name can't be played. Draw
//! a card." The name is chosen as it resolves (CR 607.2d); the restrictions last until its
//! controller's next turn (CR 611.2a) and don't counter anything (CR 101.2).

use crate::r_s01_common::supported;
use crate::r_s02_common::{can_cast, can_play_land};
use crate::r_s04_common::add_mana;
use crate::r_s25_common::cast_new;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 casts Conjurer's Ban, naming `name` as it resolves. Returns the spell (not yet
/// resolved).
fn ban(t: &mut TestGame, name: &str) -> ObjectId {
    supported("Conjurer's Ban");
    t.answer(P0, DecisionKind::Name, Answer::Text(name.into()));
    cast_new(t, P0, "Conjurer's Ban", &[])
}

#[test]
fn it_doesnt_affect_spells_cast_in_response() {
    cr!("101.2", "607.2d", "611.2a");
    ruling!(
        "Conjurer's Ban",
        "This can't be used as a counterspell. It will have no effect on spells which were on the stack when it was cast, nor on those cast in response to it."
    );
    ruling!(
        "Conjurer's Ban",
        "Players may cast spells while Conjurer's Ban is on the stack. Only after all players pass priority does Conjurer's Ban resolve, and that's when you name a card. Once you name a card, that card can't be played in response."
    );
    let mut t = TestGame::new(2);
    ban(&mut t, "Lightning Bolt");
    // P1 responds with a Lightning Bolt: it resolves (first) and deals its damage.
    cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    // Now P1 can't cast another one, until P0's next turn.
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    assert!(!can_cast(&mut t, P1, bolt, CastMethod::Normal));
    t.advance_to(P1, Step::Upkeep);
    assert!(!can_cast(&mut t, P1, bolt, CastMethod::Normal));
    t.advance_to(P0, Step::Upkeep);
    assert!(can_cast(&mut t, P1, bolt, CastMethod::Normal));
}

#[test]
fn naming_a_land_card_stops_it_from_being_played() {
    cr!("305.1", "611.2a");
    ruling!(
        "Conjurer's Ban",
        "You may name a land card to prevent it from being played."
    );
    let mut t = TestGame::new(2);
    ban(&mut t, "Forest");
    t.resolve_all();
    t.advance_to(P1, Step::PrecombatMain);
    let forest = t.hand(P1, "Forest");
    let island = t.hand(P1, "Island");
    assert!(!can_play_land(&mut t, P1, forest));
    assert!(can_play_land(&mut t, P1, island));
    // P0 can't play one either.
    t.advance_to(P0, Step::Upkeep);
    let mut t = TestGame::new(2);
    ban(&mut t, "Forest");
    t.resolve_all();
    let forest = t.hand(P0, "Forest");
    assert!(!can_play_land(&mut t, P0, forest));
}

#[test]
fn a_copy_of_a_card_isnt_affected() {
    cr!("707.12", "611.2a");
    ruling!(
        "Conjurer's Ban",
        "Conjurer's Ban only affects cards. A copy of a card (for example, one created by Eye of the Storm isn't a card, so it can be played."
    );
    supported("Isochron Scepter");
    // P1's Isochron Scepter has Lightning Bolt imprinted; P0 names Lightning Bolt.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[Entity::Object(bolt)]);
    let scepter = t.enter(P1, "Isochron Scepter");
    t.resolve_all();
    t.set_step(P0, Step::PrecombatMain);
    ban(&mut t, "Lightning Bolt");
    t.resolve_all();
    // A Lightning Bolt card can't be cast, but a copy of the imprinted one can.
    t.set_step(P1, Step::PrecombatMain);
    let other = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    assert!(!can_cast(&mut t, P1, other, CastMethod::Normal));
    add_mana(&mut t, P1, ManaType::C, 2);
    t.answer_yes(P1, true);
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[Entity::Player(P0)]);
    crate::r_s06_common::activate_containing(&mut t, P1, scepter, "copy").unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
}
