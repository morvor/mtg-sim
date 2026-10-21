//! Soulfire Eruption (hand-written, `src/cards/soulfire_eruption.rs`).

use mtg_engine::decision::Action;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn the_exiled_cards_may_be_played_until_the_end_of_the_next_turn_and_illegal_targets_are_skipped() {
    cr!("608.2b", "608.2f");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let first = t.library_top(P0, "Mountain");
    let second = t.library_top(P0, "Hill Giant");
    t.lands(P0, "Mountain", 9);
    let eruption = t.hand(P0, "Soulfire Eruption");
    t.cast(P0, eruption)
        .targets(&[Entity::Player(P1), Entity::Object(bears)])
        .go();
    // The creature leaves before it resolves: only P1 is a legal target, so only one card
    // is exiled.
    t.g.destroy(bears, None);
    t.resolve();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.zone(second), mtg_engine::object::Zone::Exile);
    assert_eq!(t.zone(first), mtg_engine::object::Zone::Library(P0));
    // The exiled card may be played until the end of P0's next turn.
    let giant = t.g.current(second);
    t.lands(P0, "Mountain", 4);
    let castable = |t: &mut TestGame| {
        t.g.turn.priority = Some(P0);
        t.g.legal_actions(P0).iter().any(
            |a| matches!(a, Action::Cast { card, method: CastMethod::Normal } if *card == giant),
        )
    };
    assert!(castable(&mut t));
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(castable(&mut t));
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(!castable(&mut t));
}

/// Targets affecting different players are taken in APNAP order (CR 608.2f): the active
/// player's own creature before the opponent, whatever order they were chosen in.
#[test]
fn targets_of_different_players_are_taken_in_apnap_order() {
    cr!("608.2f", "101.4");
    let mut t = TestGame::new(2);
    // Craw Wurm (6/4) is P0's own; P0 is the active player.
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Hill Giant");
    t.lands(P0, "Mountain", 9);
    let eruption = t.hand(P0, "Soulfire Eruption");
    t.cast(P0, eruption)
        .targets(&[Entity::Player(P1), Entity::Object(wurm)])
        .go();
    t.resolve();
    // The Wurm first: Hill Giant (4) destroys it; then Grizzly Bears (2) for P1.
    assert!(!t.on_battlefield(wurm));
    assert_eq!(t.life(P1), 18);
}
