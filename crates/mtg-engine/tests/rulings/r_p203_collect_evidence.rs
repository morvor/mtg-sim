//! Rulings batch P203 — collect evidence (CR 701.59): "whenever you collect evidence"
//! triggers, Analyze the Pollen's search, and the reflexive triggers of Sample Collector
//! and Memory Vampire.

use crate::r_s01_common::*;
use crate::r_s02_common::target_candidates;
use crate::r_s03_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

fn clues(t: &TestGame) -> usize {
    tokens(t, P0)
        .into_iter()
        .filter(|id| t.obj(*id).chars.has_subtype("Clue"))
        .count()
}

/// Casts Analyze the Pollen for P0, collecting evidence 8 with the Colossal Dreadmaw and
/// Hill Giant in P0's graveyard if `collect`, and finding `find` in P0's library.
fn analyze(t: &mut TestGame, collect: bool, find: ObjectId) -> usize {
    let dread = t.graveyard(P0, "Colossal Dreadmaw");
    let giant = t.graveyard(P0, "Hill Giant");
    let c = in_hand_with_mana(t, P0, "Analyze the Pollen");
    if collect {
        t.answer_choose(P0, &[Entity::Object(dread), Entity::Object(giant)]);
    }
    let from = t.asked().len();
    t.cast(P0, c).kicked(collect).go();
    t.answer_choose(P0, &[Entity::Object(find)]);
    t.resolve_all();
    from
}

#[test]
fn evidence_examiner_triggers_whenever_you_collect_evidence_for_any_reason() {
    cr!("701.59a", "603.2");
    ruling!(
        "Evidence Examiner",
        "Evidence Examiner's last ability triggers whenever you collect evidence for any reason, not just when you collect evidence with its first ability."
    );
    supported("Evidence Examiner");
    supported("Analyze the Pollen");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Evidence Examiner");
    let forest = t.library_top(P0, "Forest");
    analyze(&mut t, true, forest);
    assert_eq!(t.g.player(P0).graveyard.len(), 1); // the Pollen
    assert_eq!(clues(&t), 1);
    // Not collecting evidence: no Clue.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Evidence Examiner");
    let forest = t.library_top(P0, "Forest");
    analyze(&mut t, false, forest);
    assert_eq!(clues(&t), 0);
}

#[test]
fn analyze_the_pollen_with_evidence_can_find_a_nonbasic_land() {
    cr!("701.59a", "701.23a", "205.4c");
    ruling!(
        "Analyze the Pollen",
        "The collect evidence ability of Analyze the Pollen allows you to find a nonbasic land card."
    );
    let mut t = TestGame::new(2);
    let factory = t.library_top(P0, "Mishra's Factory");
    t.library_top(P0, "Forest");
    analyze(&mut t, true, factory);
    assert!(t.in_hand(P0, "Mishra's Factory"));
    // Without evidence, only a basic land card can be found.
    let mut t = TestGame::new(2);
    let factory = t.library_top(P0, "Mishra's Factory");
    let forest = t.library_top(P0, "Forest");
    let from = analyze(&mut t, false, factory);
    assert!(!t.in_hand(P0, "Mishra's Factory"));
    let offered: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert!(offered.iter().any(|c| c.contains(&Entity::Object(forest))));
    assert!(!offered.iter().any(|c| c.contains(&Entity::Object(factory))));
}

#[test]
fn sample_collectors_target_is_chosen_for_a_reflexive_trigger() {
    cr!("603.12", "701.59a", "117.3b");
    ruling!(
        "Sample Collector",
        "You don't choose a target for Sample Collector's ability at the time it triggers. Rather, a second \"reflexive\" ability triggers when you collect evidence 3 this way. You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    supported("Sample Collector");
    let mut t = TestGame::new(2);
    let sc = t.battlefield(P0, "Sample Collector");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.graveyard(P0, "Hill Giant");
    let from = t.asked().len();
    attack_with(&mut t, &[(sc, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    // No target when it triggered.
    assert!(target_candidates(&t, P0, from).is_empty());
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let from = t.asked().len();
    t.resolve();
    // The evidence was collected, then the reflexive trigger went on the stack with its
    // target, and P1 gets to respond.
    assert!(t.in_exile("Hill Giant"));
    assert_eq!(t.stack_len(), 1);
    assert_eq!(target_candidates(&t, P0, from).len(), 1);
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
    let from = t.asked().len();
    assert!(t.g.run_until(1_000, |g| g.stack.is_empty()));
    assert!(priority_asked_since(&t, from).contains(&P1));
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
}

#[test]
fn memory_vampire_needs_no_target_players_and_its_cast_is_a_reflexive_trigger() {
    cr!("603.12", "701.59a", "115.1d", "601.2c");
    ruling!(
        "Memory Vampire",
        "You don't have to choose any target players for Memory Vampire's last ability if you don't want to. In that case, the ability will still resolve and you'll have the opportunity to collect evidence 9."
    );
    ruling!(
        "Memory Vampire",
        "You don't choose a target nonland card at the time Memory Vampire's last ability triggers. Rather, a second \"reflexive\" ability triggers when you collect evidence 9 this way. You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    supported("Memory Vampire");
    let mut t = TestGame::new(2);
    let mv = t.battlefield(P0, "Memory Vampire");
    let dread = t.graveyard(P0, "Colossal Dreadmaw");
    let giant = t.graveyard(P0, "Hill Giant");
    let div = t.graveyard(P1, "Divination");
    let (l0, l1) = (t.library_size(P0), t.library_size(P1));
    attack_with(&mut t, &[(mv, Entity::Player(P1))]);
    // Choose no target players for the combat damage trigger.
    t.answer_targets(P0, &[]);
    let ok = t.g.run_until(1_000, |g| !g.stack.is_empty());
    assert!(ok);
    assert_eq!(t.life(P1), 16);
    // No targets were chosen: no nonland card either.
    let si = t.obj(t.g.stack[0]).stack.as_ref().unwrap().clone();
    assert!(si.chosen.iter().all(|cm| cm.targets.iter().all(|s| s.is_empty())));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(dread), Entity::Object(giant)]);
    t.answer_targets(P0, &[Entity::Object(div)]);
    t.resolve();
    // It resolved: nobody milled; the evidence was collected; the reflexive trigger is on
    // the stack targeting Divination.
    assert_eq!(t.library_size(P1), l1);
    assert!(t.in_exile("Colossal Dreadmaw") && t.in_exile("Hill Giant"));
    assert_eq!(t.stack_len(), 1);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    assert!(t.g.run_until(1_000, |g| g.stack.is_empty()));
    assert!(priority_asked_since(&t, from).contains(&P1));
    // P0 cast Divination: two cards drawn.
    assert_eq!(t.library_size(P0), l0 - 2);
    assert!(t.in_graveyard(P1, "Divination"));
}
