//! CR 701.20: reveal.

use crate::a701_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::reveal::is_revealed;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Casts Caustic Exhale ("As an additional cost to cast this spell, behold a Dragon or pay
/// {1}. Target creature gets -3/-3 until end of turn.") beholding `dragon` from hand.
fn exhale_beholding(t: &mut TestGame, dragon: ObjectId, target: ObjectId) -> ObjectId {
    t.lands(P0, "Swamp", 1);
    let exhale = t.hand(P0, "Caustic Exhale");
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer_choose(P0, &[Entity::Object(dragon)]);
    t.cast(P0, exhale).target(target).go()
}

#[test]
fn a_card_revealed_for_a_cost_stays_revealed_until_the_spell_leaves_the_stack() {
    cr!("701.20", "701.20a", "701.20b");
    ruling!(
        "Caustic Exhale",
        "that card remains revealed from the time the spell is announced until the time it leaves the stack"
    );
    supported("Caustic Exhale");
    let mut t = TestGame::new(2);
    let dragon = t.hand(P0, "Shivan Dragon");
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(!is_revealed(&t.g, dragon));
    let s = exhale_beholding(&mut t, dragon, bears);
    assert_eq!(t.zone(s), Zone::Stack);
    // Revealed, and still the same object in the same zone.
    assert!(is_revealed(&t.g, dragon));
    assert!(t.g.is_live(dragon));
    assert_eq!(t.obj(dragon).zone, Zone::Hand(P0));
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // The spell left the stack: the card isn't revealed any more.
    assert!(t.g.is_live(dragon));
    assert!(!is_revealed(&t.g, dragon));
}

#[test]
fn a_card_revealed_by_an_effect_is_revealed_only_while_the_effect_needs_it() {
    cr!("701.20a", "701.20b");
    supported("Duress");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bears = t.hand(P1, "Grizzly Bears");
    let log = spy(&mut t, P0, move |g, _p, d| match d {
        Decision::ChooseEntities { .. } => Some(format!(
            "{} {}",
            is_revealed(g, bolt),
            is_revealed(g, bears)
        )),
        _ => None,
    });
    let duress = t.hand(P0, "Duress");
    t.cast(P0, duress).target(P1).go();
    t.resolve();
    // While P0 chose a card, P1's hand was revealed.
    assert_eq!(probe_lines(&log), vec!["true true".to_string()]);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    // Afterwards it isn't; the Bears never left P1's hand.
    assert!(t.g.is_live(bears));
    assert!(!is_revealed(&t.g, bears));
}

#[test]
fn a_card_whose_reveal_triggers_an_ability_stays_revealed_until_that_ability_leaves() {
    cr!("701.20a");
    supported("Thunderous Wrath");
    let mut t = TestGame::new(2);
    // Miracle: "You may reveal this card from your hand as you draw it ... When you reveal
    // this card this way, you may cast it by paying [its miracle cost]."
    let wrath = t.library_top(P0, "Thunderous Wrath");
    t.answer_yes(P0, true); // reveal it
    t.g.draw_cards(P0, 1);
    let wrath = t.g.current(wrath);
    assert!(is_revealed(&t.g, wrath));
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert!(is_revealed(&t.g, wrath));
    t.answer_yes(P0, false); // don't cast it
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.obj(wrath).zone, Zone::Hand(P0));
    assert!(!is_revealed(&t.g, wrath));
}

#[test]
fn a_card_that_is_already_revealed_may_be_revealed_again() {
    cr!("701.20c");
    ruling!(
        "Caustic Exhale",
        "you may reveal it again to pay the cost of another spell or ability that requires you to reveal a card from your hand"
    );
    supported("Telepathy");
    supported("Caustic Exhale");
    let mut t = TestGame::new(2);
    // P1's Telepathy: "Your opponents play with their hands revealed."
    t.battlefield(P1, "Telepathy");
    let dragon = t.hand(P0, "Shivan Dragon");
    assert!(is_revealed(&t.g, dragon));
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    // The Dragon card is revealed to pay for one Exhale, and again (while that spell is
    // still on the stack) for another.
    exhale_beholding(&mut t, dragon, a);
    exhale_beholding(&mut t, dragon, b);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 2);
    // Neither alternative {1} was paid: only the two Swamps were used.
    assert!(t.g.is_live(dragon));
    // Without Telepathy, it's no longer revealed.
    assert!(is_revealed(&t.g, dragon));
    let telepathy = t.named_on_battlefield("Telepathy")[0];
    t.g.destroy(telepathy, None);
    t.g.recompute();
    assert!(!is_revealed(&t.g, dragon));
}

#[test]
fn reordering_a_library_ends_reveals_and_makes_new_objects() {
    cr!("701.20d");
    supported("Courser of Kruphix");
    let mut t = TestGame::new(2);
    clear_library(&mut t, P0);
    let card = t.library_top(P0, "Grizzly Bears");
    // "Play with the top card of your library revealed."
    t.battlefield(P0, "Courser of Kruphix");
    t.g.recompute();
    assert!(is_revealed(&t.g, card));
    t.g.shuffle_library(P0);
    t.g.recompute();
    // The card was reordered (even into the same place): it's a new object, revealed
    // anew as the top card.
    assert!(!t.g.is_live(card));
    let top = t.g.player(P0).library[t.g.player(P0).library.len() - 1];
    assert_ne!(top, card);
    assert!(is_revealed(&t.g, top));
}
