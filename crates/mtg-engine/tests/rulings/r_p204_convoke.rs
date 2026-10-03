//! Rulings batch P204 — convoke (CR 702.51): "Your creatures can help cast this spell.
//! Each creature you tap while casting this spell pays for {1} or one mana of that
//! creature's color." Also More Than Meets the Eye / convert (Goldbug).

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s05_common::run_from;
use mtg_engine::ability::{Destination, Effect, Sel};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

fn objs(ids: &[ObjectId]) -> Vec<Entity> {
    ids.iter().map(|o| Entity::Object(*o)).collect()
}

/// Queues `p`'s choice of creatures to convoke the next spell.
fn convoke_with(t: &mut TestGame, p: PlayerId, ids: &[ObjectId]) {
    t.answer(p, DecisionKind::Entities, Answer::Entities(objs(ids)));
}

/// The (candidates, max) of the last convoke choice asked since decision `from`.
fn convoke_offer(t: &TestGame, from: usize) -> (usize, u32) {
    t.asked()[from..]
        .iter()
        .rev()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities {
                prompt,
                candidates,
                max,
                ..
            } if prompt.contains("convoke") => Some((candidates.len(), *max)),
            _ => None,
        })
        .expect("convoke offered")
}

fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

fn n_creatures(t: &mut TestGame, p: PlayerId, name: &str, n: usize) -> Vec<ObjectId> {
    (0..n).map(|_| t.battlefield(p, name)).collect()
}

#[test]
fn a_creature_that_would_pay_nothing_cant_be_tapped_for_convoke() {
    cr!("702.51a", "702.51c", "601.2f", "601.2h");
    ruling!(
        "Calamity of Cinders",
        "If tapping a creature using convoke wouldn't pay for any mana in that spell's cost"
    );
    supported("Calamity of Cinders");
    supported("Thalia, Guardian of Thraben");
    // Calamity of Cinders {5}{R}{R}: "Convoke. Calamity of Cinders deals 6 damage to each
    // untapped creature." Seven green Bears: five pay {5}; the other two can't pay {R}{R}
    // and stay untapped; two Mountains pay the rest.
    let mut t = TestGame::new(2);
    let bears = n_creatures(&mut t, P0, "Grizzly Bears", 7);
    t.lands(P0, "Mountain", 2);
    let card = t.hand(P0, "Calamity of Cinders");
    convoke_with(&mut t, P0, &bears);
    let spell = t.cast(P0, card).go();
    assert_eq!(bears.iter().filter(|b| tapped(&t, **b)).count(), 5);
    let info = t.obj(spell).stack.as_ref().unwrap().cast.clone();
    assert_eq!(info.convoked.len(), 5);
    t.resolve_all();
    // The two untapped Bears are dealt 6 damage; the five that convoked survive.
    assert_eq!(t.g.permanents().filter(|o| o.chars.name == "Grizzly Bears").count(), 5);

    // Thalia ("Noncreature spells cost {1} more to cast") increases the total cost before
    // it's paid: eight creatures can convoke it (two red Raging Goblins pay {R}{R}).
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    let mut c = n_creatures(&mut t, P0, "Grizzly Bears", 6);
    c.extend(n_creatures(&mut t, P0, "Raging Goblin", 2));
    t.battlefield(P0, "Grizzly Bears");
    let card = t.hand(P0, "Calamity of Cinders");
    let from = t.asked().len();
    convoke_with(&mut t, P0, &c);
    t.cast(P0, card).go();
    assert_eq!(convoke_offer(&t, from), (9, 8));
    assert!(c.iter().all(|x| tapped(&t, *x)));
}

#[test]
fn a_multicolored_creature_pays_generic_or_one_of_its_colors() {
    cr!("702.51a", "105.2");
    ruling!(
        "Vote Out",
        "Tapping a multicolored creature using convoke will pay for {1} or one mana of your choice of any of that creature’s colors."
    );
    supported("Vote Out");
    supported("Tithe Drinker");
    // Vote Out {3}{B}. Tithe Drinker is white and black.
    // With three white creatures, the Drinker pays {B}.
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Hill Giant");
    let mut c = n_creatures(&mut t, P0, "Savannah Lions", 3);
    c.push(t.battlefield(P0, "Tithe Drinker"));
    let card = t.hand(P0, "Vote Out");
    convoke_with(&mut t, P0, &c);
    t.cast(P0, card).target(target).go();
    assert!(c.iter().all(|x| tapped(&t, *x)));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // With a Swamp paying {B}, the Drinker pays {1}.
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Hill Giant");
    let mut c = n_creatures(&mut t, P0, "Savannah Lions", 2);
    c.push(t.battlefield(P0, "Tithe Drinker"));
    t.lands(P0, "Swamp", 1);
    let card = t.hand(P0, "Vote Out");
    convoke_with(&mut t, P0, &c);
    t.cast(P0, card).target(target).go();
    assert!(c.iter().all(|x| tapped(&t, *x)));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn no_more_creatures_can_convoke_than_the_total_cost_needs() {
    cr!("702.51a", "601.2f");
    ruling!(
        "The Wandering Rescuer",
        "You can't tap more creatures to cast this spell using convoke than are required to pay its total cost."
    );
    ruling!(
        "Zephyr Singer",
        "You can't tap more creatures to convoke Zephyr Singer than is necessary to pay for the spell. In most cases, this means four creatures."
    );
    ruling!(
        "Knight-Errant of Eos",
        "You can't tap more creatures to convoke Knight-Errant of Eos than is necessary to pay for the spell. In most cases, this means five creatures."
    );
    supported("Sphere of Resistance");
    // (name, total mana value, cost increase from Sphere of Resistance)
    for (name, mv) in [
        ("The Wandering Rescuer", 5u32),
        ("Zephyr Singer", 4),
        ("Knight-Errant of Eos", 5),
    ] {
        supported(name);
        for sphere in [false, true] {
            let mut t = TestGame::new(2);
            if sphere {
                // "Spells cost {1} more to cast": the additional mana can be convoked too.
                t.battlefield(P1, "Sphere of Resistance");
            }
            let total = mv + sphere as u32;
            let c = n_creatures(&mut t, P0, "Savannah Lions", 4);
            let mut c2 = n_creatures(&mut t, P0, "Merfolk of the Pearl Trident", 4);
            let mut all = c.clone();
            all.append(&mut c2);
            let card = t.hand(P0, name);
            let from = t.asked().len();
            // Asking to tap more than needed is not a legal choice.
            convoke_with(&mut t, P0, &all[..(total as usize + 1)]);
            let _ = t.cast(P0, card).try_go();
            assert_eq!(convoke_offer(&t, from), (8, total), "{name} {sphere}");
            assert!(
                all.iter().filter(|x| tapped(&t, **x)).count() <= total as usize,
                "{name}"
            );
        }
    }
    // Choosing exactly the total: The Wandering Rescuer {3}{W}{W} cast by five creatures
    // alone.
    let mut t = TestGame::new(2);
    let c = n_creatures(&mut t, P0, "Savannah Lions", 5);
    let card = t.hand(P0, "The Wandering Rescuer");
    convoke_with(&mut t, P0, &c);
    t.cast(P0, card).go();
    assert!(c.iter().all(|x| tapped(&t, *x)));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("The Wandering Rescuer").len(), 1);
}

#[test]
fn convoke_given_to_a_spell_that_has_it_is_redundant() {
    cr!("702.51a", "702.51d");
    ruling!(
        "Flockchaser Phantom",
        "If the next spell you cast after Flockchaser Phantom's ability resolves already has convoke, giving it convoke again doesn't have any real benefit."
    );
    supported("Flockchaser Phantom");
    // Flockchaser Phantom attacks: the next spell (Vote Out, {3}{B}, which has convoke)
    // has convoke again. Each creature still pays for only one mana: five creatures, one
    // too many, isn't a legal choice; four pay the whole cost.
    let mut t = TestGame::new(2);
    let phantom = t.battlefield(P0, "Flockchaser Phantom");
    let target = t.battlefield(P1, "Hill Giant");
    let c = n_creatures(&mut t, P0, "Grizzly Bears", 3);
    let black = t.battlefield(P0, "Vampire Interloper");
    crate::r_s01_common::attack_with(&mut t, &[(phantom, Entity::Player(P1))]);
    t.resolve_all();
    t.advance_to(P0, mtg_engine::turn::Step::PostcombatMain);
    let mut all = c.clone();
    all.push(black);
    let card = t.hand(P0, "Vote Out");
    let from = t.asked().len();
    convoke_with(&mut t, P0, &all);
    t.cast(P0, card).target(target).go();
    assert_eq!(convoke_offer(&t, from).1, 4);
    // Asked once only, though the spell has convoke twice.
    assert_eq!(
        t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.contains("convoke")))
            .count(),
        1
    );
    assert!(all.iter().all(|x| tapped(&t, *x)));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn creatures_that_convoked_count_even_if_they_left_the_battlefield() {
    cr!("702.51c", "603.10");
    ruling!(
        "Knight-Errant of Eos",
        "It doesn't matter if the creatures that convoked Knight-Errant of Eos are still on the battlefield"
    );
    // Knight-Errant of Eos {4}{W}: three Bears convoke it; they're destroyed before its
    // enters trigger resolves; X is still 3.
    let mut t = TestGame::new(2);
    let c = n_creatures(&mut t, P0, "Grizzly Bears", 3);
    t.lands(P0, "Plains", 2);
    let giant = t.library_top(P0, "Hill Giant");
    let courser = t.library_top(P0, "Centaur Courser");
    let bears = t.library_top(P0, "Grizzly Bears");
    let card = t.hand(P0, "Knight-Errant of Eos");
    convoke_with(&mut t, P0, &c);
    t.cast(P0, card).go();
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 1, "the enters trigger");
    for b in &c {
        destroy(&mut t, *b);
    }
    assert!(c.iter().all(|b| !t.on_battlefield(*b)));
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Object(courser), Entity::Object(bears)]);
    t.resolve_all();
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .expect("a choice of creature cards");
    assert!(offered.contains(&Entity::Object(courser)));
    assert!(!offered.contains(&Entity::Object(giant)));
    assert!(t.in_hand(P0, "Centaur Courser"));
    assert_eq!(t.zone(giant), mtg_engine::object::Zone::Library(P0));
}

#[test]
fn feaster_of_fools_can_devour_its_convokers_but_not_creatures_entering_with_it() {
    cr!("702.82a", "702.51c");
    ruling!(
        "Feaster of Fools",
        "It can devour creatures that convoked it"
    );
    supported("Feaster of Fools");
    // Feaster of Fools {4}{B}{B}, devour 2: six creatures convoke it (two black ones pay
    // {B}{B}), then it devours two of them.
    let mut t = TestGame::new(2);
    let mut c = n_creatures(&mut t, P0, "Grizzly Bears", 4);
    c.extend(n_creatures(&mut t, P0, "Vampire Interloper", 2));
    let card = t.hand(P0, "Feaster of Fools");
    convoke_with(&mut t, P0, &c);
    t.cast(P0, card).go();
    assert!(c.iter().all(|x| tapped(&t, *x)));
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Object(c[0]), Entity::Object(c[1])]);
    t.resolve_all();
    let offered = devour_offer(&t, from);
    assert!(c.iter().all(|x| offered.contains(&Entity::Object(*x))));
    let feaster = t.named_on_battlefield("Feaster of Fools")[0];
    assert_eq!(t.counters(feaster, counters::PLUS1), 4);
    assert!(!t.on_battlefield(c[0]) && !t.on_battlefield(c[1]));

    // Entering at the same time as another creature, it can't devour that creature.
    let mut t = TestGame::new(2);
    let old = t.battlefield(P0, "Grizzly Bears");
    let feaster = t.hand(P0, "Feaster of Fools");
    let other = t.hand(P0, "Hill Giant");
    let from = t.asked().len();
    run_from(
        &mut t,
        P0,
        None,
        Effect::Move {
            what: Sel::AllTargets,
            to: Destination::battlefield(),
        },
        &objs(&[feaster, other]),
    );
    let offered = devour_offer(&t, from);
    assert!(offered.contains(&Entity::Object(old)));
    assert!(!offered.iter().any(|e| e.object().is_some_and(|o| t.g.obj(o).chars.name == "Hill Giant")));
}

fn devour_offer(t: &TestGame, from: usize) -> Vec<Entity> {
    t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities {
                prompt, candidates, ..
            } if prompt.contains("devour") => Some(candidates.clone()),
            _ => None,
        })
        .expect("devour offered")
}

#[test]
fn goldbug_converting_before_combat_damage_stops_preventing_it() {
    cr!("701.28a", "615.1", "510.2");
    ruling!(
        "Goldbug, Humanity's Ally // Goldbug, Scrappy Scout",
        "The same is true if it converts before combat damage, so be careful with that second spell!"
    );
    const GOLDBUG: &str = "Goldbug, Humanity's Ally // Goldbug, Scrappy Scout";
    supported(GOLDBUG);
    // Goldbug, Humanity's Ally: "Prevent all combat damage that would be dealt to
    // attacking Humans you control. Whenever you cast your second spell each turn, convert
    // Goldbug." A Human (Elite Vanguard) attacks and
    // is blocked by a Hill Giant.
    for second_spell in [false, true] {
        let mut t = TestGame::new(2);
        let goldbug = t.battlefield(P0, GOLDBUG);
        let human = t.battlefield(P0, "Elite Vanguard");
        let giant = t.battlefield(P1, "Hill Giant");
        crate::r_s01_common::attack_with(&mut t, &[(human, Entity::Player(P1))]);
        t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(giant, human)]));
        t.advance_to(P0, mtg_engine::turn::Step::DeclareBlockers);
        t.settle();
        if second_spell {
            t.lands(P0, "Mountain", 2);
            for _ in 0..2 {
                let o = t.hand(P0, "Shock");
                t.g.turn.priority = Some(P0);
                t.cast(P0, o).target(P1).go();
                t.resolve_all();
            }
            assert_eq!(t.obj_now(goldbug).chars.name, "Goldbug, Scrappy Scout");
        }
        t.advance_to(P0, mtg_engine::turn::Step::EndOfCombat);
        assert_eq!(t.on_battlefield(human), !second_spell, "{second_spell}");
    }
}
