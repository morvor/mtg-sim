//! Trigger events qualified by the spell or ability that caused them: "Whenever a spell or
//! ability an opponent controls causes you to discard a card / cards" (Spiritual Focus,
//! Pure Intentions).

use crate::combat2_common::*;
use crate::td_next_turn_durations::mana;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn spiritual_focus_triggers_only_on_an_opponents_discard_effect() {
    cr!("701.9a", "603.2");
    supported("Spiritual Focus");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P0, "Spiritual Focus");
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Grizzly Bears");
    // An opponent's Mind Rot: each card discarded triggers it.
    let rot = t.hand(P1, "Mind Rot");
    mana(&mut t, P1);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.cast_with(P1, rot, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    // Your own discard effect doesn't.
    let p0_hand = t.hand_size(P0);
    let catalog = t.hand(P0, "Catalog");
    mana(&mut t, P0);
    t.cast_with(P0, catalog, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.hand_size(P0), p0_hand + 1);
}

#[test]
fn spiritual_focus_may_draw_is_optional() {
    cr!("603.2");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P0, "Spiritual Focus");
    t.hand(P0, "Grizzly Bears");
    let rot = t.hand(P1, "Raven's Crime");
    mana(&mut t, P1);
    t.answer_yes(P0, false);
    t.cast_with(P1, rot, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn pure_intentions_returns_cards_an_opponent_made_you_discard_this_turn() {
    cr!("603.7b", "603.2c");
    ruling!(
        "Pure Intentions",
        "only the controller of the spell or ability matters"
    );
    supported("Pure Intentions");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Llanowar Elves");
    let pi = t.hand(P0, "Pure Intentions");
    mana(&mut t, P0);
    t.cast_with(P0, pi, &[]).unwrap();
    t.resolve_all();
    let rot = t.hand(P1, "Mind Rot");
    mana(&mut t, P1);
    t.cast_with(P1, rot, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    // Both discarded cards are back in P0's hand (one trigger for the two).
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_hand(P0, "Llanowar Elves"));
    let _ = (a, b);
    // P0's own discard this turn isn't returned.
    let catalog = t.hand(P0, "Catalog");
    mana(&mut t, P0);
    t.cast_with(P0, catalog, &[]).unwrap();
    t.resolve_all();
    assert_eq!(
        t.g.players[0]
            .graveyard
            .iter()
            .filter(|c| {
                let n = t.g.obj(**c).chars.name.clone();
                n.as_str() == "Grizzly Bears" || n.as_str() == "Llanowar Elves"
            })
            .count(),
        1
    );
}
