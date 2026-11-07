//! CR 702.24 Cumulative upkeep.

use crate::common_k702_011_017::*;
use crate::common_k702_018_026::*;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const CU: &str = "Cumulative Upkeep";

/// Advances to `p`'s next upkeep and puts the upkeep triggers on the stack.
fn next_upkeep(t: &mut TestGame, p: PlayerId) {
    let other = if p == P0 { P1 } else { P0 };
    if t.g.turn.active == p {
        t.advance_to(other, Step::Upkeep);
    }
    t.advance_to(p, Step::Upkeep);
    t.settle();
}

/// Number of "Pay ...?" questions asked of `p` so far.
fn pay_questions(t: &TestGame, p: PlayerId) -> usize {
    t.asked()
        .iter()
        .filter(|(q, d)| *q == p && matches!(d, Decision::YesNo { prompt, .. } if prompt.starts_with("Pay")))
        .count()
}

fn untapped_lands(t: &TestGame, p: PlayerId) -> usize {
    t.g.battlefield
        .iter()
        .filter(|id| {
            let o = t.g.obj(**id);
            o.controller == p && o.is(types::CardType::Land) && !o.tapped
        })
        .count()
}

#[test]
fn cumulative_upkeep_cost_grows_with_age_counters() {
    cr!("702.24", "702.24a");
    ruling!(
        "Arctic Wolves",
        "Paying cumulative upkeep is always optional. If it’s not paid, the permanent with cumulative upkeep is sacrificed. Partial payments of the total cumulative upkeep cost can’t be made"
    );
    assert_supported("Arctic Wolves");
    let mut t = TestGame::new(2);
    let wolves = t.battlefield(P0, "Arctic Wolves");
    t.lands(P0, "Forest", 3);
    // First upkeep: one age counter, pay {2}.
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, CU), 1);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(t.on_battlefield(wolves));
    assert_eq!(t.counters(wolves, "age"), 1);
    assert_eq!(untapped_lands(&t, P0), 1);
    // Second upkeep: two age counters, {4} — three lands can't pay it, and nothing is
    // paid partially.
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(!t.on_battlefield(wolves));
    assert!(t.in_graveyard(P0, "Arctic Wolves"));
    assert_eq!(untapped_lands(&t, P0), 3);
}

#[test]
fn declining_to_pay_sacrifices_it_after_the_age_counter() {
    cr!("702.24a");
    ruling!(
        "Revered Unicorn",
        "it will still get an age counter before being sacrificed"
    );
    let mut t = TestGame::new(2);
    let unicorn = t.battlefield(P0, "Revered Unicorn");
    t.lands(P0, "Plains", 2);
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, false);
    t.resolve();
    assert!(!t.on_battlefield(unicorn));
    assert!(t.in_graveyard(P0, "Revered Unicorn"));
    // As it last existed on the battlefield, it had an age counter.
    assert_eq!(t.g.obj(unicorn).counter("age"), 1);
    assert_eq!(untapped_lands(&t, P0), 2);
}

#[test]
fn it_triggers_only_in_its_controllers_upkeep() {
    cr!("702.24a");
    let mut t = TestGame::new(2);
    let wolves = t.battlefield(P0, "Arctic Wolves");
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, CU), 0);
    assert_eq!(t.counters(wolves, "age"), 0);
}

#[test]
fn nothing_happens_if_it_left_the_battlefield() {
    cr!("702.24a");
    let mut t = TestGame::new(2);
    let wolves = t.battlefield(P0, "Arctic Wolves");
    t.lands(P0, "Forest", 2);
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, CU), 1);
    t.g.destroy(wolves, None);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(pay_questions(&t, P0), 0);
    assert_eq!(untapped_lands(&t, P0), 2);
}

#[test]
fn a_choice_in_the_cost_is_made_for_each_age_counter() {
    cr!("702.24a");
    let mut t = TestGame::new(2);
    let nishoba = t.battlefield(P0, "Arctic Nishoba");
    t.g.add_counters(Entity::Object(nishoba), "age", 1, None);
    // Two age counters after this upkeep's: {G} or {W}, twice — pay {G}{W}.
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Plains", 1);
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(t.on_battlefield(nishoba));
    assert_eq!(t.counters(nishoba, "age"), 2);
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn a_life_cost_is_paid_once_per_age_counter() {
    cr!("702.24a");
    assert_supported("Decomposition");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Walking Corpse");
    let aura = t.battlefield(P0, "Decomposition");
    t.g.attach(aura, Entity::Object(bears));
    t.g.add_counters(Entity::Object(bears), "age", 2, None);
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.life(P0), 17);
}

#[test]
fn an_action_can_be_the_cost() {
    cr!("702.24a");
    ruling!("Braid of Fire", "\"Add {R}\" is a cost");
    assert_supported("Braid of Fire");
    let mut t = TestGame::new(2);
    let braid = t.battlefield(P0, "Braid of Fire");
    t.g.add_counters(Entity::Object(braid), "age", 2, None);
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(t.on_battlefield(braid));
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::R), 3);
}

#[test]
fn an_opponent_is_chosen_for_each_age_counter() {
    cr!("702.24a");
    ruling!(
        "Wall of Shards",
        "you may choose a different opponent for each age counter, or you can choose the same opponent multiple times"
    );
    assert_supported("Wall of Shards");
    let mut t = TestGame::new(3);
    // "Cumulative upkeep—An opponent gains 1 life."
    let wall = t.battlefield(P0, "Wall of Shards");
    t.g.add_counters(Entity::Object(wall), "age", 1, None);
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.resolve();
    assert!(t.on_battlefield(wall));
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (20, 21, 21));
}

#[test]
fn drawing_cards_as_the_cost() {
    cr!("702.24a");
    ruling!(
        "Psychic Vortex",
        "Psychic Vortex’s cumulative upkeep ability has you draw cards as a cost"
    );
    let mut t = TestGame::new(2);
    let vortex = t.battlefield(P0, "Psychic Vortex");
    t.g.add_counters(Entity::Object(vortex), "age", 1, None);
    let hand = t.hand_size(P0);
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(t.on_battlefield(vortex));
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn counters_put_on_as_the_cost() {
    cr!("702.24a");
    assert_supported("Aboroth");
    let mut t = TestGame::new(2);
    let aboroth = t.battlefield(P0, "Aboroth");
    for (upkeep, total) in [(1, 1), (2, 3), (3, 6)] {
        next_upkeep(&mut t, P0);
        t.answer_yes(P0, true);
        t.resolve();
        assert_eq!(t.counters(aboroth, "age"), upkeep);
        assert_eq!(t.counters(aboroth, "-1/-1"), total);
    }
    assert_eq!(t.pt(aboroth), (3, 3));
}

#[test]
fn each_instance_triggers_and_counts_all_age_counters() {
    cr!("702.24b");
    ruling!(
        "Mana Chains",
        "each cumulative upkeep ability will count the total number of age counters on the permanent at the time that ability resolves"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    for _ in 0..2 {
        let chains = t.battlefield(P0, "Mana Chains");
        t.g.attach(chains, Entity::Object(bears));
    }
    t.g.recompute();
    assert_eq!(keyword_count(&t, bears, KeywordKind::CumulativeUpkeep), 2);
    t.lands(P0, "Island", 3);
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, CU), 2);
    // The first puts on the first age counter and costs {1}.
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.counters(bears, "age"), 1);
    assert_eq!(untapped_lands(&t, P0), 2);
    // The second puts on the second one and costs {1} for each of the two.
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.counters(bears, "age"), 2);
    assert_eq!(untapped_lands(&t, P0), 0);
    assert!(t.on_battlefield(bears));
}
