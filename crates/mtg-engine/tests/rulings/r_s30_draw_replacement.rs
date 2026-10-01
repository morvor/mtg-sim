//! Rulings batch S30 — replacement effects on drawing a card (CR 614.11, 616.1): the Words
//! cycle ("{1}: The next time you would draw a card this turn, [effect] instead.") and
//! the order in which several draw replacements apply.

use crate::r_s01_common::supported;
use crate::r_s03_common::respond;
use crate::r_s25_common::{cast_new, creature_tokens};
use crate::r_s29_common::replacement_choosers;
use crate::r_s30_common::{pick_replacement, set_life};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p` draws a card (outside of any spell or ability); triggers are put on the stack.
fn draw(t: &mut TestGame, p: PlayerId) {
    t.g.draw_cards(p, 1);
    t.g.flush_events();
    t.settle();
}

/// Activates the Words enchantment `words` ({1}).
fn use_words(t: &mut TestGame, words: ObjectId) {
    t.lands(P0, "Wastes", 1);
    t.activate(P0, words, 0, &[]).unwrap();
    t.resolve();
}

fn wilding_first(g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    pick_replacement(g, d, "Words of Wilding")
}

#[test]
fn the_words_that_compile() {
    cr!("614.11");
    for name in ["Words of Worship", "Words of Wilding", "Words of Waste"] {
        supported(name);
    }
}

#[test]
fn a_words_replaces_only_the_next_draw() {
    cr!("614.11", "614.1a");
    let mut t = TestGame::new(2);
    // "{1}: The next time you would draw a card this turn, you gain 5 life instead."
    let worship = t.battlefield(P0, "Words of Worship");
    use_words(&mut t, worship);
    let hand = t.hand_size(P0);
    draw(&mut t, P0);
    assert_eq!(t.life(P0), 25);
    assert_eq!(t.hand_size(P0), hand);
    draw(&mut t, P0);
    assert_eq!(t.life(P0), 25);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn with_several_words_used_the_player_chooses_which_to_apply_each_draw() {
    cr!("616.1", "614.11");
    ruling!(
        "Words of Wilding",
        "If multiple Words have been used prior to drawing a card, then you can choose which one to apply (and use up) each time you draw a card."
    );
    supported("Words of Wilding");
    supported("Words of Worship");
    let mut t = TestGame::new(2);
    let worship = t.battlefield(P0, "Words of Worship");
    // "{1}: The next time you would draw a card this turn, create a 2/2 green Bear
    // creature token instead."
    let wilding = t.battlefield(P0, "Words of Wilding");
    use_words(&mut t, worship);
    use_words(&mut t, wilding);
    let hand = t.hand_size(P0);
    respond(&mut t, P0, wilding_first);
    // The first draw: P0 chose Words of Wilding, which was used up.
    let from = t.asked().len();
    draw(&mut t, P0);
    assert_eq!(replacement_choosers(&t, from), vec![P0]);
    assert_eq!(creature_tokens(&t, P0), 1);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.hand_size(P0), hand);
    // The second: Words of Worship.
    draw(&mut t, P0);
    assert_eq!(t.life(P0), 25);
    assert_eq!(creature_tokens(&t, P0), 1);
    assert_eq!(t.hand_size(P0), hand);
    // The third is drawn.
    draw(&mut t, P0);
    assert_eq!(t.hand_size(P0), hand + 1);
}

fn plagiarize_first(g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    pick_replacement(g, d, "Plagiarize")
}

fn phial_first(g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    pick_replacement(g, d, "Phial of Galadriel")
}

/// `caster` casts Plagiarize ("Until end of turn, if target player would draw a card,
/// instead that player skips that draw and you draw a card.") targeting `target`.
fn plagiarize(t: &mut TestGame, caster: PlayerId, target: PlayerId) {
    cast_new(t, caster, "Plagiarize", &[Entity::Player(target)]);
    t.resolve_all();
}

#[test]
fn the_player_drawing_orders_the_draw_replacements() {
    cr!("616.1", "614.11", "614.5");
    ruling!(
        "Phial of Galadriel",
        "If two or more replacement effects would apply to a card-drawing event, the player drawing the card chooses the order in which to apply them."
    );
    supported("Phial of Galadriel");
    supported("Plagiarize");
    // P0 (no cards in hand) controls Phial of Galadriel ("If you would draw a card while
    // you have no cards in hand, draw two cards instead."); P1 cast Plagiarize on P0.
    let run = |choice: fn(&mtg_engine::game::Game, &Decision) -> Option<Answer>| {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Phial of Galadriel");
        plagiarize(&mut t, P1, P0);
        assert_eq!(t.hand_size(P0), 0);
        let p1_hand = t.hand_size(P1);
        respond(&mut t, P0, choice);
        let from = t.asked().len();
        draw(&mut t, P0);
        assert_eq!(replacement_choosers(&t, from), vec![P0]);
        assert_eq!(t.hand_size(P0), 0);
        t.hand_size(P1) - p1_hand
    };
    // Plagiarize first: P0's draw is replaced by P1 drawing one card.
    assert_eq!(run(plagiarize_first), 1);
    // Phial first: "draw two cards", each of which Plagiarize replaces.
    assert_eq!(run(phial_first), 2);
}

#[test]
fn phial_replaces_only_the_first_of_several_draws() {
    cr!("121.2", "614.11");
    ruling!(
        "Phial of Galadriel",
        "Any time you are instructed to draw more than one card, you draw them one at a time."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Phial of Galadriel");
    // Divination ("Draw two cards.") is P0's only card in hand.
    cast_new(&mut t, P0, "Divination", &[]);
    assert_eq!(t.hand_size(P0), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn phial_doubles_life_gained_while_at_5_or_less_life() {
    cr!("614.1a");
    // "If you would gain life while you have 5 or less life, you gain twice that much life
    // instead."
    for (life, after) in [(5, 11), (6, 9)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Phial of Galadriel");
        set_life(&mut t, P0, life);
        t.g.recompute();
        t.g.gain_life(P0, 3);
        t.settle();
        assert_eq!(t.life(P0), after);
    }
}

#[test]
fn plagiarize_on_yourself_does_nothing_useful() {
    cr!("614.5", "614.11");
    ruling!(
        "Plagiarize",
        "If you target yourself, this spell has no useful effect. It will not cause an infinite loop since a replacement effect can’t modify the same event more than once."
    );
    let mut t = TestGame::new(2);
    plagiarize(&mut t, P0, P0);
    let hand = t.hand_size(P0);
    let lib = t.library_size(P0);
    draw(&mut t, P0);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.library_size(P0), lib - 1);
}

#[test]
fn plagiarize_draws_from_the_casters_library() {
    cr!("614.11", "121.1");
    ruling!(
        "Plagiarize",
        "You draw the card from your library as normal, not from your opponent’s library."
    );
    let mut t = TestGame::new(2);
    plagiarize(&mut t, P1, P0);
    let (h0, l0) = (t.hand_size(P0), t.library_size(P0));
    let (h1, l1) = (t.hand_size(P1), t.library_size(P1));
    draw(&mut t, P0);
    assert_eq!((t.hand_size(P0), t.library_size(P0)), (h0, l0));
    assert_eq!((t.hand_size(P1), t.library_size(P1)), (h1 + 1, l1 - 1));
}

#[test]
fn two_plagiarizes_cancel_each_other_out() {
    cr!("614.5", "616.1");
    ruling!(
        "Plagiarize",
        "If you and your opponent each cast Plagiarize on each other during the same turn, the two spells effectively cancel each other out."
    );
    let mut t = TestGame::new(2);
    plagiarize(&mut t, P0, P1);
    plagiarize(&mut t, P1, P0);
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    draw(&mut t, P0);
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (h0 + 1, h1));
    draw(&mut t, P1);
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (h0 + 1, h1 + 1));
}

#[test]
fn plagiarize_works_on_a_player_with_an_empty_library() {
    cr!("614.11", "704.5b");
    ruling!(
        "Plagiarize",
        "If you target a player whose library is empty, any effect or turn-based action that would cause that player to draw a card will cause you to draw a card instead."
    );
    let mut t = TestGame::new(2);
    plagiarize(&mut t, P1, P0);
    t.g.players[P0.idx()].library.clear();
    let h1 = t.hand_size(P1);
    draw(&mut t, P0);
    assert_eq!(t.hand_size(P1), h1 + 1);
    // P0 didn't draw from an empty library.
    assert!(!t.has_lost(P0));
}
