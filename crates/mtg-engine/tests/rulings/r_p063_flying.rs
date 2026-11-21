//! Rulings batch P063 — creatures and permanents that gain flying: evasion is checked
//! only as blockers are declared (CR 509.1b, 509.1h, 506.4), conditional P/T and damage
//! that stays marked (CR 120.6, 704.5g), animated permanents and the layers (CR 613.4b,
//! 613.7), intervening-if triggers (CR 603.4), and a few other abilities of these cards.

use crate::r_p062_common::is_tapped;
use crate::r_p063_common::*;
use crate::r_p076_common::unblockable_after_blocked;
use crate::r_p122_common::still_blocks_after;
use crate::r_s01_common::supported;
use crate::r_s02_common::{can_attack, can_play_land, destroy};
use crate::r_s04_common::next_upkeep;
use crate::r_s05_common::{enter, move_to};
use crate::r_s06_common::{damage, give_control};
use crate::r_s29_common::cast_and_resolve;
use mtg_engine::decision::{Action, Answer, Decision, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters::PLUS1;
use mtg_engine::types::CardType;
use mtg_engine::*;

fn flies(t: &mut TestGame, id: ObjectId) -> bool {
    has(t, id, KeywordKind::Flying)
}

#[test]
fn ironclaw_buzzardiers_stays_blocking_after_the_attackers_power_grows() {
    cr!("509.1b", "506.4");
    ruling!(
        "Ironclaw Buzzardiers",
        "\"Can't block\" abilities apply only as blockers are declared. After Ironclaw Buzzardiers blocks, increasing the blocked creature's power won't make it become unblocked."
    );
    supported("Ironclaw Buzzardiers");
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    let buzz = t.battlefield(P1, "Ironclaw Buzzardiers");
    still_blocks_after(&mut t, elves, buzz, |t| {
        cast_and_resolve(t, P0, "Giant Growth", &[obj(elves)]);
    });
}

/// P0's `name` attacks P1 and is blocked by P1's Grizzly Bears; then `act` gives it
/// flying. It stays blocked (CR 509.1h).
fn gains_flying_after_blocked(name: &str, act: impl FnOnce(&mut TestGame, ObjectId)) {
    supported(name);
    let mut t = TestGame::new(2);
    let me = t.battlefield(P0, name);
    let bears = t.battlefield(P1, "Grizzly Bears");
    unblockable_after_blocked(&mut t, me, bears, move |t| {
        act(t, me);
        t.resolve_all();
        assert!(flies(t, me), "{name} gained flying");
    });
}

#[test]
fn gaining_flying_after_being_blocked_doesnt_unblock() {
    cr!("509.1h", "506.4", "702.9b");
    ruling!(
        "Beetleform Mage",
        "Activating Beetleform Mage's ability after it's been blocked by a creature without flying or reach won't cause it to become unblocked."
    );
    ruling!(
        "Dukhara Peafowl",
        "Activating Dukhara Peafowl's ability after it's been blocked by a creature without flying or reach won't cause it to become unblocked."
    );
    ruling!(
        "Goblin Balloon Brigade",
        "For flying to work as an evasion ability, an attacking Goblin Balloon Brigade's ability must be activated before the declare blockers step begins. Once Goblin Balloon Brigade has become blocked, giving it flying won't change that."
    );
    ruling!(
        "Chillerpillar",
        "Chillerpillar gaining flying after it becomes blocked won’t remove the blocking creature from combat or cause Chillerpillar to become unblocked."
    );
    ruling!(
        "Flutterfox",
        "Flutterfox gaining flying after it becomes blocked won't remove the blocking creature from combat or cause Flutterfox to become unblocked."
    );
    ruling!(
        "Moorland Drifter",
        "Flying only matters as blockers are chosen. Causing Moorland Drifter to gain flying after blockers are chosen won't cause it to become unblocked."
    );
    ruling!(
        "Auriok Sunchaser",
        "For the purposes of blocking (either because Auriok Sunchaser is attacking or because a creature with flying is attacking Auriok Sunchaser’s controller), whether Auriok Sunchaser has flying is checked only as the declare blockers step begins."
    );
    ruling!(
        "Snapsail Glider",
        "For the purposes of combat, whether Snapsail Glider has flying is relevant only as the declare blockers step begins."
    );
    for (name, ty, n) in [
        ("Beetleform Mage", None, 0),
        ("Dukhara Peafowl", Some(ManaType::U), 1),
        ("Goblin Balloon Brigade", Some(ManaType::R), 1),
    ] {
        gains_flying_after_blocked(name, move |t, me| {
            if let Some(ty) = ty {
                pool(t, P0, ty, n);
            } else {
                pool(t, P0, ManaType::G, 1);
                pool(t, P0, ManaType::U, 1);
            }
            t.activate(P0, me, 0, &[]).unwrap();
        });
    }
    gains_flying_after_blocked("Chillerpillar", |t, me| {
        t.lands(P0, "Snow-Covered Island", 6);
        t.activate(P0, me, 0, &[]).unwrap();
    });
    gains_flying_after_blocked("Flutterfox", |t, _| {
        t.battlefield(P0, "Rite of Passage");
    });
    gains_flying_after_blocked("Moorland Drifter", |t, _| {
        for n in ["Forest", "Grizzly Bears", "Shock", "Lava Axe"] {
            t.graveyard(P0, n);
        }
    });
    for name in ["Auriok Sunchaser", "Snapsail Glider"] {
        gains_flying_after_blocked(name, |t, _| {
            for _ in 0..3 {
                t.battlefield(P0, "Ornithopter");
            }
        });
    }
}

#[test]
fn a_flying_blocker_losing_flying_keeps_blocking() {
    cr!("509.1b", "506.4", "702.9b");
    ruling!(
        "Auriok Sunchaser",
        "If Auriok Sunchaser blocks a creature with flying, causing it to lose flying (by destroying an artifact its controller controls, for example) won’t change that."
    );
    ruling!(
        "Snapsail Glider",
        "Similarly, if a Snapsail Glider with flying blocks another creature with flying, losing control of enough artifacts to cause the Glider to lose flying won't change that."
    );
    for name in ["Auriok Sunchaser", "Snapsail Glider"] {
        supported(name);
        let mut t = TestGame::new(2);
        let drake = t.battlefield(P0, "Wind Drake");
        let blocker = t.battlefield(P1, name);
        // (Snapsail Glider is an artifact itself.)
        let n = if name == "Snapsail Glider" { 2 } else { 3 };
        let arts: Vec<ObjectId> = (0..n).map(|_| t.battlefield(P1, "Ornithopter")).collect();
        assert!(flies(&mut t, blocker), "{name}");
        still_blocks_after(&mut t, drake, blocker, move |t| destroy(t, arts[0]));
    }
}

#[test]
fn phelddagrifs_three_abilities_target_an_opponent() {
    cr!("115.1", "602.2b");
    ruling!("Phelddagrif", "All three abilities are now targeted.");
    supported("Phelddagrif");
    let mut t = TestGame::new(2);
    let ph = t.battlefield(P0, "Phelddagrif");
    for a in &t.obj_now(ph).chars.abilities {
        if let mtg_engine::ability::AbilityKind::Activated(act) = &a.kind {
            assert_eq!(act.body.targets.len(), 1, "{}", a.text);
        }
    }
    // {G}: trample, and the opponent creates a Hippo.
    pool(&mut t, P0, ManaType::G, 1);
    t.activate(P0, ph, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert!(has(&mut t, ph, KeywordKind::Trample));
    assert_eq!(crate::r_s01_common::tokens(&t, P1).len(), 1);
    // {W}: flying, and the opponent gains 2 life.
    pool(&mut t, P0, ManaType::W, 1);
    t.activate(P0, ph, 1, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert!(flies(&mut t, ph));
    assert_eq!(t.life(P1), 22);
    // {U}: back to hand, and the opponent may draw.
    let hand = t.hand_size(P1);
    pool(&mut t, P0, ManaType::U, 1);
    t.answer_yes(P1, true);
    t.activate(P0, ph, 2, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Phelddagrif"));
    assert_eq!(t.hand_size(P1), hand + 1);
}

#[test]
fn daxos_s_torment_doesnt_enter_as_a_creature() {
    cr!("603.6a", "611.2e");
    ruling!(
        "Daxos's Torment",
        "Although Daxos’s Torment will become a creature after it enters the battlefield, it doesn’t enter as a creature. Abilities that trigger whenever a creature enters the battlefield won’t trigger."
    );
    supported("Daxos's Torment");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soul Warden");
    let dt = enter(&mut t, P0, "Daxos's Torment");
    assert_eq!(t.stack_len(), 1, "only its constellation trigger");
    t.resolve_all();
    assert_eq!(t.life(P0), 20, "Soul Warden didn't trigger");
    assert!(t.obj_now(dt).is(CardType::Creature));
    assert!(flies(&mut t, dt));
    // Control: a creature entering triggers Soul Warden.
    enter(&mut t, P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn ozai_keeps_unspent_mana_as_red_until_it_leaves() {
    cr!("500.4", "106.4", "614.1");
    ruling!(
        "Ozai, the Phoenix King",
        "As long as Ozai, the Phoenix King is on the battlefield, you'll keep unspent mana as steps and phases end (although it will become red). This means you can add mana and spend it during a future step, phase, or turn. Once Ozai leaves the battlefield, you have until the end of the current step or phase to use the mana before it disappears."
    );
    supported("Ozai, the Phoenix King");
    let mut t = TestGame::new(2);
    let ozai = t.battlefield(P0, "Ozai, the Phoenix King");
    pool(&mut t, P0, ManaType::U, 2);
    t.advance_to(P0, Step::BeginningOfCombat);
    let p = &t.g.players[0].mana_pool;
    assert_eq!(p.total(), 2, "kept");
    assert_eq!(p.count(ManaType::R), 2, "as red");
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.g.players[0].mana_pool.total(), 2, "into the next turn");
    move_to(&mut t, ozai, Zone::Graveyard(P0));
    assert_eq!(t.g.players[0].mana_pool.total(), 2, "until this step ends");
    t.advance_to(P1, Step::Draw);
    assert_eq!(t.g.players[0].mana_pool.total(), 0);
}

#[test]
fn multiform_wonder_chooses_the_keyword_as_the_ability_resolves() {
    cr!("602.2", "608.2c");
    ruling!(
        "Multiform Wonder",
        "As you activate one of Multiform Wonder's two activated abilities, you must announce which one you're activating. However, you don't choose that ability's effect until it resolves."
    );
    supported("Multiform Wonder");
    let mut t = TestGame::new(2);
    let mw = t.battlefield(P0, "Multiform Wonder");
    t.g.add_counters(Entity::Player(P0), "energy", 3, None);
    let from = t.asked().len();
    t.activate(P0, mw, 0, &[]).unwrap();
    let options_now = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseOption { .. } | Decision::ChooseModes { .. }))
        .count();
    assert_eq!(options_now, 0, "no choice yet");
    assert_eq!(t.stack_len(), 1);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.resolve_all();
    assert!(flies(&mut t, mw));
    assert!(t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseOption { .. })));
}

/// P0's turn's end step (from the precombat main phase), triggers put on the stack.
fn to_end_step(t: &mut TestGame) {
    t.advance_to(P0, Step::End);
    t.settle();
}

#[test]
fn emergent_haunting_intervening_if_on_trigger_and_resolution() {
    cr!("603.4", "601.2");
    ruling!(
        "Emergent Haunting",
        "At the beginning of your end step, Emergent Haunting’s ability will check to see if you’ve cast a spell from your hand this turn or if Emergent Haunting is a creature. If either is true, the ability won’t trigger at all. If the ability does trigger, it will check again as it tries to resolve. If either is true as the ability tries to resolve, the ability won’t do anything."
    );
    supported("Emergent Haunting");
    // A spell cast from hand this turn: no trigger.
    let mut t = TestGame::new(2);
    let eh = t.battlefield(P0, "Emergent Haunting");
    cast_and_resolve(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    to_end_step(&mut t);
    assert_eq!(t.stack_len(), 0);
    assert!(!t.obj_now(eh).is(CardType::Creature));
    // A spell cast from hand while it waits: no effect.
    let mut t = TestGame::new(2);
    let eh = t.battlefield(P0, "Emergent Haunting");
    to_end_step(&mut t);
    assert_eq!(t.stack_len(), 1);
    crate::r_s25_common::cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(!t.obj_now(eh).is(CardType::Creature));
    // Neither: it becomes a 3/3 flying Spirit.
    let mut t = TestGame::new(2);
    let eh = t.battlefield(P0, "Emergent Haunting");
    to_end_step(&mut t);
    t.resolve_all();
    assert!(t.obj_now(eh).is(CardType::Creature));
    assert_eq!(t.pt(eh), (3, 3));
    assert!(flies(&mut t, eh));
}

#[test]
fn emergent_haunting_stays_a_creature_indefinitely() {
    cr!("611.2a");
    ruling!(
        "Emergent Haunting",
        "Emergent Haunting’s first ability doesn’t have a duration, which means it remains a creature indefinitely."
    );
    let mut t = TestGame::new(2);
    let eh = t.battlefield(P0, "Emergent Haunting");
    to_end_step(&mut t);
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(eh).is(CardType::Creature));
    assert_eq!(t.pt(eh), (3, 3));
}

#[test]
fn inventive_wingsmith_intervening_if_on_trigger_and_resolution() {
    cr!("603.4", "601.2");
    ruling!(
        "Inventive Wingsmith",
        "At the beginning of your end step, Inventive Wingsmith’s ability will check to see if you’ve cast a spell from your hand this turn and if Inventive Wingsmith has a flying counter on it. If either is true, the ability won’t trigger at all. If the ability does trigger, it will check again as it tries to resolve. If either is true as the ability tries to resolve, the ability won’t do anything."
    );
    supported("Inventive Wingsmith");
    // Already has a flying counter: no trigger.
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Inventive Wingsmith");
    put(&mut t, w, "flying", 1);
    to_end_step(&mut t);
    assert_eq!(t.stack_len(), 0);
    // A spell cast from hand this turn: no trigger.
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Inventive Wingsmith");
    cast_and_resolve(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    to_end_step(&mut t);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.counters(w, "flying"), 0);
    // A spell cast from hand while it waits: nothing.
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Inventive Wingsmith");
    to_end_step(&mut t);
    assert_eq!(t.stack_len(), 1);
    crate::r_s25_common::cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.counters(w, "flying"), 0);
    // Neither: a flying counter.
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Inventive Wingsmith");
    to_end_step(&mut t);
    t.resolve_all();
    assert_eq!(t.counters(w, "flying"), 1);
    assert!(flies(&mut t, w));
}

/// `id` is dealt `n` damage (by a Shock-like source of P1's), survives, then `shrink`
/// makes its toughness drop to at most the damage: it dies (CR 704.5g).
fn marked_damage_becomes_lethal(
    t: &mut TestGame,
    id: ObjectId,
    n: i32,
    shrink: impl FnOnce(&mut TestGame),
) {
    let src = t.battlefield(P1, "Grizzly Bears");
    damage(t, src, n, id);
    assert!(t.on_battlefield(id), "it survived the damage");
    shrink(t);
    t.settle();
    assert!(!t.on_battlefield(id), "the marked damage became lethal");
}

#[test]
fn marked_damage_becomes_lethal_when_the_bonus_goes_away() {
    cr!("120.6", "704.5g", "514.2");
    ruling!(
        "Dragonloft Idol",
        "Because damage remains marked on a creature until it's removed as the turn ends, damage dealt to Dragonloft Idol may become lethal if you no longer control a Dragon later in the turn."
    );
    ruling!(
        "Mystic Enforcer",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to Mystic Enforcer may become lethal if the number of cards in your graveyard falls below seven."
    );
    ruling!(
        "Mystic Zealot",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to Mystic Zealot may become lethal if the number of cards in your graveyard falls below seven."
    );
    supported("Dragonloft Idol");
    let mut t = TestGame::new(2);
    let idol = t.battlefield(P0, "Dragonloft Idol");
    let dragon = t.battlefield(P0, "Scion of Ugin");
    assert_eq!(t.pt(idol), (4, 4));
    marked_damage_becomes_lethal(&mut t, idol, 3, move |t| destroy(t, dragon));
    for (name, n) in [("Mystic Enforcer", 4), ("Mystic Zealot", 4)] {
        supported(name);
        let mut t = TestGame::new(2);
        let m = t.battlefield(P0, name);
        let gy: Vec<ObjectId> = (0..7).map(|_| t.graveyard(P0, "Grizzly Bears")).collect();
        t.g.recompute();
        assert!(flies(&mut t, m), "{name}: threshold");
        marked_damage_becomes_lethal(&mut t, m, n, move |t| {
            move_to(t, gy[0], Zone::Exile);
        });
    }
}

#[test]
fn blinkmoth_nexus_pumps_itself_while_blocking_and_stays_in_combat() {
    cr!("506.4", "510.1", "701.26a");
    ruling!(
        "Blinkmoth Nexus",
        "Blinkmoth Nexus can be the target of its last ability after activating its middle ability. Activating it after it has blocked a creature won't remove it from combat or stop it from dealing and being dealt combat damage."
    );
    supported("Blinkmoth Nexus");
    let mut t = TestGame::new(2);
    let nexus = t.battlefield(P0, "Blinkmoth Nexus");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // P1's turn: P0 animates the Nexus, which blocks the attacking Bears.
    t.set_step(P1, Step::BeginningOfCombat);
    pool(&mut t, P0, ManaType::C, 1);
    t.activate(P0, nexus, 1, &[]).unwrap();
    t.resolve_all();
    assert!(flies(&mut t, nexus));
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P0))]),
    );
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(nexus, bears)]),
    );
    t.advance_to(P1, Step::DeclareBlockers);
    // {1}, {T}: target Blinkmoth gets +1/+1 — itself.
    pool(&mut t, P0, ManaType::C, 1);
    t.activate(P0, nexus, 2, &[obj(nexus)]).unwrap();
    t.resolve_all();
    assert!(is_tapped(&t, nexus));
    assert_eq!(t.pt(nexus), (2, 2));
    t.advance_to(P1, Step::EndOfCombat);
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "it dealt its 2 damage");
    assert!(t.in_graveyard(P0, "Blinkmoth Nexus"), "and was dealt lethal damage");
    assert_eq!(t.life(P0), 20);
}

/// P0's Bogardan Dragonheart becomes a Dragon (sacrificing Grizzly Bears).
fn dragonheart(t: &mut TestGame, dh: ObjectId) {
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, dh, 0, &[]).unwrap();
    t.resolve_all();
}

#[test]
fn bogardan_dragonheart_keeps_its_abilities_and_layers_its_base_pt() {
    cr!("613.4b", "613.7", "613.4c", "613.4d");
    ruling!(
        "Bogardan Dragonheart",
        "Bogardan Dragonheart doesn’t lose any abilities when it becomes a Dragon."
    );
    ruling!(
        "Bogardan Dragonheart",
        "Bogardan Dragonheart’s ability overwrites all previous effects that set its creature types, power, and/or toughness to specific values. Other effects that set these characteristics to specific values that start to apply after the ability resolves will overwrite that part of this effect."
    );
    ruling!(
        "Bogardan Dragonheart",
        "Effects that modify Bogardan Dragonheart’s power or toughness (such as the effects of Force of Virtue or Giant Growth) will apply no matter when they started to take effect. The same is true for counters that change its power or toughness (such as +1/+1 counters)."
    );
    supported("Bogardan Dragonheart");
    let mut t = TestGame::new(2);
    let dh = t.battlefield(P0, "Bogardan Dragonheart");
    // Earlier: Diminish (base 1/1), Giant Growth (+3/+3), a +1/+1 counter.
    cast_and_resolve(&mut t, P0, "Diminish", &[obj(dh)]);
    cast_and_resolve(&mut t, P0, "Giant Growth", &[obj(dh)]);
    put(&mut t, dh, PLUS1, 1);
    assert_eq!(t.pt(dh), (5, 5));
    dragonheart(&mut t, dh);
    assert_eq!(t.pt(dh), (8, 8), "base 4/4 overwrites Diminish; the rest still applies");
    assert!(crate::r_p064_common::is_a(&mut t, dh, "Dragon"));
    assert!(flies(&mut t, dh) && has(&mut t, dh, KeywordKind::Haste));
    // Its ability is still there: it can be activated again.
    let a = t.obj_now(dh).chars.abilities.iter().any(|a| {
        matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_))
    });
    assert!(a, "it kept its ability");
    // A later Diminish overwrites the base 4/4.
    cast_and_resolve(&mut t, P0, "Diminish", &[obj(dh)]);
    assert_eq!(t.pt(dh), (5, 5));
    assert!(crate::r_p064_common::is_a(&mut t, dh, "Dragon"), "still a Dragon");
}

#[test]
fn thunderous_orator_checks_your_creatures_as_its_trigger_resolves() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Thunderous Orator",
        "Check the creatures you control as the triggered ability resolves to determine which abilities Thunderous Orator gains. Once Thunderous Orator gains an ability, it doesn’t matter what happens to other creatures you control with that ability."
    );
    supported("Thunderous Orator");
    // The flyer leaves before the trigger resolves: no flying.
    let mut t = TestGame::new(2);
    let orator = t.battlefield(P0, "Thunderous Orator");
    let drake = t.battlefield(P0, "Wind Drake");
    crate::r_s01_common::attack_with(&mut t, &[(orator, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, drake);
    t.resolve_all();
    assert!(!flies(&mut t, orator));
    // A flyer arrives before it resolves: flying, kept after the flyer leaves.
    let mut t = TestGame::new(2);
    let orator = t.battlefield(P0, "Thunderous Orator");
    crate::r_s01_common::attack_with(&mut t, &[(orator, Entity::Player(P1))]);
    let drake = t.battlefield(P0, "Wind Drake");
    t.resolve_all();
    assert!(flies(&mut t, orator));
    destroy(&mut t, drake);
    assert!(flies(&mut t, orator));
}

#[test]
fn crovax_can_sacrifice_itself_to_its_upkeep_trigger() {
    cr!("603.3", "701.21a");
    ruling!(
        "Crovax the Cursed",
        "Crovax can sacrifice itself to its upkeep triggered ability."
    );
    supported("Crovax the Cursed");
    let mut t = TestGame::new(2);
    let crovax = t.battlefield(P0, "Crovax the Cursed");
    put(&mut t, crovax, PLUS1, 4);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(crovax)]);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Crovax the Cursed"));
}

#[test]
fn essence_channeler_one_trigger_per_life_gain_event() {
    cr!("119.10", "510.2", "702.15b");
    ruling!(
        "Essence Channeler",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event."
    );
    ruling!(
        "Essence Channeler",
        "Essence Channeler’s second ability triggers just once for each life-gaining event, no matter how much life was gained."
    );
    supported("Essence Channeler");
    // Two lifelinkers deal combat damage at once: two triggers.
    let mut t = TestGame::new(2);
    let ec = t.battlefield(P0, "Essence Channeler");
    let a = t.battlefield(P0, "Vampire Nighthawk");
    let b = t.battlefield(P0, "Vampire Nighthawk");
    to_first_strike_damage(&mut t, &[a, b], &[]);
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.counters(ec, PLUS1), 2);
    // One lifelinker damaging a player and a blocker... (two blockers): one trigger.
    let mut t = TestGame::new(2);
    let ec = t.battlefield(P0, "Essence Channeler");
    let n = t.battlefield(P0, "Vampire Nighthawk");
    put(&mut t, n, PLUS1, 2);
    let x = t.battlefield(P1, "Llanowar Elves");
    let y = t.battlefield(P1, "Llanowar Elves");
    t.answer(
        P0,
        DecisionKind::Damage,
        Answer::Numbers(vec![2, 2]),
    );
    to_first_strike_damage(&mut t, &[n], &[(x, n), (y, n)]);
    assert!(t.life(P0) > 20);
    assert_eq!(t.counters(ec, PLUS1), 1);
}

#[test]
fn essence_channeler_cares_about_losing_life_not_the_net_change() {
    cr!("119.3");
    ruling!(
        "Essence Channeler",
        "Essence Channeler’s first ability cares whether you’ve lost life this turn, not how your life total has changed."
    );
    let mut t = TestGame::new(2);
    let ec = t.battlefield(P0, "Essence Channeler");
    t.g.gain_life(P0, 2);
    t.g.flush_events();
    t.resolve_all();
    assert!(!flies(&mut t, ec));
    t.g.lose_life(P0, 1);
    t.g.flush_events();
    t.settle();
    assert!(t.life(P0) > 20);
    assert!(flies(&mut t, ec));
    assert!(has(&mut t, ec, KeywordKind::Vigilance));
}

#[test]
fn essence_channeler_puts_the_same_counters_it_had_of_each_kind() {
    cr!("603.10a", "122.1");
    ruling!(
        "Essence Channeler",
        "Essence Channeler’s last ability doesn’t cause you to move counters from Essence Channeler onto the target creature. Rather, you put the same number of each kind of counter Essence Channeler had when it died onto the target creature."
    );
    ruling!(
        "Essence Channeler",
        "Essence Channeler’s last ability puts all counters that were on Essence Channeler onto the target creature, not just its +1/+1 counters."
    );
    let mut t = TestGame::new(2);
    let ec = t.battlefield(P0, "Essence Channeler");
    let bears = t.battlefield(P0, "Grizzly Bears");
    put(&mut t, ec, PLUS1, 2);
    put(&mut t, ec, "oil", 3);
    t.answer_targets(P0, &[obj(bears)]);
    destroy(&mut t, ec);
    t.resolve_all();
    assert_eq!(t.counters(bears, PLUS1), 2);
    assert_eq!(t.counters(bears, "oil"), 3);
    // (The dead Channeler's last known counters are unchanged.)
    assert_eq!(t.g.obj(ec).counter(PLUS1), 2);
}

/// P0's Angel's Tomb and a creature entering under P0's control (the Tomb animated).
fn animate_tomb(t: &mut TestGame, tomb: ObjectId) {
    t.answer_yes(P0, true);
    enter(t, P0, "Grizzly Bears");
    t.resolve_all();
    assert!(t.obj_now(tomb).is(CardType::Creature));
}

#[test]
fn angels_tomb_base_pt_and_later_effects() {
    cr!("613.4b", "613.4c", "613.7");
    ruling!(
        "Angel's Tomb",
        "Effects setting Angel’s Tomb’s base power and toughness to specific values that begin to apply after Angel’s Tomb has become a creature will override the effect of Angel’s Tomb."
    );
    ruling!(
        "Angel's Tomb",
        "If Angel’s Tomb is already a creature when a creature enters the battlefield under your control, its ability will override any effects that set its base power and toughness to specific values, but other changes to its power and toughness (such as the one created by Titanic Growth) will still apply."
    );
    supported("Angel's Tomb");
    let mut t = TestGame::new(2);
    let tomb = t.battlefield(P0, "Angel's Tomb");
    animate_tomb(&mut t, tomb);
    assert_eq!(t.pt(tomb), (3, 3));
    // Diminish (base 1/1) applies after: 1/1.
    cast_and_resolve(&mut t, P0, "Diminish", &[obj(tomb)]);
    assert_eq!(t.pt(tomb), (1, 1));
    // Titanic Growth: 5/5.
    cast_and_resolve(&mut t, P0, "Titanic Growth", &[obj(tomb)]);
    assert_eq!(t.pt(tomb), (5, 5));
    // The Tomb's ability again: base 3/3 again, Titanic Growth still applies.
    animate_tomb(&mut t, tomb);
    assert_eq!(t.pt(tomb), (7, 7));
}

#[test]
fn goblin_ski_patrol_once_per_game_and_its_new_controller_sacrifices_it() {
    cr!("602.5b", "603.7c");
    ruling!(
        "Goblin Ski Patrol",
        "Goblin Ski Patrol's ability may be activated only once in the entire game. If its ability is activated and then it changes controllers, its ability may not be activated again, and its new controller will still have to sacrifice it at the beginning of the End step."
    );
    supported("Goblin Ski Patrol");
    let mut t = TestGame::new(2);
    let gsp = t.battlefield(P0, "Goblin Ski Patrol");
    t.battlefield(P0, "Snow-Covered Mountain");
    t.battlefield(P1, "Snow-Covered Mountain");
    pool(&mut t, P0, ManaType::R, 2);
    t.activate(P0, gsp, 0, &[]).unwrap();
    t.resolve_all();
    assert!(flies(&mut t, gsp));
    give_control(&mut t, gsp, P1);
    // P1 can't activate it again.
    pool(&mut t, P1, ManaType::R, 2);
    assert!(t.activate(P1, gsp, 0, &[]).is_err());
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Goblin Ski Patrol"), "sacrificed (its owner's graveyard)");
    // Control (fresh game): once is once, even for the same controller next turn.
    let mut t = TestGame::new(2);
    let gsp = t.battlefield(P0, "Goblin Ski Patrol");
    t.battlefield(P0, "Snow-Covered Mountain");
    pool(&mut t, P0, ManaType::R, 2);
    t.activate(P0, gsp, 0, &[]).unwrap();
    pool(&mut t, P0, ManaType::R, 2);
    assert!(t.activate(P0, gsp, 0, &[]).is_err());
}

#[test]
fn goddrics_ability_affects_only_the_dragons_there_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Goddric, Cloaked Reveler",
        "Goddric's activated ability affects only Dragons you control at the time it resolves. Any Dragons that come under your control later in the turn won't be affected."
    );
    supported("Goddric, Cloaked Reveler");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Goddric, Cloaked Reveler");
    // Celebration: two nonland permanents entered this turn.
    enter(&mut t, P0, "Ornithopter");
    enter(&mut t, P0, "Ornithopter");
    assert!(crate::r_p064_common::is_a(&mut t, g, "Dragon"));
    assert_eq!(t.pt(g), (4, 4));
    pool(&mut t, P0, ManaType::R, 1);
    crate::r_s06_common::activate_containing(&mut t, P0, g, "Dragons you control")
        .unwrap();
    t.resolve_all();
    assert_eq!(t.pt(g), (5, 4));
    let scion = enter(&mut t, P0, "Scion of Ugin");
    assert_eq!(t.pt(scion), (4, 4), "the later Dragon isn't affected");
}

#[test]
fn golem_artisan_can_target_itself() {
    cr!("115.1", "602.2b");
    ruling!("Golem Artisan", "Golem Artisan can be the target of its own abilities.");
    supported("Golem Artisan");
    let mut t = TestGame::new(2);
    let ga = t.battlefield(P0, "Golem Artisan");
    pool(&mut t, P0, ManaType::C, 2);
    t.activate(P0, ga, 0, &[obj(ga)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(ga), (4, 4));
    pool(&mut t, P0, ManaType::C, 2);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.activate(P0, ga, 1, &[obj(ga)]).unwrap();
    t.resolve_all();
    assert!(flies(&mut t, ga));
}

#[test]
fn hearthhull_additional_land_play_is_still_only_on_your_turn() {
    cr!("305.2", "305.1");
    ruling!(
        "Hearthhull, the Worldseed",
        "Hearthhull’s second ability doesn’t allow you to play lands on other players’ turns."
    );
    supported("Hearthhull, the Worldseed");
    let mut t = TestGame::new(2);
    let hh = t.battlefield(P0, "Hearthhull, the Worldseed");
    put(&mut t, hh, "charge", 2);
    let land = t.battlefield(P0, "Forest");
    let card = t.hand(P0, "Mountain");
    t.set_step(P1, Step::PrecombatMain);
    pool(&mut t, P0, ManaType::C, 1);
    t.answer_choose(P0, &[obj(land)]);
    crate::r_s06_common::activate_containing(&mut t, P0, hh, "Draw two cards").unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Forest"));
    assert!(!can_play_land(&mut t, P0, card), "not on P1's turn");
    // Control: on P0's own turn it adds a land play.
    let mut t = TestGame::new(2);
    let hh = t.battlefield(P0, "Hearthhull, the Worldseed");
    put(&mut t, hh, "charge", 2);
    let land = t.battlefield(P0, "Forest");
    let first = t.hand(P0, "Mountain");
    let second = t.hand(P0, "Mountain");
    t.play_land(P0, first).unwrap();
    pool(&mut t, P0, ManaType::C, 1);
    t.answer_choose(P0, &[obj(land)]);
    crate::r_s06_common::activate_containing(&mut t, P0, hh, "Draw two cards").unwrap();
    t.resolve_all();
    assert!(can_play_land(&mut t, P0, second));
}

#[test]
fn hero_of_bretagard_counts_any_exile_from_hand_and_permanents_your_effects_exile() {
    cr!("406.3", "702.143a", "603.2");
    ruling!(
        "Hero of Bretagard",
        "Cards put into exile from your hand for any reason, such as the foretell ability, cause the ability to trigger"
    );
    ruling!(
        "Hero of Bretagard",
        "Hero of Bretagard's triggered ability will trigger when a spell or ability you control exiles any permanent, including token permanents and permanents you don't control."
    );
    supported("Hero of Bretagard");
    // Foretelling a card (a special action) exiles it from P0's hand.
    let mut t = TestGame::new(2);
    let hero = t.battlefield(P0, "Hero of Bretagard");
    let card = t.hand(P0, "Saw It Coming");
    pool(&mut t, P0, ManaType::C, 2);
    t.g.turn.priority = Some(P0);
    t.g.take_action(P0, Action::Special(SpecialAction::Foretell { card }));
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.counters(hero, PLUS1), 1);
    // Swords to Plowshares exiles P1's token.
    let token = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    cast_and_resolve(&mut t, P0, "Swords to Plowshares", &[obj(token)]);
    assert!(!t.on_battlefield(token));
    assert_eq!(t.counters(hero, PLUS1), 2);
}

#[test]
fn answered_prayers_animated_the_turn_it_arrived_cant_attack() {
    cr!("302.6", "611.2e");
    ruling!(
        "Answered Prayers",
        "If Answered Prayers becomes a creature but you haven’t controlled it continuously since your most recent turn began, you won’t be able to attack with it."
    );
    supported("Answered Prayers");
    for (since_turn_began, attacks) in [(false, false), (true, true)] {
        let mut t = TestGame::new(2);
        let ap = if since_turn_began {
            t.battlefield(P0, "Answered Prayers")
        } else {
            enter(&mut t, P0, "Answered Prayers")
        };
        enter(&mut t, P0, "Grizzly Bears");
        t.resolve_all();
        assert!(t.obj_now(ap).is(CardType::Creature));
        t.set_step(P0, Step::BeginningOfCombat);
        assert_eq!(can_attack(&mut t, ap), attacks);
    }
}
