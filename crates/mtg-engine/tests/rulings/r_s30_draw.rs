//! Rulings batch S30 — drawing cards (CR 121): what is and isn't a draw, "whenever you
//! draw your second card each turn" triggers, draw-step triggers, cycling triggers,
//! revealing each card drawn from a library played with its top card revealed, and an
//! additional beginning phase.

use crate::r_s01_common::{custom_card, supported};
use crate::r_s04_common::{can_cycle, cycle};
use crate::r_s25_common::{cast_new, creature_tokens};
use crate::r_s28_common::cast_card;
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p` draws `n` cards (outside of any spell or ability), and triggers are put on the
/// stack.
fn draw(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.draw_cards(p, n);
    t.g.flush_events();
    t.settle();
}

/// P0 puts a card into their hand without drawing it: Impulse ("Look at the top four
/// cards of your library. Put one of them into your hand and the rest on the bottom of
/// your library in any order.").
fn impulse(t: &mut TestGame, p: PlayerId) {
    let before = t.hand_size(p);
    cast_card(t, p, "Impulse");
    t.resolve_all();
    assert_eq!(t.hand_size(p), before + 1);
}

/// P0 returns Grizzly Bears from their graveyard to their hand with Raise Dead.
fn raise_dead(t: &mut TestGame) {
    let bears = t.graveyard(P0, "Grizzly Bears");
    cast_new(t, P0, "Raise Dead", &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn cycling_triggers_resolve_before_the_cycling_draw() {
    cr!("702.29a", "702.29d", "603.3");
    ruling!(
        "Archfiend of Ifnir",
        "Some cards have an ability that triggers whenever you cycle any card. These triggered abilities resolve before you draw from the cycling ability."
    );
    supported("Archfiend of Ifnir");
    let mut t = TestGame::new(2);
    // "Whenever you cycle or discard another card, put a -1/-1 counter on each creature
    // your opponents control."
    t.battlefield(P0, "Archfiend of Ifnir");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Ominous Seas: "Cycling {2}".
    let seas = t.hand(P0, "Ominous Seas");
    t.lands(P0, "Wastes", 2);
    let hand = t.hand_size(P0);
    cycle(&mut t, P0, seas, 0).unwrap();
    t.settle();
    // The cycling ability is on the stack with the trigger above it.
    assert_eq!(t.stack_len(), 2);
    assert_eq!(t.hand_size(P0), hand - 1);
    t.resolve();
    // The trigger resolved first: the counter is there, and the card isn't drawn yet.
    assert_eq!(t.counters(bears, "-1/-1"), 1);
    assert_eq!(t.hand_size(P0), hand - 1);
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn howling_mine_triggers_after_the_draw_for_the_turn() {
    cr!("504.1", "504.2");
    ruling!(
        "Howling Mine",
        "The triggered ability is put onto the stack after you have already drawn your card for the turn."
    );
    supported("Howling Mine");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Howling Mine");
    t.advance_to(P1, Step::Upkeep);
    let hand = t.hand_size(P1);
    t.advance_to(P1, Step::Draw);
    // When P1 first gets priority in the draw step, the turn's draw has happened and the
    // trigger is waiting on the stack.
    t.settle();
    assert_eq!(t.hand_size(P1), hand + 1);
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(t.hand_size(P1), hand + 2);
}

#[test]
fn faerie_vandal_triggers_only_on_the_second_card_drawn_each_turn() {
    cr!("603.2", "121.2");
    ruling!(
        "Faerie Vandal",
        "The triggered ability can trigger only once each turn. It doesn't matter whether the permanent with that ability was on the battlefield when the first card was drawn."
    );
    supported("Faerie Vandal");
    // The first card is drawn before Faerie Vandal is on the battlefield; it triggers on
    // the second, but not on the third or fourth.
    let mut t = TestGame::new(2);
    draw(&mut t, P0, 1);
    let vandal = t.battlefield(P0, "Faerie Vandal");
    draw(&mut t, P0, 1);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.counters(vandal, "+1/+1"), 1);
    draw(&mut t, P0, 1);
    draw(&mut t, P0, 1);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.counters(vandal, "+1/+1"), 1);
    // Not on the battlefield when the second card is drawn: it can't trigger that turn.
    let mut t = TestGame::new(2);
    draw(&mut t, P0, 2);
    let vandal = t.battlefield(P0, "Faerie Vandal");
    draw(&mut t, P0, 1);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.counters(vandal, "+1/+1"), 0);
}

#[test]
fn a_second_card_trigger_goes_on_the_stack_after_the_whole_draw() {
    cr!("603.3", "121.2");
    ruling!(
        "Knowledge Seeker",
        "If an effect instructs you to draw multiple cards, the ability triggers after you draw whichever is the second one for the turn."
    );
    supported("Knowledge Seeker");
    let mut t = TestGame::new(2);
    // "Whenever you draw your second card each turn, put a +1/+1 counter on this
    // creature."
    let seeker = t.battlefield(P0, "Knowledge Seeker");
    let hand = t.hand_size(P0);
    // Concentrate: "Draw three cards."
    cast_card(&mut t, P0, "Concentrate");
    t.resolve();
    // Concentrate finished resolving (all three cards drawn) before the trigger was put
    // on the stack; it triggered once.
    assert_eq!(t.hand_size(P0), hand + 3);
    assert!(t.in_graveyard(P0, "Concentrate"));
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.counters(seeker, "+1/+1"), 0);
    t.resolve_all();
    assert_eq!(t.counters(seeker, "+1/+1"), 1);
}

#[test]
fn otter_penguin_doesnt_count_a_card_put_into_hand_without_drawing() {
    cr!("121.1");
    ruling!(
        "Otter-Penguin",
        "If a card or ability instructs you to put cards into your hand without specifically using the word \"draw,\" it's not a card drawn."
    );
    supported("Otter-Penguin");
    let mut t = TestGame::new(2);
    // "Whenever you draw your second card each turn, this creature gets +1/+2 until end
    // of turn and can't be blocked this turn."
    let otter = t.battlefield(P0, "Otter-Penguin");
    draw(&mut t, P0, 1);
    impulse(&mut t, P0);
    assert_eq!(t.pt(otter), (2, 1));
    // The next card actually drawn is the second.
    draw(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(t.pt(otter), (3, 3));
}

#[test]
fn lorescale_coatl_doesnt_trigger_on_a_card_returned_to_hand() {
    cr!("121.1");
    ruling!(
        "Lorescale Coatl",
        "If a spell or ability causes you to put a card into your hand without specifically using the word \"draw,\" it's not a card drawn."
    );
    supported("Lorescale Coatl");
    let mut t = TestGame::new(2);
    // "Whenever you draw a card, put a +1/+1 counter on this creature."
    let coatl = t.battlefield(P0, "Lorescale Coatl");
    raise_dead(&mut t);
    assert_eq!(t.counters(coatl, "+1/+1"), 0);
    draw(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(t.counters(coatl, "+1/+1"), 1);
}

#[test]
fn bloodhaze_wolverine_doesnt_count_cards_put_into_hand() {
    cr!("121.1");
    ruling!(
        "Bloodhaze Wolverine",
        "If a spell or ability causes you to put cards into your hand without specifically using the word \"draw,\" it's not a card drawn."
    );
    supported("Bloodhaze Wolverine");
    let mut t = TestGame::new(2);
    // "Whenever you draw your second card each turn, this creature gets +1/+1 and gains
    // first strike until end of turn."
    let wolverine = t.battlefield(P0, "Bloodhaze Wolverine");
    draw(&mut t, P0, 1);
    // Unsummon returns P0's own Grizzly Bears to their hand.
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Unsummon", &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(t.pt(wolverine), (2, 1));
    draw(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(t.pt(wolverine), (3, 2));
}

#[test]
fn underworld_dreams_doesnt_trigger_when_a_card_is_put_into_hand() {
    cr!("121.1");
    ruling!(
        "Underworld Dreams",
        "If a spell or ability causes you to put cards into your hand without specifically using the word “draw,” it’s not a card drawn."
    );
    supported("Underworld Dreams");
    let mut t = TestGame::new(2);
    // "Whenever an opponent draws a card, this enchantment deals 1 damage to that
    // player."
    t.battlefield(P1, "Underworld Dreams");
    impulse(&mut t, P0);
    assert_eq!(t.life(P0), 20);
    draw(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
}

#[test]
fn mad_ratter_doesnt_count_a_card_put_into_hand_by_raven_familiar() {
    cr!("121.1");
    ruling!(
        "Mad Ratter",
        "If a spell or ability causes you to put cards into your hand without specifically using the word “draw,” it's not a card drawn."
    );
    supported("Mad Ratter");
    supported("Raven Familiar");
    let mut t = TestGame::new(2);
    // "Whenever you draw your second card each turn, create two 1/1 black Rat creature
    // tokens."
    t.battlefield(P0, "Mad Ratter");
    draw(&mut t, P0, 1);
    // Raven Familiar: "When this creature enters, look at the top three cards of your
    // library. Put one of them into your hand and the rest on the bottom of your library
    // in any order."
    let hand = t.hand_size(P0);
    t.enter(P0, "Raven Familiar");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(creature_tokens(&t, P0), 0);
    draw(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(creature_tokens(&t, P0), 2);
}

#[test]
fn impulse_is_not_a_draw() {
    cr!("121.1");
    ruling!("Impulse", "This is not a draw.");
    supported("Impulse");
    supported("Ominous Seas");
    let mut t = TestGame::new(2);
    // Ominous Seas: "Whenever you draw a card, put a foreshadow counter on this
    // enchantment."
    let seas = t.battlefield(P0, "Ominous Seas");
    impulse(&mut t, P0);
    assert_eq!(t.counters(seas, "foreshadow"), 0);
    draw(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(t.counters(seas, "foreshadow"), 1);
}

#[test]
fn each_card_drawn_is_revealed_before_it_is_drawn() {
    cr!("401.5", "121.2");
    ruling!(
        "Field of Dreams",
        "When playing with the top card of your library revealed, if an effect tells you to draw several cards, reveal each one before you draw it."
    );
    supported("Field of Dreams");
    let mut t = TestGame::new(2);
    // "Players play with the top card of their libraries revealed."
    t.battlefield(P0, "Field of Dreams");
    t.settle();
    let revealed = |t: &TestGame| {
        t.g.turn_events
            .iter()
            .chain(t.g.events.iter())
            .filter_map(|e| match e {
                Event::Custom {
                    name, player, obj, ..
                } if name == mtg_engine::zones::TOP_REVEALED && *player == Some(P1) => *obj,
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let before = revealed(&t).len();
    let lib: Vec<ObjectId> = t.g.players[P1.idx()].library.clone();
    let n = lib.len();
    // P1 draws three cards: the second and third were each revealed (as the new top card)
    // before being drawn, and then the new top card.
    draw(&mut t, P1, 3);
    let now = revealed(&t);
    assert_eq!(now.len() - before, 3);
    let names: Vec<ObjectId> = now[before..]
        .iter()
        .map(|o| t.g.current(*o))
        .collect::<Vec<_>>();
    // They were the cards that were second, third, and fourth from the top.
    let expect: Vec<ObjectId> = [n - 2, n - 3, n - 4]
        .iter()
        .map(|i| t.g.current(lib[*i]))
        .collect();
    assert_eq!(names, expect);
}

#[test]
fn an_additional_beginning_phase_untaps_has_upkeep_triggers_and_a_draw() {
    cr!("500.8", "502.3", "503.1a", "504.1");
    ruling!(
        "Sphinx of the Second Sun",
        "The additional beginning phase will be a lot like your normal beginning phase."
    );
    supported("Sphinx of the Second Sun");
    supported("Phyrexian Arena");
    let mut t = TestGame::new(2);
    // P0's second turn (P0 skipped the draw step of their first turn).
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::BeginningOfCombat);
    // "At the beginning of each of your postcombat main phases, there is an additional
    // beginning phase after this phase."
    t.battlefield(P0, "Sphinx of the Second Sun");
    // "At the beginning of your upkeep, you draw a card and lose 1 life."
    t.battlefield(P0, "Phyrexian Arena");
    let land = t.lands(P0, "Island", 1)[0];
    t.g.tap(land);
    t.advance_to(P0, Step::PostcombatMain);
    t.settle();
    t.resolve_all();
    let turn = t.g.turn.number;
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    // Still the same turn: the untap step untapped the land, and the upkeep trigger is on
    // the stack.
    assert_eq!(t.g.turn.number, turn);
    assert!(!t.obj_now(land).tapped);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.hand_size(P0), hand + 1);
    // And P0 draws a card in the draw step.
    t.advance_to(P0, Step::Draw);
    assert_eq!(t.g.turn.number, turn);
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn cycling_4_means_pay_4_and_discard_this_card_to_draw_a_card() {
    cr!("702.29a");
    ruling!("Gunk Slug", "Cycling {4} means");
    // A Gunk token card ("a colorless Gunk sorcery named Gunk with cycling {4}"; Gunk Slug
    // itself isn't supported), built as a custom card.
    let gunk = custom_card("Gunk", "Sorcery — Gunk", "", None, "Cycling {4}");
    let mut t = TestGame::new(2);
    let card = t.custom(P0, gunk, Zone::Hand(P0));
    // {4} is the whole mana cost: three lands aren't enough.
    let mut lands = t.lands(P0, "Wastes", 3);
    assert!(!can_cycle(&mut t, P0, card));
    lands.extend(t.lands(P0, "Wastes", 1));
    assert!(can_cycle(&mut t, P0, card));
    let hand = t.hand_size(P0);
    cycle(&mut t, P0, card, 0).unwrap();
    // Discarding the card is part of the cost; the card is drawn as the ability resolves.
    assert!(lands.iter().all(|l| t.obj_now(*l).tapped));
    assert!(t.in_graveyard(P0, "Gunk"));
    assert_eq!(t.hand_size(P0), hand - 1);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}
