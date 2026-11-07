//! Firestorm Phoenix (hand-written, `src/cards/firestorm_phoenix.rs`): it returns to its
//! owner's hand instead of dying; until that player's next turn, that card is revealed and
//! can't be played.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn phoenix_in_hand(t: &TestGame, p: PlayerId) -> ObjectId {
    t.g.player(p)
        .hand
        .iter()
        .copied()
        .find(|o| t.obj(*o).chars.name == "Firestorm Phoenix")
        .expect("Firestorm Phoenix in hand")
}

fn kill_phoenix(t: &mut TestGame) -> ObjectId {
    let phoenix = t.battlefield(P0, "Firestorm Phoenix");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.g.turn.priority = Some(P1);
    t.cast(P1, bolt).target(phoenix).go();
    t.resolve();
    phoenix_in_hand(t, P0)
}

#[test]
fn it_returns_to_hand_revealed_and_cant_be_cast_until_its_owners_next_turn() {
    cr!("614.1a");
    ruling!("Firestorm Phoenix", "The “rebirth” is a replacement effect.");
    ruling!("Firestorm Phoenix", "It may not be cast again until your next turn");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let card = kill_phoenix(&mut t);
    assert!(mtg_engine::reveal::is_revealed(&t.g, card));
    t.lands(P0, "Mountain", 6);
    assert!(t.cast(P0, card).try_go().is_err());
    // Still during the opponent's turn; on its owner's next turn it's no longer tracked.
    t.advance_to(P1, Step::PrecombatMain);
    assert!(mtg_engine::reveal::is_revealed(&t.g, card));
    t.advance_to(P0, Step::PrecombatMain);
    let card = phoenix_in_hand(&t, P0);
    assert!(!mtg_engine::reveal::is_revealed(&t.g, card));
    for l in t.g.permanents_controlled_by(P0) {
        t.g.untap(l);
    }
    t.cast(P0, card).go();
}

#[test]
fn another_phoenix_in_hand_can_be_cast() {
    cr!("614.1a");
    ruling!("Firestorm Phoenix", "a different Firestorm Phoenix in your hand can");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let returned = kill_phoenix(&mut t);
    let other = t.hand(P0, "Firestorm Phoenix");
    assert!(!mtg_engine::reveal::is_revealed(&t.g, other));
    t.lands(P0, "Mountain", 6);
    assert!(t.cast(P0, returned).try_go().is_err());
    t.cast(P0, other).go();
}
