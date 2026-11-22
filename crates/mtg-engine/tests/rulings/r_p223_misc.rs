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

#[test]
fn jund_gives_devour_to_creature_spells_that_are_black_red_or_green() {
    cr!("702.82a", "105.2");
    ruling!("Jund", "Casting a multicolor spell will cause the first ability to trigger as long as that spell is black, red, or green. It can be other colors as well.");
    supported("Jund");
    // "Whenever a player casts a creature spell that's black, red, or green, it gains
    // devour 5."
    let mut t = crate::r_s19_common::planechase_game(2);
    crate::r_s19_common::start_planar_deck(&mut t, P0, &["Jund"]);
    // A green-white creature spell (Watchwolf) triggers it, and devours for five
    // counters per creature.
    let food = t.battlefield(P0, "Llanowar Elves");
    let wolf = in_hand_with_mana(&mut t, P0, "Watchwolf");
    t.answer_choose(P0, &[Entity::Object(food)]);
    t.cast(P0, wolf).go();
    t.resolve_all();
    assert!(!t.on_battlefield(food));
    let wolf = t.named_on_battlefield("Watchwolf")[0];
    assert_eq!(
        t.counters(wolf, mtg_engine::types::counters::PLUS1),
        5,
        "devour 5 from Jund"
    );
    // A blue creature spell doesn't: nothing is devoured.
    let food = t.battlefield(P1, "Llanowar Elves");
    let drake = in_hand_with_mana(&mut t, P1, "Wind Drake");
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.answer_choose(P1, &[Entity::Object(food)]);
    t.cast(P1, drake).go();
    t.resolve_all();
    assert!(t.on_battlefield(food));
    let drake = t.named_on_battlefield("Wind Drake")[0];
    assert_eq!(t.counters(drake, mtg_engine::types::counters::PLUS1), 0);
}
