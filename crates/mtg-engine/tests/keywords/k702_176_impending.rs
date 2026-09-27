//! CR 702.176 Impending.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_052_066::{remove_counters, run_effect};
use crate::common_k702_140_152::*;
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

const IMPENDING: CastMethod = CastMethod::Keyword(KeywordKind::Impending);
const FLOODPITS: &str = "Overlord of the Floodpits";

/// Casts Overlord of the Floodpits for its impending cost {1}{U}{U} and resolves it (and
/// its enters trigger). Returns the permanent.
fn impending_overlord(t: &mut TestGame) -> ObjectId {
    let card = t.hand(P0, FLOODPITS);
    add_mana(t, P0, ManaType::U, 3);
    t.cast(P0, card).method(IMPENDING).go();
    t.resolve_all();
    t.g.current(card)
}

fn time(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, "time")
}

#[test]
fn an_impending_permanent_enters_with_time_counters_and_isnt_a_creature() {
    cr!("702.176", "702.176a");
    assert_supported(FLOODPITS);
    ruling!(
        "Overlord of the Floodpits",
        "\"Impending N–[cost]\" is a keyword that represents multiple abilities. The official rules are as follows: (a) You may choose to pay [cost] rather than pay this spell's mana cost. (b) If you chose to pay this spell's impending cost, it enters the battlefield with N time counters on it. (c) As long as this permanent has a time counter on it, if it was cast for its impending cost, it's not a creature. (d) At the beginning of your end step, if this permanent was cast for its impending cost, remove a time counter from it."
    );
    // Overlord of the Floodpits: {3}{U}{U} 5/3 flying, impending 4—{1}{U}{U}, "Whenever
    // this permanent enters or attacks, draw two cards, then discard a card."
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    let card = t.hand(P0, FLOODPITS);
    add_mana(&mut t, P0, ManaType::U, 3);
    let spell = t.cast(P0, card).method(IMPENDING).go();
    // The alternative cost was paid; the mana value is still that of its mana cost, and
    // it's still a creature spell.
    assert_eq!(pool(&t, P0), 0);
    assert_eq!(t.g.mana_value_of(spell), 5);
    assert!(t.obj(spell).chars.is(CardType::Creature));
    let hand = t.hand_size(P0);
    t.resolve_all();
    let overlord = t.g.current(card);
    assert_eq!(time(&t, overlord), 4);
    assert!(!t.obj(overlord).is_creature());
    assert!(t.obj(overlord).chars.is(CardType::Enchantment));
    // It still entered: "draw two cards, then discard a card".
    assert_eq!(t.hand_size(P0), hand + 1);
    // At the beginning of each of its controller's end steps, a time counter is removed;
    // not in the opponent's.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(time(&t, overlord), 3);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(time(&t, overlord), 3);
    for left in [2, 1, 0] {
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert_eq!(time(&t, overlord), left);
        t.advance_to(P1, Step::Upkeep);
    }
    // With the last one removed, it's a creature.
    assert!(t.obj(overlord).is_creature());
    assert_eq!(t.pt(overlord), (5, 3));
    // No more counters to remove: the ability doesn't trigger.
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn cast_normally_it_is_a_creature_without_time_counters() {
    cr!("702.176a");
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    let card = t.hand(P0, FLOODPITS);
    add_mana(&mut t, P0, ManaType::U, 5);
    t.cast(P0, card).go();
    t.resolve_all();
    let overlord = t.g.current(card);
    assert_eq!(time(&t, overlord), 0);
    assert!(t.obj(overlord).is_creature());
    // Time counters put on it later don't make it stop being a creature.
    run_effect(
        &mut t,
        None,
        P0,
        Effect::AddCounters {
            what: Sel::Target(0),
            kind: "time".into(),
            n: Value::c(2),
        },
        &[Entity::Object(overlord)],
    );
    assert!(t.obj(overlord).is_creature());
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(time(&t, overlord), 2);
}

#[test]
fn it_isnt_a_creature_as_long_as_it_has_a_time_counter() {
    cr!("702.176a");
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    let overlord = impending_overlord(&mut t);
    // More time counters (e.g. proliferate): still not a creature.
    run_effect(
        &mut t,
        None,
        P0,
        Effect::AddCounters {
            what: Sel::Target(0),
            kind: "time".into(),
            n: Value::c(1),
        },
        &[Entity::Object(overlord)],
    );
    assert_eq!(time(&t, overlord), 5);
    assert!(!t.obj(overlord).is_creature());
    // All of them removed by another effect: a creature at once.
    remove_counters(&mut t, overlord, "time", 5);
    assert!(t.obj(overlord).is_creature());
}

#[test]
fn casting_for_the_impending_cost_is_still_casting_the_creature_spell() {
    cr!("702.176a");
    ruling!(
        "Overlord of the Hauntwoods",
        "If you choose to pay the impending cost of a creature spell, it's still a creature spell on the stack. You can cast that spell for its impending cost only when you could normally cast that creature spell."
    );
    ruling!(
        "Overlord of the Hauntwoods",
        "If you choose to pay the impending cost rather than the mana cost, you're still casting the spell. It goes on the stack and can be responded to, countered, and so on."
    );
    let mut t = TestGame::new(2);
    let card = t.hand(P0, FLOODPITS);
    add_mana(&mut t, P0, ManaType::U, 3);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!castable(&mut t, P0, card, IMPENDING));
    t.set_step(P0, Step::PrecombatMain);
    assert!(castable(&mut t, P0, card, IMPENDING));
    let spell = t.cast(P0, card).method(IMPENDING).go();
    let counter = t.hand(P1, "Counterspell");
    add_mana(&mut t, P1, ManaType::U, 2);
    t.cast(P1, counter).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, FLOODPITS));
}

#[test]
fn a_copy_of_an_impending_permanent_is_a_creature_without_time_counters() {
    cr!("702.176a", "707.2");
    assert_supported("Copy Enchantment");
    ruling!(
        "Overlord of the Hauntwoods",
        "If an object enters as a copy of a permanent that was cast with its impending cost, it won't enter with time counters, and it will be a creature."
    );
    let mut t = TestGame::new(2);
    for _ in 0..6 {
        t.library_top(P0, "Island");
    }
    let overlord = impending_overlord(&mut t);
    let copy = t.hand(P0, "Copy Enchantment");
    add_mana(&mut t, P0, ManaType::U, 3);
    t.answer_choose(P0, &[Entity::Object(overlord)]);
    t.cast(P0, copy).go();
    t.resolve_all();
    let copy = t.g.current(copy);
    assert_eq!(t.obj(copy).chars.name, FLOODPITS);
    assert_eq!(time(&t, copy), 0);
    assert!(t.obj(copy).is_creature());
    assert!(!t.obj(overlord).is_creature());
}
