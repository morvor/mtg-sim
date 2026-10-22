//! Rulings batch P226 — "At the beginning of your end step, if ~ didn't attack this
//! turn, ..." (CR 603.4, 508.1): Homicidal Brute and Air Nomad Student.

use crate::r_s01_common::*;
use crate::r_s08_common::is_tapped;
use crate::r_s17_common::{enter_transformed, name_of};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const SCHOLAR: &str = "Civilized Scholar // Homicidal Brute";

#[test]
fn homicidal_brute_taps_and_transforms_even_if_it_couldnt_attack() {
    cr!("603.4", "508.1", "701.27a");
    ruling!(
        "Civilized Scholar // Homicidal Brute",
        "You'll tap and transform Homicidal Brute even if it couldn't attack."
    );
    // Homicidal Brute: "At the beginning of your end step, if this creature didn't attack this turn, tap this
    // creature, then transform it." It entered this turn: it couldn't attack.
    let mut t = TestGame::new(2);
    let brute = enter_transformed(&mut t, P0, SCHOLAR);
    assert_eq!(name_of(&t, brute), "Homicidal Brute");
    assert!(t.obj_now(brute).summoning_sick);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(name_of(&t, brute), "Civilized Scholar");
    assert!(is_tapped(&t, brute));
    // It attacked: nothing.
    let mut t = TestGame::new(2);
    let brute = enter_transformed(&mut t, P0, SCHOLAR);
    t.g.objects[brute.0 as usize].summoning_sick = false;
    t.attack(&[(brute, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 15);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(name_of(&t, brute), "Homicidal Brute");
}

#[test]
fn air_nomad_student_grows_on_turns_it_doesnt_attack() {
    cr!("603.4", "508.1", "702.9b");
    supported("Air Nomad Student");
    // Flying. "At the beginning of your end step, if this creature didn't attack this
    // turn, put a +1/+1 counter on it."
    let mut t = TestGame::new(2);
    let student = t.battlefield(P0, "Air Nomad Student");
    assert!(t.obj_now(student).has_keyword(KeywordKind::Flying));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(student, counters::PLUS1), 1);
    // Attacking: no counter.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(student, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.counters(student, counters::PLUS1), 1);
}

#[test]
fn civilized_scholar_untaps_and_transforms_without_a_chance_to_loot_again() {
    cr!("608.2c", "117.3", "701.27a");
    ruling!(
        "Civilized Scholar // Homicidal Brute",
        "You don't have priority between untapping Civilized Scholar and transforming it."
    );
    // "{T}: Draw a card, then discard a card. If a creature card is discarded this way,
    // untap this creature, then transform it." Both happen as the ability resolves: by
    // the time anyone has priority it is an untapped Homicidal Brute, whose draw-and-
    // discard ability is gone.
    let mut t = TestGame::new(2);
    let scholar = t.battlefield(P0, SCHOLAR);
    t.library_top(P0, "Island");
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, scholar, 0, &[]).unwrap();
    assert!(is_tapped(&t, scholar));
    t.resolve();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(name_of(&t, scholar), "Homicidal Brute");
    assert!(!is_tapped(&t, scholar));
    assert!(!crate::r_s02_common::can_activate(&mut t, P0, scholar));
    // A noncreature card discarded: it stays a tapped Civilized Scholar.
    let mut t = TestGame::new(2);
    let scholar = t.battlefield(P0, SCHOLAR);
    t.library_top(P0, "Grizzly Bears");
    let island = t.hand(P0, "Island");
    t.answer_choose(P0, &[Entity::Object(island)]);
    t.activate(P0, scholar, 0, &[]).unwrap();
    t.resolve();
    assert!(t.in_graveyard(P0, "Island"));
    assert_eq!(name_of(&t, scholar), "Civilized Scholar");
    assert!(is_tapped(&t, scholar));
}

#[test]
fn homicidal_brute_remembers_that_civilized_scholar_attacked() {
    cr!("603.4", "508.1", "712.18");
    ruling!(
        "Civilized Scholar // Homicidal Brute",
        "If Civilized Scholar attacks, and later in the turn (but before the beginning of your end step), it transforms, Homicidal Brute's last ability won't trigger."
    );
    let mut t = TestGame::new(2);
    let scholar = t.battlefield(P0, SCHOLAR);
    t.attack(&[(scholar, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 20);
    let id = t.g.current(scholar);
    assert!(mtg_engine::dfc::transform(&mut t.g, id));
    t.settle();
    assert_eq!(name_of(&t, scholar), "Homicidal Brute");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(name_of(&t, scholar), "Homicidal Brute");
}
