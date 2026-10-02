//! Rulings batch S33 — {X} in a library: a card's mana value in any zone but the stack
//! treats X as 0 (CR 202.3e), so a revealed or searched-for card with {X} in its mana
//! cost has the mana value of the rest of its cost.

use crate::r_s01_common::{stack_library, supported};
use crate::r_s03_common::choice_candidates;
use crate::r_s04_common::next_upkeep;
use crate::r_s05_common::enter;
use crate::r_s33_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn dark_confidant_reveals_a_card_with_x_as_mana_value_without_x() {
    cr!("202.3e", "202.3b");
    ruling!(
        "Dark Confidant",
        "If a card in a player's library has {X} in its mana cost, X is considered to be 0."
    );
    supported("Dark Confidant");
    // "At the beginning of your upkeep, reveal the top card of your library and put that
    // card into your hand. You lose life equal to its mana value."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dark Confidant");
    next_upkeep(&mut t, P0);
    // The top card when the trigger resolves: Fireball ({X}{R}) has mana value 1.
    let fireball = t.library_top(P0, "Fireball");
    let life = t.life(P0);
    t.resolve_all();
    assert_eq!(t.zone(fireball), mtg_engine::object::Zone::Hand(P0));
    assert_eq!(t.life(P0), life - 1);
}

#[test]
fn sifter_wurm_gains_life_equal_to_mana_value_with_x_as_0() {
    cr!("202.3e", "701.22a");
    ruling!(
        "Sifter Wurm",
        "For cards in your library with {X} in their mana costs, X is considered to be 0."
    );
    supported("Sifter Wurm");
    // "When this creature enters, scry 3, then reveal the top card of your library. You
    // gain life equal to that card's mana value." Its top card is Blaze ({X}{R}).
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Blaze", "Grizzly Bears", "Hill Giant"]);
    enter(&mut t, P0, "Sifter Wurm");
    let life = t.life(P0);
    t.resolve_all();
    assert_eq!(t.life(P0), life + 1);
}

#[test]
fn ranger_captain_of_eos_can_find_a_creature_with_x_in_its_cost() {
    cr!("202.3e", "701.23a");
    ruling!(
        "Ranger-Captain of Eos",
        "If a card in a player’s library has {X} in its mana cost, X is considered to be 0."
    );
    supported("Ranger-Captain of Eos");
    // "When this creature enters, you may search your library for a creature card with
    // mana value 1 or less, reveal it, put it into your hand, then shuffle."
    // Walking Ballista ({X}{X}) has mana value 0 in the library; Grizzly Bears (mana
    // value 2) doesn't qualify.
    let mut t = TestGame::new(2);
    let ballista = t.library_top(P0, "Walking Ballista");
    t.library_top(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(ballista)]);
    let from = t.asked().len();
    enter(&mut t, P0, "Ranger-Captain of Eos");
    t.resolve_all();
    let cands = choice_candidates(&t, from, "Search");
    assert_eq!(cands, vec![vec![Entity::Object(ballista)]]);
    assert!(t.in_hand(P0, "Walking Ballista"));
    assert!(library_top_first(&t, P0)
        .iter()
        .all(|c| t.g.obj(*c).chars.name != "Walking Ballista"));
}
