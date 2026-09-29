//! Rulings batch S26 — spells with "any number of target players" and a strive-like cost
//! per extra target: Call the Coppercoats ("Choose any number of target opponents. Create X
//! 1/1 white Human Soldier creature tokens, where X is the number of creatures those
//! opponents control.") and Officious Interrogation ("Choose any number of target players.
//! Investigate X times, where X is the total number of creatures those players control.").
//! A copy keeps the number of targets (CR 707.10c, 115.7); targets that became illegal
//! aren't counted (CR 608.2b); the extra cost doesn't change the mana value (CR 202.3).

use crate::r_s01_common::{supported, tokens};
use crate::r_s25_common::{change_copy_targets, targets_of};
use crate::r_s26_common::*;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

/// A three-player game where P1 controls two creatures and P2 three.
fn three_players() -> TestGame {
    let mut t = TestGame::new(3);
    for _ in 0..2 {
        t.battlefield(P1, "Grizzly Bears");
    }
    for _ in 0..3 {
        t.battlefield(P2, "Grizzly Bears");
    }
    t
}

/// The Clue tokens `p` controls.
fn clues(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.subtypes.iter().any(|s| s == "Clue"))
        .count()
}

#[test]
fn a_copy_keeps_the_number_of_targets() {
    cr!("707.10c", "115.7");
    ruling!(
        "Call the Coppercoats",
        "If this spell is copied and the effect that copies the spell allows a player to choose new targets for the copy, the number of targets can't be changed."
    );
    supported("Call the Coppercoats");
    supported("Twincast");
    let mut t = three_players();
    t.lands(P0, "Plains", 3);
    t.lands(P0, "Island", 2);
    // Call the Coppercoats targets only P1.
    let call = t.hand(P0, "Call the Coppercoats");
    let call = t.cast(P0, call).targets(&[Entity::Player(P1)]).go();
    assert_eq!(targets_of(&t, call), vec![Entity::Player(P1)]);
    // Twincast copies it: the copy's one target can become P2, but it can't gain one.
    let twincast = t.hand(P0, "Twincast");
    t.cast(P0, twincast).target(call).go();
    change_copy_targets(&mut t, P0, &[Some(Entity::Player(P2))]);
    t.resolve();
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(targets_on_stack(&t, copies[0]), vec![Entity::Player(P2)]);
    t.resolve();
    assert_eq!(tokens(&t, P0).len(), 3);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 5);
}

#[test]
fn opponents_that_are_no_longer_legal_targets_arent_counted() {
    cr!("608.2b", "115.2", "702.11c");
    ruling!(
        "Call the Coppercoats",
        "Any target opponents that are no longer legal targets by the time Call the Coppercoats resolves won't have their creatures counted"
    );
    ruling!(
        "Call the Coppercoats",
        "The mana value of a strive spell doesn't change no matter how many targets it has."
    );
    supported("Leyline of Sanctity");
    let mut t = three_players();
    t.lands(P0, "Plains", 5);
    let call = t.hand(P0, "Call the Coppercoats");
    let call = t
        .cast(P0, call)
        .targets(&[Entity::Player(P1), Entity::Player(P2)])
        .go();
    assert_eq!(targets_of(&t, call).len(), 2);
    // Two targets: {2}{W} plus {1}{W}, all five lands tapped; the mana value is still 3.
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 5);
    assert_eq!(mv(&mut t, call), 3);
    // P2 gains hexproof ("You have hexproof.") before it resolves.
    t.battlefield(P2, "Leyline of Sanctity");
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2);
}

#[test]
fn officious_interrogation_counts_its_legal_target_players() {
    cr!("608.2b", "701.16a", "202.3");
    ruling!(
        "Officious Interrogation",
        "Any target players that are no longer legal targets by the time Officious Interrogation resolves won't have their creatures counted"
    );
    ruling!(
        "Officious Interrogation",
        "Officious Interrogation's mana value doesn't change no matter how many targets it has."
    );
    supported("Officious Interrogation");
    let mut t = three_players();
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Tundra", 6);
    // Targets P0 itself, P1 and P2: {W}{U} plus {W}{U} twice.
    let spell = t.hand(P0, "Officious Interrogation");
    let spell = t
        .cast(P0, spell)
        .targets(&[Entity::Player(P0), Entity::Player(P1), Entity::Player(P2)])
        .go();
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 6);
    assert_eq!(mv(&mut t, spell), 2);
    t.battlefield(P1, "Leyline of Sanctity");
    t.resolve_all();
    // P0's one creature and P2's three.
    assert_eq!(clues(&t, P0), 4);
}

#[test]
fn officious_interrogation_can_have_no_targets() {
    cr!("601.2c", "701.16a");
    ruling!(
        "Officious Interrogation",
        "It's legal to cast Officious Interrogation with no targets"
    );
    let mut t = three_players();
    t.lands(P0, "Tundra", 2);
    let spell = t.hand(P0, "Officious Interrogation");
    let spell = t.cast(P0, spell).targets(&[]).go();
    assert!(targets_of(&t, spell).is_empty());
    t.resolve_all();
    assert_eq!(clues(&t, P0), 0);
    assert!(t.in_graveyard(P0, "Officious Interrogation"));
}

#[test]
fn casting_it_without_paying_its_mana_cost_still_costs_extra_targets() {
    cr!("118.9d", "601.2f");
    ruling!(
        "Call the Coppercoats",
        "If a spell or ability allows you to cast a strive spell without paying its mana cost, you must pay the additional cost for any targets beyond the first."
    );
    let mut t = three_players();
    t.lands(P0, "Plains", 2);
    let call = t.hand(P0, "Call the Coppercoats");
    mtg_engine::casting::grant_play_permission(
        &mut t.g,
        P0,
        vec![call],
        mtg_engine::ability::Duration::EndOfTurn,
        true,
        None,
    );
    t.cast(P0, call)
        .targets(&[Entity::Player(P1), Entity::Player(P2)])
        .method(CastMethod::Free)
        .go();
    // The {1}{W} for the second target was paid.
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 2);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 5);
}
