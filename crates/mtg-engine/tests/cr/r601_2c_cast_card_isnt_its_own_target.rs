//! CR 601.2a, 601.2c, 115.1: a card being cast is moved to the stack before its targets are
//! chosen, so a spell cast from a graveyard can't target the card itself there. A card whose
//! only legal target would be itself can't be cast (it isn't offered among the legal
//! actions).

use mtg_engine::decision::Action;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const FLASHBACK: CastMethod = CastMethod::Keyword(KeywordKind::Flashback);

fn offered(t: &mut TestGame, p: PlayerId, card: ObjectId, method: CastMethod) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Cast { card: c, method: m } if *c == card && *m == method))
}

#[test]
fn mystic_retrieval_cant_target_itself_in_the_graveyard() {
    cr!("601.2a", "601.2c", "115.1");
    ruling!("Mystic Retrieval", "can't target itself");
    // "Return target instant or sorcery card from your graveyard to your hand. Flashback
    // {2}{R}."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 3);
    let mr = t.graveyard(P0, "Mystic Retrieval");
    assert!(!offered(&mut t, P0, mr, FLASHBACK));
    assert!(t.cast(P0, mr).method(FLASHBACK).try_go().is_err());
    // With another sorcery card there, it is.
    t.graveyard(P0, "Divination");
    assert!(offered(&mut t, P0, mr, FLASHBACK));
    t.cast(P0, mr).method(FLASHBACK).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Divination"));
}

#[test]
fn other_cards_cast_from_a_graveyard_cant_target_themselves() {
    cr!("601.2a", "601.2c", "115.1");
    // Recoup: "Target sorcery card in your graveyard gains flashback until end of turn."
    // Sorceress's Schemes: "Return target instant or sorcery card from your graveyard or
    // exiled card with flashback you own to your hand."
    for (name, lands) in [("Recoup", 4), ("Sorceress's Schemes", 5)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        t.lands(P0, "Mountain", lands);
        let card = t.graveyard(P0, name);
        assert!(!offered(&mut t, P0, card, FLASHBACK), "{name}");
        t.graveyard(P0, "Divination");
        assert!(offered(&mut t, P0, card, FLASHBACK), "{name}");
    }
}

#[test]
fn a_card_cast_from_the_hand_may_target_another_copy_in_the_graveyard() {
    cr!("115.1", "601.2c");
    // Only the card being cast is left out: Mystic Retrieval cast from the hand may
    // target another Mystic Retrieval in the graveyard.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 4);
    let other = t.graveyard(P0, "Mystic Retrieval");
    let mr = t.hand(P0, "Mystic Retrieval");
    assert!(offered(&mut t, P0, mr, CastMethod::Normal));
    t.cast(P0, mr).target(other).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Mystic Retrieval"));
}
