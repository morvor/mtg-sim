//! CR 702.154 Enlist.

use crate::common_k702_153_167::*;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::*;

/// Declares attackers for P0 (from the beginning of combat), choosing `enlisted` for each
/// enlist question in order, and stops in the declare attackers step with attack triggers
/// on the stack.
fn attack_enlisting(t: &mut TestGame, attackers: &[ObjectId], enlisted: &[Option<ObjectId>]) {
    t.set_step(P0, Step::BeginningOfCombat);
    let decl: Vec<(ObjectId, Entity)> = attackers
        .iter()
        .map(|a| (*a, Entity::Player(P1)))
        .collect();
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(decl));
    for e in enlisted {
        let v: Vec<Entity> = e.iter().map(|o| Entity::Object(*o)).collect();
        t.answer_choose(P0, &v);
    }
    let ok = t.g.run_until(10_000, |g| {
        g.turn.step == Step::DeclareAttackers
            && g.turn.stage == Stage::Priority
            && g.turn.priority == Some(P0)
    });
    assert!(ok, "attackers not declared");
    t.settle();
}

/// The creatures offered for an enlist question.
fn enlist_candidates(t: &TestGame) -> Vec<Vec<Entity>> {
    t.asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities {
                prompt, candidates, ..
            } if prompt.starts_with("Enlist") => Some(candidates),
            _ => None,
        })
        .collect()
}

#[test]
fn enlist_taps_a_creature_to_add_its_power() {
    cr!("702.154", "702.154a", "702.154b");
    ruling!(
        "Coalition Warbrute",
        "This is a triggered ability that goes on the stack immediately after attackers have been declared in the declare attackers step."
    );
    assert_supported("Coalition Warbrute");
    let mut t = TestGame::new(2);
    // Coalition Warbrute: 3/4 enlist, trample.
    let brute = t.battlefield(P0, "Coalition Warbrute");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_enlisting(&mut t, &[brute], &[Some(bears)]);
    assert!(t.obj_now(bears).tapped);
    assert_eq!(triggers_named(&t, "Enlist").len(), 1);
    // The bonus comes from the triggered ability.
    assert_eq!(t.pt(brute), (3, 4));
    t.resolve();
    assert_eq!(t.pt(brute), (5, 4));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 15);
    // Until end of turn.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(brute), (3, 4));
}

#[test]
fn enlisting_is_optional() {
    cr!("702.154a");
    ruling!(
        "Coalition Warbrute",
        "You can’t choose to enlist a creature later."
    );
    let mut t = TestGame::new(2);
    let brute = t.battlefield(P0, "Coalition Warbrute");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_enlisting(&mut t, &[brute], &[None]);
    assert!(!t.obj_now(bears).tapped);
    assert!(triggers_named(&t, "Enlist").is_empty());
    t.resolve_all();
    assert_eq!(t.pt(brute), (3, 4));
    // Nothing is asked again later in combat.
    let asked = enlist_candidates(&t).len();
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(enlist_candidates(&t).len(), asked);
}

#[test]
fn only_untapped_nonattacking_creatures_without_summoning_sickness_or_with_haste() {
    cr!("702.154a");
    ruling!(
        "Coalition Warbrute",
        "To enlist a creature, that creature must be untapped, it must not be attacking (even if it has vigilance), and it must have haste or have been under that attacking player’s control since the beginning of their current turn."
    );
    let mut t = TestGame::new(2);
    let brute = t.battlefield(P0, "Coalition Warbrute");
    let fresh = t.battlefield(P0, "Grizzly Bears");
    let sick = t.battlefield_sick(P0, "Grizzly Bears");
    let hasty = t.battlefield_sick(P0, "Raging Goblin");
    let tapped = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(tapped);
    // A vigilance attacker isn't tapped, but it's attacking.
    let knight = t.battlefield(P0, "Serra Angel");
    // An opponent's creature isn't yours.
    let theirs = t.battlefield(P1, "Grizzly Bears");
    attack_enlisting(&mut t, &[brute, knight], &[Some(hasty)]);
    let offered = enlist_candidates(&t);
    assert_eq!(offered.len(), 1);
    let offered = &offered[0];
    assert!(offered.contains(&Entity::Object(fresh)));
    assert!(offered.contains(&Entity::Object(hasty)));
    for no in [sick, tapped, knight, theirs, brute] {
        assert!(!offered.contains(&Entity::Object(no)));
    }
    t.resolve_all();
    assert!(t.obj_now(hasty).tapped);
    assert_eq!(t.pt(brute), (4, 4));
}

#[test]
fn a_creature_enlists_the_creature_tapped_for_its_enlist_cost() {
    cr!("702.154c");
    ruling!(
        "Guardian of New Benalia",
        "Guardian of New Benalia's second ability triggers whenever its controller taps another creature for Guardian of New Benalia's enlist ability."
    );
    assert_supported("Guardian of New Benalia");
    // Guardian of New Benalia: enlist; "Whenever this creature enlists a creature, scry 2."
    let mut t = TestGame::new(2);
    let guardian = t.battlefield(P0, "Guardian of New Benalia");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_enlisting(&mut t, &[guardian], &[Some(bears)]);
    assert_eq!(triggers_named(&t, "Enlist").len(), 1);
    assert_eq!(triggers_starting(&t, "Whenever ~ enlists a creature").len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(guardian), (4, 2));
    // Not enlisting: no scry.
    let mut t = TestGame::new(2);
    let guardian = t.battlefield(P0, "Guardian of New Benalia");
    t.battlefield(P0, "Grizzly Bears");
    attack_enlisting(&mut t, &[guardian], &[None]);
    assert!(triggers_starting(&t, "Whenever ~ enlists a creature").is_empty());
    // A creature can't enlist itself: with no other creature, nothing is asked.
    let mut t = TestGame::new(2);
    let guardian = t.battlefield(P0, "Guardian of New Benalia");
    attack_enlisting(&mut t, &[guardian], &[]);
    assert!(enlist_candidates(&t).is_empty());
}

#[test]
fn several_instances_of_enlist_work_independently() {
    cr!("702.154d");
    ruling!(
        "Coalition Warbrute",
        "You may tap only one creature for an enlist ability of an attacking creature, and a single creature can’t be tapped for more than one enlist ability."
    );
    let mut t = TestGame::new(2);
    let brute = t.battlefield(P0, "Coalition Warbrute");
    // A second instance of enlist.
    gain(&mut t, P0, brute, Keyword::new(KeywordKind::Enlist));
    assert_eq!(kw_count(&t, brute, KeywordKind::Enlist), 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    attack_enlisting(&mut t, &[brute], &[Some(bears), Some(giant)]);
    let offered = enlist_candidates(&t);
    assert_eq!(offered.len(), 2);
    // The creature tapped for the first instance isn't offered for the second.
    assert!(!offered[1].contains(&Entity::Object(bears)));
    // Each instance's triggered ability triggers once, for its own cost.
    assert_eq!(triggers_named(&t, "Enlist").len(), 2);
    t.resolve_all();
    assert_eq!(t.pt(brute), (3 + 2 + 3, 4));
    // Paying only one of them: one trigger.
    let mut t = TestGame::new(2);
    let brute = t.battlefield(P0, "Coalition Warbrute");
    gain(&mut t, P0, brute, Keyword::new(KeywordKind::Enlist));
    let giant = t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Grizzly Bears");
    attack_enlisting(&mut t, &[brute], &[None, Some(giant)]);
    assert_eq!(triggers_named(&t, "Enlist").len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(brute), (6, 4));
}

#[test]
fn if_it_enlisted_a_creature_this_combat() {
    cr!("702.154c");
    ruling!(
        "Aradesh, the Founder",
        "Aradesh’s last ability has an intervening if clause. If a creature that attacks hasn’t enlisted a creature this combat, the ability won’t trigger at all for that creature."
    );
    assert_supported("Aradesh, the Founder");
    // Aradesh: "Whenever a creature you control attacks, if it enlisted a creature this
    // combat, the creature that attacked gains double strike until end of turn. If that
    // creature's power is 4 or greater, draw a card."
    for enlist_first in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Aradesh, the Founder");
        let brute = t.battlefield(P0, "Coalition Warbrute");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P0, "Hill Giant");
        // The enlist trigger is put on the stack first (bottom) by default.
        if enlist_first {
            t.answer(P0, DecisionKind::Order, Answer::Indices(vec![1, 0]));
        }
        attack_enlisting(&mut t, &[brute, giant], &[Some(bears)]);
        let order = t
            .asked()
            .into_iter()
            .find_map(|(_, d)| match d {
                Decision::Order { items, .. } => Some(items),
                _ => None,
            })
            .expect("triggers ordered");
        assert_eq!(order.len(), 2, "no trigger for the Hill Giant: {order:?}");
        assert!(order[0].contains("Enlist"));
        t.resolve_all();
        assert!(has_kw(&t, brute, KeywordKind::DoubleStrike));
        assert!(!has_kw(&t, giant, KeywordKind::DoubleStrike));
        // The Warbrute's power is 5 once its enlist ability resolved, 3 before.
        assert_eq!(t.hand_size(P0), usize::from(enlist_first));
    }
}
