//! Rulings batch P035 — static abilities and replacement effects of "counters matter"
//! permanents: Winding Constrictor, The Magic Mirror, and maximum hand size setters
//! (Twenty-Toed Toad, Midnight Oil).

use crate::r_p035_common::*;
use crate::r_s01_common::{give_mana_for, supported, watch};
use crate::r_s04_common::untapped_lands;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

const CONSTRICTOR: &str = "Winding Constrictor";

#[test]
fn winding_constrictor_adds_one_to_entering_counters() {
    cr!("614.1c", "614.12");
    ruling!(
        "Winding Constrictor",
        "If an artifact or creature you control would enter the battlefield with a number of any kind of counters on it, it enters with that many plus one instead."
    );
    supported(CONSTRICTOR);
    supported("Servant of the Scale");
    supported("Everflowing Chalice");
    let mut t = TestGame::new(2);
    t.battlefield(P0, CONSTRICTOR);
    // A creature: "This creature enters with a +1/+1 counter on it."
    let servant = t.enter(P0, "Servant of the Scale");
    assert_eq!(t.counters(servant, counters::PLUS1), 2);
    // An artifact with charge counters (Everflowing Chalice kicked twice).
    give_mana_for(&mut t, P0, "Everflowing Chalice");
    t.lands(P0, "Wastes", 4);
    let chalice = t.hand(P0, "Everflowing Chalice");
    t.answer(
        P0,
        DecisionKind::OptionalCost,
        mtg_engine::decision::Answer::Number(2),
    );
    t.cast(P0, chalice).go();
    t.resolve_all();
    let chalice = t.named_on_battlefield("Everflowing Chalice")[0];
    assert_eq!(t.counters(chalice, counters::CHARGE), 3, "two plus one");
    // An opponent's creature isn't affected.
    let theirs = t.enter(P1, "Servant of the Scale");
    assert_eq!(t.counters(theirs, counters::PLUS1), 1);
}

#[test]
fn winding_constrictor_applies_to_each_instruction() {
    cr!("614.1a", "614.6");
    ruling!(
        "Winding Constrictor",
        "Winding Constrictor's effect applies to each of those instructions."
    );
    supported("Lifecrafter's Gift");
    let mut t = TestGame::new(2);
    t.battlefield(P0, CONSTRICTOR);
    let target = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    let plain = t.battlefield(P0, "Elite Vanguard");
    put(&mut t, other, counters::PLUS1, 1);
    assert_eq!(t.counters(other, counters::PLUS1), 2, "1 + 1");
    give_mana_for(&mut t, P0, "Lifecrafter's Gift");
    let gift = t.hand(P0, "Lifecrafter's Gift");
    t.cast(P0, gift).target(target).go();
    t.resolve_all();
    // "Put a +1/+1 counter on target creature" (1+1), "then put a +1/+1 counter on each
    // creature you control with a +1/+1 counter on it" (1+1 again).
    assert_eq!(t.counters(target, counters::PLUS1), 4);
    assert_eq!(t.counters(other, counters::PLUS1), 4);
    assert_eq!(t.counters(plain, counters::PLUS1), 0);
}

#[test]
fn two_winding_constrictors_add_two() {
    cr!("614.5", "616.1");
    ruling!(
        "Winding Constrictor",
        "If you control two Winding Constrictors, the number of counters placed on the artifact or creature is the original number plus two."
    );
    for n in 2..=3u32 {
        let mut t = TestGame::new(2);
        for _ in 0..n {
            t.battlefield(P0, CONSTRICTOR);
        }
        let bears = t.battlefield(P0, "Grizzly Bears");
        put(&mut t, bears, counters::PLUS1, 1);
        assert_eq!(
            t.counters(bears, counters::PLUS1),
            1 + n,
            "{n} Constrictors"
        );
    }
}

#[test]
fn winding_constrictor_doesnt_apply_to_itself_or_permanents_entering_with_it() {
    cr!("614.12");
    ruling!(
        "Winding Constrictor",
        "Winding Constrictor's effect can't apply to itself as it's entering the battlefield or to any other permanent entering the battlefield at the same time as it."
    );
    supported("Tromell, Seymour's Butler");
    supported("Raise the Past");
    // Tromell: "Each other nontoken creature you control enters with an additional +1/+1
    // counter on it." The entering Constrictor gets just that one counter.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tromell, Seymour's Butler");
    let snake = t.enter(P0, CONSTRICTOR);
    assert_eq!(t.counters(snake, counters::PLUS1), 1);
    // Entering at the same time (Raise the Past): Servant of the Scale gets one counter.
    let mut t = TestGame::new(2);
    t.graveyard(P0, CONSTRICTOR);
    t.graveyard(P0, "Servant of the Scale");
    give_mana_for(&mut t, P0, "Raise the Past");
    let raise = t.hand(P0, "Raise the Past");
    t.cast(P0, raise).go();
    t.resolve_all();
    let servant = t.named_on_battlefield("Servant of the Scale")[0];
    assert!(!t.named_on_battlefield(CONSTRICTOR).is_empty());
    assert_eq!(t.counters(servant, counters::PLUS1), 1);
}

#[test]
fn magic_mirror_counts_a_split_instant_sorcery_card_once() {
    cr!("709.4c", "601.2f");
    ruling!(
        "The Magic Mirror",
        "If a split card is both an instant card and a sorcery card, it's only counted once"
    );
    supported("The Magic Mirror");
    let mut t = TestGame::new(2);
    // Discovery (sorcery) // Dispersal (instant), and Lightning Bolt: two cards.
    t.graveyard(P0, "Discovery // Dispersal");
    t.graveyard(P0, "Lightning Bolt");
    t.lands(P0, "Island", 3);
    t.lands(P0, "Wastes", 6);
    let mirror = t.hand(P0, "The Magic Mirror");
    t.cast(P0, mirror).go();
    // {6}{U}{U}{U} minus 2.
    assert_eq!(untapped_lands(&t, P0), 2);
}

#[test]
fn twenty_toed_toad_and_spellbook_apply_in_timestamp_order() {
    cr!("402.2", "613.7", "613.7d");
    ruling!(
        "Twenty-Toed Toad",
        "If multiple effects modify your hand size, apply them in timestamp order."
    );
    supported("Twenty-Toed Toad");
    supported("Spellbook");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Twenty-Toed Toad");
    t.battlefield(P0, "Spellbook");
    t.g.recompute();
    assert_eq!(t.g.player(P0).max_hand_size, None, "Toad, then Spellbook");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spellbook");
    t.battlefield(P0, "Twenty-Toed Toad");
    t.g.recompute();
    assert_eq!(
        t.g.player(P0).max_hand_size,
        Some(20),
        "Spellbook, then Toad"
    );
}

#[test]
fn midnight_oil_latest_hand_size_effect_wins() {
    cr!("402.2", "613.7");
    ruling!(
        "Midnight Oil",
        "If multiple effects try to set your maximum hand size to a certain number or state that you have no maximum hand size, the one created latest is the one that applies."
    );
    supported("Midnight Oil");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Twenty-Toed Toad");
    let oil = t.enter(P0, "Midnight Oil");
    assert_eq!(t.counters(oil, "hour"), 7);
    t.g.recompute();
    assert_eq!(t.g.player(P0).max_hand_size, Some(7), "Toad, then Oil");
    let mut t = TestGame::new(2);
    t.enter(P0, "Midnight Oil");
    t.battlefield(P0, "Spellbook");
    t.g.recompute();
    assert_eq!(t.g.player(P0).max_hand_size, None, "Oil, then Spellbook");
    t.battlefield(P0, "Twenty-Toed Toad");
    t.g.recompute();
    assert_eq!(t.g.player(P0).max_hand_size, Some(20), "then Toad");
}

/// Wraps P0's agent: the first time P0 gets priority in a cleanup step, P0 casts `card`
/// (Inspiration, targeting P0); otherwise the scripted answers are used.
fn cast_in_cleanup(t: &mut TestGame, card: ObjectId) {
    use mtg_engine::decision::{Action, Agent, Answer, PassiveAgent};
    use mtg_engine::object::CastMethod;
    struct InCleanup {
        inner: Box<dyn Agent>,
        card: Option<ObjectId>,
    }
    impl Agent for InCleanup {
        fn decide(&mut self, g: &mtg_engine::game::Game, p: PlayerId, d: &Decision) -> Answer {
            if matches!(d, Decision::Priority { .. }) && g.turn.step == Step::Cleanup {
                if let Some(card) = self.card.take() {
                    return Answer::Action(Action::Cast {
                        card,
                        method: CastMethod::Normal,
                    });
                }
            }
            if let Decision::ChooseTargets { .. } = d {
                return Answer::Entities(vec![Entity::Player(PlayerId(0))]);
            }
            self.inner.decide(g, p, d)
        }
    }
    let mut agents = t.g.agents.0.lock().unwrap();
    let inner = std::mem::replace(&mut agents[0], Box::new(PassiveAgent));
    agents[0] = Box::new(InCleanup {
        inner,
        card: Some(card),
    });
}

#[test]
fn midnight_oil_discarding_in_cleanup_gives_another_cleanup_step() {
    cr!("514.1", "514.3a");
    ruling!(
        "Midnight Oil",
        "If you discard a card during your cleanup step while you control Midnight Oil, its last ability will trigger, and players will get priority in your cleanup step. You'll have another cleanup step after that one"
    );
    let mut t = TestGame::new(2);
    let oil = t.enter(P0, "Midnight Oil");
    // Two hour counters: maximum hand size 2.
    let o = t.g.current(oil);
    t.g.remove_counters(Entity::Object(o), "hour", 5);
    t.g.recompute();
    t.lands(P0, "Island", 4);
    let inspiration = t.hand(P0, "Inspiration");
    let bears: Vec<_> = (0..3).map(|_| t.hand(P0, "Grizzly Bears")).collect();
    // The first cleanup step: four cards, so two Bears are discarded (Inspiration kept).
    t.answer_choose(P0, &[Entity::Object(bears[0]), Entity::Object(bears[1])]);
    cast_in_cleanup(&mut t, inspiration);
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::Priority { .. }),
        |g| (g.turn.active, g.turn.step, g.stack.len()),
    );
    t.advance_to(P1, Step::Upkeep);
    let cleanup: Vec<_> = seen
        .lock()
        .unwrap()
        .iter()
        .filter(|(a, s, _)| *a == P0 && *s == Step::Cleanup)
        .cloned()
        .collect();
    assert!(
        cleanup.iter().any(|(_, _, stack)| *stack > 0),
        "priority in the cleanup step with the discard triggers on the stack: {cleanup:?}"
    );
    assert!(t.in_graveyard(P0, "Inspiration"), "cast during cleanup");
    // Inspiration drew two cards (hand 3): the next cleanup step discarded one more, and
    // that discard triggered too.
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.life(P0), 17, "three discards in two cleanup steps");
}
