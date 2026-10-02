//! Rulings batch P223 — shroud and shadow: Diplomatic Immunity (CR 702.18a) and Faceless
//! Devourer's loop (CR 104.4b).

use crate::r_s01_common::*;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s04_common::spell_targets;
use mtg_engine::game::GameResult;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn diplomatic_immunity_can_be_targeted_on_the_stack() {
    cr!("702.18a", "115.1");
    ruling!("Diplomatic Immunity", "Shroud does not apply until after it is on the battlefield, so it can be the target of a spell while it is on the stack.");
    supported("Diplomatic Immunity");
    // On the stack: Cancel can target it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let di = in_hand_with_mana(&mut t, P0, "Diplomatic Immunity");
    let spell = t.cast(P0, di).target(bears).go();
    let cancel = in_hand_with_mana(&mut t, P1, "Cancel");
    t.cast(P1, cancel).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Diplomatic Immunity"));
    // On the battlefield: it has shroud, and so does the enchanted creature.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let di = in_hand_with_mana(&mut t, P0, "Diplomatic Immunity");
    t.cast(P0, di).target(bears).go();
    t.resolve_all();
    let aura = t.named_on_battlefield("Diplomatic Immunity")[0];
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    assert!(!spell_targets(&mut t, P1, "Naturalize").contains(&Entity::Object(aura)));
    assert!(!spell_targets(&mut t, P1, "Lightning Bolt").contains(&Entity::Object(bears)));
}

#[test]
fn three_faceless_devourers_loop_forever_and_the_game_is_a_draw() {
    cr!("104.4b", "603.6a", "610.3");
    ruling!("Faceless Devourer", "Three Faceless Devourers entering the battlefield at separate times with no other creatures with shadow on the battlefield will result in an infinite loop. The game ends in a draw unless a player somehow interrupts the loop.");
    supported("Faceless Devourer");
    // "When this creature enters, exile another target creature with shadow. When this
    // creature leaves the battlefield, return the exiled card to the battlefield under
    // its owner's control."
    let mut t = TestGame::new(2);
    t.enter(P0, "Faceless Devourer");
    t.resolve_all();
    t.enter(P0, "Faceless Devourer");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Faceless Devourer").len(), 1);
    assert!(t.g.result.is_none());
    t.enter(P0, "Faceless Devourer");
    t.g.run_until(2_000, |g| g.result.is_some());
    assert_eq!(t.g.result, Some(GameResult::Draw));
}
