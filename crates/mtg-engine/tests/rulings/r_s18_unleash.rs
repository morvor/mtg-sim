//! Rulings batch S18 — unleash (CR 702.98): "You may have this permanent enter with an
//! additional +1/+1 counter on it" and "This permanent can't block as long as it has a
//! +1/+1 counter on it."

use crate::r_s01_common::*;
use crate::r_s03_common::{in_hand_with_mana, to_blockers};
use crate::r_s10_common::blocking;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// The unleash choices asked of `p` so far.
fn unleash_asked(t: &TestGame, p: PlayerId) -> usize {
    t.asked()
        .iter()
        .filter(|(q, d)| {
            *q == p && matches!(d, Decision::YesNo { prompt, .. } if prompt.contains("Unleash"))
        })
        .count()
}

#[test]
fn the_unleash_choice_is_made_as_it_enters_too_late_to_respond() {
    cr!("702.98a", "614.12a", "608.3");
    ruling!(
        "Hellhole Flailer",
        "You make the choice to have the creature with unleash enter the battlefield with a +1/+1 counter or not as it's entering the battlefield. At that point, it's too late for a player to respond to the creature spell by trying to counter it, for example."
    );
    supported("Hellhole Flailer");
    // P1 responds to the creature spell by countering it: the choice was never made.
    let mut t = TestGame::new(2);
    let flailer = in_hand_with_mana(&mut t, P0, "Hellhole Flailer");
    let spell = t.cast(P0, flailer).go();
    assert_eq!(unleash_asked(&t, P0), 0);
    t.lands(P1, "Island", 3);
    let cancel = t.hand(P1, "Cancel");
    t.cast(P1, cancel).target(spell).go();
    t.resolve_all();
    assert_eq!(unleash_asked(&t, P0), 0);
    assert!(t.in_graveyard(P0, "Hellhole Flailer"));
    // Unopposed, the choice is made as it resolves, and it enters with the counter.
    let mut t = TestGame::new(2);
    let flailer = in_hand_with_mana(&mut t, P0, "Hellhole Flailer");
    t.cast(P0, flailer).go();
    t.settle();
    assert_eq!(unleash_asked(&t, P0), 0);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(unleash_asked(&t, P0), 1);
    assert!(t.on_battlefield(flailer));
    assert_eq!(t.counters(flailer, counters::PLUS1), 1);
    assert_eq!(t.pt(flailer), (4, 3));
    assert!(!t.g.can_block_at_all(t.g.current(flailer)));
}

#[test]
fn any_plus_one_counter_stops_a_creature_with_unleash_from_blocking() {
    cr!("702.98a", "509.1b");
    ruling!(
        "Chaos Imps",
        "A creature with unleash can't block if it has any +1/+1 counter on it, not just one put on it by the unleash ability."
    );
    supported("Chaos Imps");
    supported("Travel Preparations");
    // Chaos Imps entered without the counter: it can block.
    let mut t = TestGame::new(2);
    let imps = in_hand_with_mana(&mut t, P0, "Chaos Imps");
    t.cast(P0, imps).go();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.counters(imps, counters::PLUS1), 0);
    let imps = t.g.current(imps);
    assert!(t.g.can_block_at_all(imps));
    assert!(!t.obj_now(imps).has_keyword(KeywordKind::Trample));
    // Travel Preparations puts a +1/+1 counter on it: it can't block (and has trample).
    t.lands(P0, "Forest", 2);
    let prep = t.hand(P0, "Travel Preparations");
    t.cast(P0, prep).targets(&[Entity::Object(imps)]).go();
    t.resolve_all();
    assert_eq!(t.counters(imps, counters::PLUS1), 1);
    assert!(!t.g.can_block_at_all(imps));
    assert!(t.obj_now(imps).has_keyword(KeywordKind::Trample));
    // P1 attacks: the Imps can't be declared as a blocker.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P0))], &[(imps, bears)]);
    assert_eq!(t.life(P0), 18);
    assert!(t.on_battlefield(bears));
}

#[test]
fn a_blocking_creature_with_unleash_that_gets_a_counter_keeps_blocking() {
    cr!("702.98a", "506.4", "509.1b");
    ruling!(
        "Tesak, Judith's Hellhound",
        "Putting a +1/+1 counter on a creature with unleash that's already blocking won't remove it from combat. It will continue to block."
    );
    supported("Tesak, Judith's Hellhound");
    supported("Battlegrowth");
    let mut t = TestGame::new(2);
    let tesak = t.battlefield(P0, "Tesak, Judith's Hellhound");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    to_blockers(&mut t, &[(bears, Entity::Player(P0))], &[(tesak, bears)]);
    assert!(blocking(&t, tesak));
    // Battlegrowth: a +1/+1 counter on the blocking Tesak.
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Battlegrowth");
    t.cast(P0, growth).target(tesak).go();
    t.resolve_all();
    assert_eq!(t.counters(tesak, counters::PLUS1), 1);
    assert!(!t.g.can_block_at_all(tesak));
    assert!(blocking(&t, tesak));
    t.advance_to(P1, Step::EndOfCombat);
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(tesak));
    assert_eq!(t.life(P0), 20);
}
