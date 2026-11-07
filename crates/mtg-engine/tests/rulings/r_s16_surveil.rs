//! Rulings on surveil (CR 701.25): the choices, the order of a card's instructions, and
//! targeted spells that surveil.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s04_common::next_upkeep;
use crate::r_s07_common::resolved;
use crate::r_s15_common::library_top_n;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn is_surveil(d: &Decision) -> bool {
    matches!(d, Decision::Surveil { .. })
}

/// P0 casts Otherworldly Gaze ("Surveil 3.") with Swamp, Island, and Forest on top of the
/// library (top first), keeping the cards at indices `top` on top in that order and
/// putting those at `graveyard` into the graveyard. Returns the three cards, the top three
/// cards of the library afterwards, and which of the three are in the graveyard.
fn gaze(
    top: &[usize],
    graveyard: &[usize],
) -> (Vec<ObjectId>, Vec<ObjectId>, Vec<ObjectId>) {
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P0, &["Swamp", "Island", "Forest"]);
    t.lands(P0, "Island", 1);
    let g = t.hand(P0, "Otherworldly Gaze");
    t.answer(
        P0,
        DecisionKind::Surveil,
        Answer::Split(
            top.iter().map(|i| cards[*i]).collect(),
            graveyard.iter().map(|i| cards[*i]).collect(),
        ),
    );
    t.cast(P0, g).go();
    t.resolve_all();
    let yard = cards
        .iter()
        .copied()
        .filter(|c| t.zone(*c) == mtg_engine::object::Zone::Graveyard(P0))
        .collect();
    (cards, library_top_n(&t, P0, 3), yard)
}

#[test]
fn surveil_may_keep_all_mill_all_or_split_the_cards() {
    cr!("701.25a");
    ruling!(
        "Otherworldly Gaze",
        "When you surveil, you may put all the cards you look at back on top of your library, you may put all of those cards into your graveyard, or you may put some of those cards on top and the rest of them into your graveyard."
    );
    supported("Otherworldly Gaze");
    // All back on top, in any order.
    let (c, top, yard) = gaze(&[2, 0, 1], &[]);
    assert_eq!(top, vec![c[2], c[0], c[1]]);
    assert!(yard.is_empty());
    // All into the graveyard.
    let (c, top, yard) = gaze(&[], &[0, 1, 2]);
    assert!(top.iter().all(|x| !c.contains(x)));
    assert_eq!(yard.len(), 3);
    // Some on top, the rest into the graveyard.
    let (c, top, yard) = gaze(&[1], &[0, 2]);
    assert_eq!(top[0], c[1]);
    assert!(!top.contains(&c[0]) && !top.contains(&c[2]));
    assert_eq!(yard.len(), 2);
}

#[test]
fn a_cards_instructions_are_followed_in_order() {
    cr!("608.2c", "701.25a");
    ruling!(
        "Consider",
        "You perform the actions stated on a card in sequence. For some spells and abilities, you'll surveil last. For others, you'll surveil and then perform other actions."
    );
    supported("Consider");
    // Consider: "Surveil 1. Draw a card." The surveilled card goes to the graveyard, then
    // the card under it is drawn.
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P0, &["Swamp", "Island"]);
    t.lands(P0, "Island", 1);
    let c = t.hand(P0, "Consider");
    t.answer(
        P0,
        DecisionKind::Surveil,
        Answer::Split(vec![], vec![cards[0]]),
    );
    t.cast(P0, c).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Swamp"));
    assert!(t.in_hand(P0, "Island"));

    // Price of Fame: "Destroy target creature. Surveil 2." The creature is already
    // destroyed when P0 surveils.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let seen = watch(&mut t, P0, is_surveil, |g| {
        g.find_in_zone(mtg_engine::object::Zone::Graveyard(P1), "Grizzly Bears")
            .len()
    });
    t.lands(P0, "Swamp", 4);
    let pof = t.hand(P0, "Price of Fame");
    t.cast(P0, pof).target(bears).go();
    t.resolve_all();
    assert_eq!(*seen.lock().unwrap(), vec![1]);
}

#[test]
fn if_its_target_is_illegal_the_spell_doesnt_resolve_and_you_dont_surveil() {
    cr!("608.2b", "701.25a");
    ruling!(
        "Price of Fame",
        "Some spells that instruct you to surveil require targets. You can't cast a spell without choosing legal targets. If all of those targets become illegal, the spell doesn't resolve and you won't surveil."
    );
    supported("Price of Fame");
    let mut t = TestGame::new(2);
    // No creature to target: it can't be cast.
    t.lands(P0, "Swamp", 4);
    let pof = t.hand(P0, "Price of Fame");
    assert!(t.cast(P0, pof).try_go().is_err());
    t.clear_answers();
    assert!(t.in_hand(P0, "Price of Fame"));
    // Its target leaves before it resolves: no surveil.
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = t.cast(P0, pof).target(bears).go();
    destroy(&mut t, bears);
    let from = t.asked().len();
    t.resolve_all();
    assert!(!resolved(&t, spell));
    assert!(!t.asked()[from..].iter().any(|(_, d)| is_surveil(d)));
    assert!(t.in_graveyard(P0, "Price of Fame"));
}

#[test]
fn a_card_not_put_into_the_graveyard_stays_on_top() {
    cr!("701.25a");
    ruling!(
        "Uurg, Spawn of Turg",
        "If you don't put the card into your graveyard, it stays on top of your library."
    );
    supported("Uurg, Spawn of Turg");
    // Uurg: "At the beginning of your upkeep, surveil 1."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Uurg, Spawn of Turg");
    let top = t.library_top(P0, "Mountain");
    t.answer(P0, DecisionKind::Surveil, Answer::Split(vec![top], vec![]));
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(library_top_n(&t, P0, 1), vec![top]);
    assert!(!t.in_graveyard(P0, "Mountain"));
    // It's the card drawn in the draw step.
    t.advance_to(P0, Step::Draw);
    assert!(t.in_hand(P0, "Mountain"));
}

#[test]
fn you_surveil_before_the_destroyed_creatures_dies_triggers_resolve() {
    cr!("603.3", "608.2c");
    ruling!(
        "Deadly Visit",
        "You'll surveil before resolving any abilities that trigger on the target creature dying."
    );
    supported("Deadly Visit");
    supported("Doomed Traveler");
    // Deadly Visit: "Destroy target creature. Surveil 2." Doomed Traveler: "When this
    // creature dies, create a 1/1 white Spirit creature token with flying."
    let mut t = TestGame::new(2);
    let traveler = t.battlefield(P1, "Doomed Traveler");
    let seen = watch(&mut t, P0, is_surveil, |g| {
        (
            g.permanents().filter(|o| o.chars.has_subtype("Spirit")).count(),
            g.stack.len(),
        )
    });
    t.lands(P0, "Swamp", 5);
    let dv = t.hand(P0, "Deadly Visit");
    t.cast(P0, dv).target(traveler).go();
    t.resolve_all();
    // When P0 surveilled, the Spirit hadn't been created and the dies trigger wasn't on
    // the stack yet (only Deadly Visit, resolving, was).
    assert_eq!(*seen.lock().unwrap(), vec![(0, 1)]);
    assert_eq!(with_subtype(&t, P1, "Spirit").len(), 1);
}
