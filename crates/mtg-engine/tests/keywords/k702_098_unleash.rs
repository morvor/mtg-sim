//! CR 702.98 Unleash.

use crate::common_k702_011_017::{assert_supported, attack_with, is_blocking};
use crate::common_k702_018_026::declare_blocks;
use crate::common_k702_052_066::yes_no_asked;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn an_unleashed_creature_enters_with_a_counter_and_cant_block() {
    cr!("702.98", "702.98a");
    ruling!(
        "Rakdos Cackler",
        "You make the choice to have the creature with unleash enter the battlefield with a +1/+1 counter or not as it’s entering the battlefield."
    );
    assert_supported("Rakdos Cackler");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let cackler = t.hand(P0, "Rakdos Cackler");
    t.cast(P0, cackler).go();
    t.settle();
    // Not asked while the spell is on the stack.
    assert_eq!(yes_no_asked(&t, P0, "Unleash"), 0);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(yes_no_asked(&t, P0, "Unleash"), 1);
    assert_eq!(t.counters(cackler, counters::PLUS1), 1);
    assert_eq!(t.pt(cackler), (2, 2));
    let cackler = t.g.current(cackler);
    assert!(!t.g.can_block_at_all(cackler));
    // It can still attack.
    t.g.objects[cackler.0 as usize].summoning_sick = false;
    attack_with(&mut t, &[(cackler, Entity::Player(P1))]);
    assert!(t.g.is_attacking(cackler));
}

#[test]
fn a_creature_that_isnt_unleashed_can_block() {
    cr!("702.98a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let cackler = t.hand(P0, "Rakdos Cackler");
    t.cast(P0, cackler).go();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.counters(cackler, counters::PLUS1), 0);
    assert_eq!(t.pt(cackler), (1, 1));
    // P1 attacks; the Cackler blocks.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    let cackler = t.g.current(cackler);
    declare_blocks(&mut t, P0, &[(cackler, bears)]);
    assert!(is_blocking(&t, cackler));
}

#[test]
fn unleash_applies_wherever_the_creature_enters_from() {
    cr!("702.98a");
    ruling!(
        "Rakdos Cackler",
        "The unleash ability applies no matter where the creature is entering the battlefield from."
    );
    let mut t = TestGame::new(2);
    let dead = t.graveyard(P0, "Gore-House Chainwalker");
    t.lands(P0, "Swamp", 1);
    let reanimate = t.hand(P0, "Reanimate");
    t.cast(P0, reanimate).target(dead).go();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(dead));
    assert_eq!(t.counters(dead, counters::PLUS1), 1);
    assert_eq!(t.pt(dead), (3, 2));
}

#[test]
fn any_plus_one_counter_stops_it_from_blocking() {
    cr!("702.98a");
    ruling!(
        "Rakdos Cackler",
        "A creature with unleash can’t block if it has any +1/+1 counter on it, not just one put on it by the unleash ability."
    );
    let mut t = TestGame::new(2);
    let cackler = t.battlefield(P0, "Rakdos Cackler");
    assert!(t.g.can_block_at_all(cackler));
    t.g.add_counters(Entity::Object(cackler), counters::PLUS1, 1, None);
    t.g.recompute();
    assert!(!t.g.can_block_at_all(cackler));
    // Other kinds of counters don't matter.
    let other = t.battlefield(P0, "Rakdos Cackler");
    t.g.add_counters(Entity::Object(other), counters::MINUS1, 1, None);
    t.g.recompute();
    assert!(t.g.can_block_at_all(other));
}

#[test]
fn a_blocking_creature_that_gets_a_counter_keeps_blocking() {
    cr!("702.98a");
    ruling!(
        "Rakdos Cackler",
        "Putting a +1/+1 counter on a creature with unleash that’s already blocking won’t remove it from combat. It will continue to block."
    );
    let mut t = TestGame::new(2);
    let cackler = t.battlefield(P0, "Rakdos Cackler");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    declare_blocks(&mut t, P0, &[(cackler, bears)]);
    assert!(is_blocking(&t, cackler));
    t.g.add_counters(Entity::Object(cackler), counters::PLUS1, 1, None);
    t.g.recompute();
    assert!(is_blocking(&t, cackler));
    t.advance_to(P1, Step::EndOfCombat);
    // The 2/2 Cackler and the Bears traded.
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P0), 20);
}
