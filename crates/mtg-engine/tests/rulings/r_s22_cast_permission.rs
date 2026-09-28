//! Rulings batch S22 — "Choose target artifact card in your graveyard. You may cast that
//! card this turn." (Emry, Lurker of the Loch; Silas Renn, Seeker Adept): a permission to
//! cast the card later in the turn, paying its costs (additional costs too, or an
//! alternative cost instead of its mana cost, CR 118.8, 118.9), following the timing
//! rules. It never lets a land card be played (CR 305.9).

use crate::r_s01_common::*;
use crate::r_s04_common::add_mana;
use crate::r_s08_common::legal_cast_methods;
use crate::r_s22_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Emry's ability ("{T}: Choose target artifact card in your graveyard. You may cast that
/// card this turn.") targets `card` in P0's graveyard and resolves.
fn emry_chooses(t: &mut TestGame, card: ObjectId) {
    let emry = t.battlefield(P0, "Emry, Lurker of the Loch");
    t.activate(P0, emry, 0, &[Entity::Object(card)])
        .expect("activate Emry");
    t.resolve_all();
}

#[test]
fn a_card_chosen_by_emry_is_cast_paying_its_costs_including_additional_costs() {
    cr!("118.8", "118.8a", "601.2b", "601.2f", "601.2h", "305.9");
    ruling!(
        "Emry, Lurker of the Loch",
        "You'll still pay all costs for a spell cast this way, including additional costs. You may also pay alternative costs if any are available."
    );
    supported("Emry, Lurker of the Loch");
    // Skyclave Relic {3}, kicker {3}: "When this artifact enters, if it was kicked,
    // create two tapped tokens that are copies of this artifact."
    let mut t = TestGame::new(2);
    let relic = t.graveyard(P0, "Skyclave Relic");
    // Before Emry's ability, it can't be cast from the graveyard.
    add_mana(&mut t, P0, ManaType::C, 6);
    assert!(legal_cast_methods(&mut t, P0, relic).is_empty());
    emry_chooses(&mut t, relic);
    assert!(!legal_cast_methods(&mut t, P0, relic).is_empty());
    // Its mana cost and its kicker cost are paid.
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, relic).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Skyclave Relic").len(), 3);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0, "six mana was spent");

    // Without mana for its cost, it can't be cast.
    let mut t = TestGame::new(2);
    let relic = t.graveyard(P0, "Skyclave Relic");
    emry_chooses(&mut t, relic);
    assert!(legal_cast_methods(&mut t, P0, relic).is_empty());

    // An alternative cost may be paid: Mistvein Borderpost ("You may pay {1} and return a
    // basic land you control to its owner's hand rather than pay this spell's mana cost.")
    let mut t = TestGame::new(2);
    let post = t.graveyard(P0, "Mistvein Borderpost");
    let island = t.lands(P0, "Island", 1)[0];
    emry_chooses(&mut t, post);
    let alt = legal_cast_methods(&mut t, P0, post)
        .into_iter()
        .find(|m| matches!(m, CastMethod::Alternative(_)))
        .expect("the alternative cost is available");
    t.cast(P0, post).method(alt).go();
    assert!(!t.on_battlefield(island));
    assert!(t.in_hand(P0, "Island"));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Mistvein Borderpost").len(), 1);
}

#[test]
fn a_card_chosen_by_emry_follows_the_timing_rules() {
    cr!("307.1", "601.3");
    ruling!(
        "Emry, Lurker of the Loch",
        "You'll still pay all costs for a spell cast this way"
    );
    supported("Emry, Lurker of the Loch");
    let mut t = TestGame::new(2);
    let relic = t.graveyard(P0, "Skyclave Relic");
    emry_chooses(&mut t, relic);
    add_mana(&mut t, P0, ManaType::C, 3);
    // Not while a spell is on the stack: an artifact is cast only at sorcery speed.
    let bolt = t.hand(P0, "Lightning Bolt");
    add_mana(&mut t, P0, ManaType::R, 1);
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    assert!(legal_cast_methods(&mut t, P0, relic).is_empty());
    t.resolve_all();
    assert!(!legal_cast_methods(&mut t, P0, relic).is_empty());
    // The permission lasts only this turn.
    let mut t = TestGame::new(2);
    let relic = t.graveyard(P0, "Skyclave Relic");
    emry_chooses(&mut t, relic);
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    add_mana(&mut t, P0, ManaType::C, 3);
    assert!(legal_cast_methods(&mut t, P0, relic).is_empty());
}

#[test]
fn silas_renn_lets_a_card_be_cast_but_not_a_land_played() {
    cr!("305.9", "305.1", "601.2");
    ruling!(
        "Silas Renn, Seeker Adept",
        "An effect that instructs you to \"cast\" a card doesn't allow you to play lands."
    );
    supported("Silas Renn, Seeker Adept");
    // "Whenever Silas Renn deals combat damage to a player, choose target artifact card in
    // your graveyard. You may cast that card this turn." Seat of the Synod is an artifact
    // land.
    let mut t = TestGame::new(2);
    let silas = t.battlefield(P0, "Silas Renn, Seeker Adept");
    let seat = t.graveyard(P0, "Seat of the Synod");
    t.answer_targets(P0, &[Entity::Object(seat)]);
    attack_p1_unblocked(&mut t, silas);
    t.advance_to(P0, Step::PostcombatMain);
    assert!(t.g.player(P0).lands_played_this_turn == 0);
    assert!(t.play_land(P0, seat).is_err(), "the land can't be played");
    assert_eq!(t.zone(seat), mtg_engine::object::Zone::Graveyard(P0));
    // An artifact card that isn't a land can be cast.
    let mut t = TestGame::new(2);
    let silas = t.battlefield(P0, "Silas Renn, Seeker Adept");
    let thopter = t.graveyard(P0, "Ornithopter");
    t.answer_targets(P0, &[Entity::Object(thopter)]);
    attack_p1_unblocked(&mut t, silas);
    t.advance_to(P0, Step::PostcombatMain);
    t.cast(P0, thopter).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 1);
}
