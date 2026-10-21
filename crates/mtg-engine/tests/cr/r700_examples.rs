//! The worked examples of CR 700 (general game terms), played out as tests. The CR 700.3c
//! example is cited by a test in `r700_piles.rs`.

use crate::r600_examples_common::*;
use crate::r609_common::modify_target;
use mtg_engine::ability::Modification;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn being_blocked_by_two_creatures_is_one_event_or_two() {
    cr!("509.3c", "509.3d");
    cr_example!("700.1", "blocked by two creatures");
    let mut t = TestGame::new(2);
    let attacker = t.custom(
        P0,
        custom_card(
            "Twice Watched",
            "Creature — Beast",
            Some((5, 5)),
            "Whenever this creature becomes blocked, you gain 1 life.\nWhenever this creature becomes blocked by a creature, draw a card.",
        ),
        Zone::Battlefield,
    );
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let hand = t.hand_size(P0);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(attacker, Entity::Player(P1))], &[(a, attacker), (b, attacker)]);
    // One event for "becomes blocked", two for "becomes blocked by a creature".
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn this_creature_refers_to_the_object_even_after_it_stops_being_a_creature() {
    cr!("603.7a", "611.2a");
    cr_example!("700.7", "Destroy that creature at the beginning of the next end step");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    t.lands(P0, "Forest", 1);
    let s = t.custom(
        P0,
        custom_costed(
            "Last Hurrah",
            "Instant",
            "{G}",
            "Target creature gets +2/+2 until end of turn. Destroy that creature at the beginning of the next end step.",
        ),
        Zone::Hand(P0),
    );
    t.cast(P0, s).target(thopter).go();
    t.resolve();
    assert_eq!(t.pt(thopter), (2, 4));
    // It stops being a creature.
    modify_target(
        &mut t,
        P0,
        thopter,
        vec![Modification::RemoveTypes(vec![CardType::Creature])],
    );
    assert!(!t.obj_now(thopter).is_creature());
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Ornithopter"));
}

#[test]
fn bark_knuckle_boxer_triggers_as_divination_brings_the_total_to_five() {
    cr!("603.2");
    cr_example!("700.14", "Bark-Knuckle Boxer");
    supported("Bark-Knuckle Boxer");
    supported("Divination");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Island", 3);
    let boxer = t.hand(P0, "Bark-Knuckle Boxer");
    t.cast(P0, boxer).go();
    t.resolve_all();
    let boxer = t.named_on_battlefield("Bark-Knuckle Boxer")[0];
    // Two mana spent so far: no trigger.
    assert_eq!(t.stack_len(), 0);
    let div = t.hand(P0, "Divination");
    t.cast(P0, div).go();
    t.settle();
    // Five mana spent: the trigger is on the stack above Divination.
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert!(t.obj_now(boxer).has_keyword(mtg_engine::keywords::KeywordKind::Indestructible));
    t.resolve_all();
    // Only once: the fourth mana is spent only once in a turn.
    assert_eq!(t.stack_len(), 0);
}
