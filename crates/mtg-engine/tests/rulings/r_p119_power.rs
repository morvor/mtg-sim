//! Rulings batch P119 — abilities that look at their source's own power: last known
//! information when it has left the battlefield (CR 608.2h, 603.10a), values fixed as the
//! ability resolves (CR 608.2h), damage divided as the ability is put on the stack
//! (CR 601.2d, 603.3d), blocking restrictions checked only as blockers are declared
//! (CR 509.1b), and creatures put onto the battlefield attacking (CR 508.4).

use crate::r_p119_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// --- last known power -----------------------------------------------------------------

#[test]
fn raffines_silencer_uses_its_last_known_power() {
    cr!("608.2h", "603.10a");
    ruling!(
        "Raffine's Silencer",
        "Use Raffine's Silencer's power as it last existed on the battlefield to determine the value of X."
    );
    supported("Raffine's Silencer");
    let mut t = TestGame::new(2);
    let silencer = with_counters(&mut t, P0, "Raffine's Silencer", 2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    destroy(&mut t, silencer);
    t.resolve_all();
    assert!(
        t.in_graveyard(P1, "Hill Giant"),
        "-3/-3 from a 3/3 Silencer"
    );
}

#[test]
fn stalactite_stalker_uses_its_last_known_power() {
    cr!("608.2h", "602.2");
    ruling!(
        "Stalactite Stalker",
        "Use Stalactite Stalker's power as it last existed on the battlefield to determine the value of X."
    );
    supported("Stalactite Stalker");
    let mut t = TestGame::new(2);
    let stalker = with_counters(&mut t, P0, "Stalactite Stalker", 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    t.activate(P0, stalker, 0, &[obj(bears)]).unwrap();
    assert!(!t.on_battlefield(stalker), "sacrificed as a cost");
    t.resolve_all();
    assert!(
        t.in_graveyard(P1, "Grizzly Bears"),
        "-2/-2 from a 2/2 Stalker"
    );
}

/// `name` with `n` +1/+1 counters dies; returns the game afterwards (stack resolved).
fn dies_with_counters(name: &str, n: u32) -> TestGame {
    let mut t = TestGame::new(2);
    let c = with_counters(&mut t, P0, name, n);
    destroy(&mut t, c);
    t.resolve_all();
    t
}

#[test]
fn termagant_swarm_uses_its_last_known_power() {
    cr!("608.2h", "603.10a");
    ruling!(
        "Termagant Swarm",
        "Use Termagent Swarm's power as it last existed on the battlefield to determine the number of tokens you create."
    );
    supported("Termagant Swarm");
    let t = dies_with_counters("Termagant Swarm", 3);
    assert_eq!(tokens_named(&t, P0, "Tyranid"), 3);
}

#[test]
fn willow_geist_uses_its_last_known_power() {
    cr!("608.2h", "603.10a");
    ruling!(
        "Willow Geist",
        "Use Willow Geist's power as it last existed on the battlefield to determine how much life you will gain as its last ability resolves."
    );
    supported("Willow Geist");
    let t = dies_with_counters("Willow Geist", 2);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn nested_shambler_uses_its_last_known_power() {
    cr!("608.2h", "603.10a", "107.1b");
    ruling!(
        "Nested Shambler",
        "X is Nested Shambler's power the last time it was on the battlefield, not its power in the graveyard. If its power was 0 or less when it died, you won't create any tokens."
    );
    supported("Nested Shambler");
    let t = dies_with_counters("Nested Shambler", 2);
    assert_eq!(tokens_named(&t, P0, "Squirrel"), 3);
    // With a -1/-1 counter it dies as a 0/0: no tokens.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Nested Shambler");
    give_minus1(&mut t, s, 1);
    assert!(t.in_graveyard(P0, "Nested Shambler"));
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Squirrel"), 0);
}

#[test]
fn doom_weaver_draws_the_paired_creatures_last_known_power() {
    cr!("608.2h", "702.95a");
    ruling!(
        "Doom Weaver",
        "Use the creature's power as it last existed on the battlefield to determine how many cards to draw."
    );
    supported("Doom Weaver");
    let mut t = TestGame::new(2);
    let bears = with_counters(&mut t, P0, "Grizzly Bears", 2);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Doom Weaver");
    t.resolve_all();
    let hand = t.hand_size(P0);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 4, "the 4/4 Bears' power");
}

#[test]
fn jenova_draws_the_dead_mutants_last_known_power() {
    cr!("608.2h", "603.10a");
    ruling!(
        "Jenova, Ancient Calamity",
        "Use the power of the Mutant that died as it last existed on the battlefield to determine how many cards to draw with Jenova's last ability."
    );
    supported("Jenova, Ancient Calamity");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jenova, Ancient Calamity");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 1);
    assert!(t.obj_now(bears).chars.has_subtype("Mutant"));
    giant_growth(&mut t, bears);
    assert_eq!(t.pt(bears), (6, 6));
    let hand = t.hand_size(P0);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 6);
}

#[test]
fn rhovanion_rampager_uses_the_sacrificed_creatures_last_known_power() {
    cr!("608.2h", "603.10a");
    ruling!(
        "Rhovanion Rampager",
        "Use the sacrificed creature's power as it last existed on the battlefield to determine how many counters Rhovanion Rampager gets."
    );
    supported("Rhovanion Rampager");
    let mut t = TestGame::new(2);
    let rampager = t.battlefield(P0, "Rhovanion Rampager");
    let bears = with_counters(&mut t, P0, "Grizzly Bears", 1);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    attack_with(&mut t, &[(rampager, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(plus1(&t, rampager), 3);
}

// --- X determined as the ability resolves --------------------------------------------------

/// The cards of the scry decisions asked so far.
fn scried(t: &TestGame) -> Vec<usize> {
    t.asked()
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::Scry { cards } => Some(cards.len()),
            _ => None,
        })
        .collect()
}

#[test]
fn spotter_thopter_scries_its_power_on_resolution_or_last_known() {
    cr!("608.2h", "701.22a");
    ruling!(
        "Spotter Thopter",
        "Use Spotter Thopter's power at the time its triggered ability resolves (not the time it triggered) to determine the value of X. If Spotter Thopter is no longer on the battlefield at the time the ability resolves, use its power as it existed the last time it was on the battlefield."
    );
    supported("Spotter Thopter");
    // Pumped in response: scry 7.
    let mut t = TestGame::new(2);
    let thopter = t.enter(P0, "Spotter Thopter");
    t.settle();
    assert_eq!(stack(&t), 1);
    giant_growth(&mut t, thopter);
    t.resolve_all();
    assert_eq!(scried(&t), vec![7]);
    // Pumped, then destroyed in response: scry 7 (its last known power).
    let mut t = TestGame::new(2);
    let thopter = t.enter(P0, "Spotter Thopter");
    t.settle();
    giant_growth(&mut t, thopter);
    destroy(&mut t, thopter);
    t.resolve_all();
    assert_eq!(scried(&t), vec![7]);
}

#[test]
fn emergent_woodwurm_looks_at_its_power_on_resolution_or_last_known() {
    cr!("608.2h");
    ruling!(
        "Emergent Woodwurm",
        "Use the power of the creature as the triggered ability resolves to determine the value of X. If the creature is no longer on the battlefield at that time, use its power from when it was last on the battlefield."
    );
    supported("Emergent Woodwurm");
    for gone in [false, true] {
        let mut t = TestGame::new(2);
        let wurm = t.battlefield(P0, "Emergent Woodwurm");
        let lib = stack_library(
            &mut t,
            P0,
            &[
                "Divination",
                "Divination",
                "Divination",
                "Divination",
                "Divination",
                "Divination",
                "Colossal Dreadmaw",
            ],
        );
        attack_with(&mut t, &[(wurm, Entity::Player(P1))]);
        assert_eq!(stack(&t), 1);
        giant_growth(&mut t, wurm);
        if gone {
            destroy(&mut t, wurm);
        }
        t.answer_choose(P0, &[obj(lib[6])]);
        t.resolve_all();
        assert_eq!(
            t.named_on_battlefield("Colossal Dreadmaw").len(),
            1,
            "X is 7 (gone: {gone})"
        );
    }
}

#[test]
fn vincent_checks_its_power_as_the_chaos_ability_resolves() {
    cr!("603.4", "608.2h");
    ruling!(
        "Vincent, Vengeful Atoner",
        "Vincent's last ability doesn't care what Vincent's power is when it deals combat damage to an opponent; it only matters what its power is when the ability resolves."
    );
    supported("Vincent, Vengeful Atoner");
    for shrink in [false, true] {
        let mut t = TestGame::new(3);
        let vincent = with_counters(&mut t, P0, "Vincent, Vengeful Atoner", 4);
        assert_eq!(t.pt(vincent).0, 7);
        t.answer(
            P0,
            DecisionKind::Attackers,
            Answer::Attackers(vec![(vincent, Entity::Player(P1))]),
        );
        t.advance_to(P0, Step::CombatDamage);
        assert_eq!(t.life(P1), 13);
        t.settle();
        assert!(stack(&t) >= 1);
        if shrink {
            let v = t.g.current(vincent);
            t.g.remove_counters(Entity::Object(v), "+1/+1", 4);
            t.g.recompute();
        }
        t.resolve_all();
        let expected = if shrink { 20 } else { 13 };
        assert_eq!(t.life(P2), expected, "shrunk before resolution: {shrink}");
    }
}

#[test]
fn meglonoth_checks_its_power_on_resolution() {
    cr!("608.2h");
    ruling!(
        "Meglonoth",
        "You don’t check Meglonoth’s power until its triggered ability resolves."
    );
    supported("Meglonoth");
    let mut t = TestGame::new(2);
    let meglo = t.battlefield(P0, "Meglonoth");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(giant, Entity::Player(P0))]),
    );
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(meglo, giant)]),
    );
    t.advance_to(P1, Step::DeclareBlockers);
    t.settle();
    assert_eq!(stack(&t), 1);
    giant_growth(&mut t, meglo);
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 9);
}

#[test]
fn cultivator_of_blades_chooses_and_counts_on_resolution() {
    cr!("608.2h", "608.2d");
    ruling!(
        "Cultivator of Blades",
        "You choose whether or not to have your other attacking creatures get +X/+X as Cultivator of Blades's triggered ability resolves, not as you put it onto the stack."
    );
    supported("Cultivator of Blades");
    let mut t = TestGame::new(2);
    let cultivator = t.battlefield(P0, "Cultivator of Blades");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = n_asked(&t);
    attack_with(
        &mut t,
        &[
            (cultivator, Entity::Player(P1)),
            (bears, Entity::Player(P1)),
        ],
    );
    assert_eq!(stack(&t), 1);
    let yes_no = |t: &TestGame| {
        t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
            .count()
    };
    assert_eq!(yes_no(&t), 0, "nothing chosen yet");
    giant_growth(&mut t, cultivator);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(yes_no(&t), 1);
    assert_eq!(t.pt(bears), (6, 6));
}

#[test]
fn stone_giant_destroys_the_creature_even_if_its_toughness_grew() {
    cr!("603.7c");
    ruling!(
        "Stone Giant",
        "When the delayed triggered ability resolves, the targeted creature is destroyed, even if it’s no longer a creature, no longer under your control, or no longer has toughness less than Stone Giant’s power at that time."
    );
    supported("Stone Giant");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Stone Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, giant, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    giant_growth(&mut t, bears);
    assert_eq!(t.pt(bears).1, 5);
    end_step(&mut t, P0);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

// --- targets and division ------------------------------------------------------------------

/// The divide decisions asked so far: (total, recipients, min each).
fn divides(t: &TestGame) -> Vec<(u32, usize, u32)> {
    t.asked()
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::Divide {
                total,
                recipients,
                min_each,
                ..
            } => Some((*total, recipients.len(), *min_each)),
            _ => None,
        })
        .collect()
}

#[test]
fn orca_divides_its_damage_as_the_trigger_is_put_on_the_stack() {
    cr!("603.3d", "601.2d");
    ruling!(
        "Orca, Siege Demon",
        "You announce how the damage will be divided as part of putting Orca, Siege Demon's last triggered ability on the stack. Each chosen target must receive at least 1 damage."
    );
    supported("Orca, Siege Demon");
    let mut t = TestGame::new(2);
    let orca = with_counters(&mut t, P0, "Orca, Siege Demon", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1), obj(bears)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![5, 2]));
    destroy(&mut t, orca);
    assert_eq!(stack(&t), 1);
    assert_eq!(divides(&t), vec![(7, 2, 1)], "divided before it resolves");
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn living_inferno_divides_its_damage_as_it_is_activated() {
    cr!("602.2b", "601.2d");
    ruling!(
        "Living Inferno",
        "You divide the damage among the target creatures as you activate Living Inferno's ability. Each target must be assigned at least 1 damage."
    );
    supported("Living Inferno");
    let mut t = TestGame::new(2);
    let inferno = t.battlefield(P0, "Living Inferno");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(bears), obj(giant)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 6]));
    t.activate(P0, inferno, 0, &[]).unwrap();
    assert_eq!(stack(&t), 1);
    assert_eq!(divides(&t), vec![(8, 2, 1)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // The Bears and the Giant dealt 5 damage to the 8/5 Inferno.
    assert!(t.in_graveyard(P0, "Living Inferno"));
}

/// P0's Ravenous Gigantotherium enters devouring nothing (a 3/3), targeting `targets`.
fn gigantotherium(t: &mut TestGame, targets: &[Entity], split: Vec<i64>) -> ObjectId {
    t.answer_choose(P0, &[]);
    t.answer_targets(P0, targets);
    if !split.is_empty() {
        t.answer(P0, DecisionKind::Divide, Answer::Numbers(split));
    }
    let g = t.enter(P0, "Ravenous Gigantotherium");
    t.settle();
    g
}

#[test]
fn ravenous_gigantotherium_divides_as_the_trigger_is_put_on_the_stack() {
    cr!("603.3d", "601.2d");
    ruling!(
        "Ravenous Gigantotherium",
        "You divide the damage as Ravenous Gigantotherium's triggered ability is put onto the stack, not as it resolves. Each target must be assigned at least 1 damage. You can't choose more than X targets and assign 0 damage to a target."
    );
    supported("Ravenous Gigantotherium");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Elite Vanguard");
    let from = n_asked(&t);
    let giga = gigantotherium(&mut t, &[obj(a), obj(b)], vec![2, 1]);
    assert_eq!(t.pt(giga), (3, 3));
    assert_eq!(stack(&t), 1);
    let max_targets: Vec<u32> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { max, .. } => Some(*max),
            _ => None,
        })
        .collect();
    assert_eq!(max_targets, vec![3], "up to X targets");
    assert_eq!(divides(&t), vec![(3, 2, 1)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Elite Vanguard"));
}

#[test]
fn ravenous_gigantotherium_with_zero_targets_deals_and_is_dealt_nothing() {
    cr!("115.1", "601.2c");
    ruling!(
        "Ravenous Gigantotherium",
        "You may choose zero targets. If you do, Ravenous Gigantotherium deals no damage and is dealt no damage."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let giga = gigantotherium(&mut t, &[], vec![]);
    t.resolve_all();
    assert!(t.on_battlefield(giga));
    assert_eq!(t.obj_now(giga).damage, 0);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn rangers_of_ithilien_with_an_illegal_target_doesnt_tempt() {
    cr!("608.2b", "701.54a");
    ruling!(
        "Rangers of Ithilien",
        "When Rangers of Ithilien's ability triggers, you can choose not to target a creature just to have the Ring tempt you. However, if you do choose a target, and that target is illegal at the time the ability tries to resolve, the ability won't resolve and none of its effects will happen. The Ring won't tempt you."
    );
    supported("Rangers of Ithilien");
    // No target: the Ring tempts you.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[]);
    t.enter(P0, "Rangers of Ithilien");
    t.resolve_all();
    assert_eq!(t.g.player(P0).ring_level, 1);
    // A target that grows too big: nothing happens.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Rangers of Ithilien");
    t.settle();
    assert_eq!(stack(&t), 1);
    giant_growth(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P1);
    assert_eq!(t.g.player(P0).ring_level, 0);
}

#[test]
fn velomachus_lets_you_cast_the_spell_right_away_ignoring_timing() {
    cr!("608.2g", "601.2");
    ruling!(
        "Velomachus Lorehold",
        "When Velomachus Lorehold’s triggered ability resolves, it lets you cast a spell immediately, ignoring timing restrictions. It does not let you cast that spell later in the turn."
    );
    supported("Velomachus Lorehold");
    for cast in [true, false] {
        let mut t = TestGame::new(2);
        let velo = t.battlefield(P0, "Velomachus Lorehold");
        let lib = stack_library(&mut t, P0, &["Divination"]);
        attack_with(&mut t, &[(velo, Entity::Player(P1))]);
        let chosen = if cast { vec![obj(lib[0])] } else { vec![] };
        t.answer_choose(P0, &chosen);
        t.answer_yes(P0, cast);
        let hand = t.hand_size(P0);
        t.resolve();
        if cast {
            // Divination (a sorcery) is cast during combat.
            assert_eq!(stack(&t), 1);
            t.resolve_all();
            assert_eq!(t.hand_size(P0), hand + 2);
        } else {
            assert_eq!(stack(&t), 0);
            assert!(!t.in_hand(P0, "Divination"));
            assert!(matches!(
                t.zone(t.g.current(lib[0])),
                mtg_engine::object::Zone::Library(_)
            ));
        }
    }
}

// --- blocking restrictions --------------------------------------------------------------

/// `name` (a 2/1) attacks and is blocked by P1's Grizzly Bears (power 2, not less), then
/// is pumped: it stays blocked.
fn stays_blocked(name: &str) {
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, name);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(attacker, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(bears, attacker)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(t.g.combat.as_ref().unwrap().is_blocked(attacker));
    giant_growth(&mut t, attacker);
    assert!(t.pt(attacker).0 > 2);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20, "{name} stayed blocked");
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn den_protector_and_formation_breaker_compare_power_only_as_blockers_are_declared() {
    cr!("509.1b", "506.4");
    ruling!(
        "Den Protector",
        "You compare Den Protector's power to the power of any creature trying to block it only as blockers are assigned. Once Den Protector has been legally blocked by a creature, changing the power of either creature won't change or undo the block."
    );
    ruling!(
        "Formation Breaker",
        "You compare Formation Breaker’s power to the power of any creature trying to block it only as blockers are declared. Once Formation Breaker has been legally blocked by a creature, changing the power of either creature won’t change or undo the block."
    );
    supported("Den Protector");
    supported("Formation Breaker");
    stays_blocked("Den Protector");
    stays_blocked("Formation Breaker");
}

// --- creatures put onto the battlefield attacking ------------------------------------------

/// In a three-player game, `attackers` attack P1 and P0 puts a Grizzly Bears from its
/// hand onto the battlefield attacking, choosing P2. Returns what the Bears attack.
fn put_in_attacking_p2(lead: &str, extra: usize) -> Option<Entity> {
    let mut t = TestGame::new(3);
    let lead = t.battlefield(P0, lead);
    let mut attackers = vec![(lead, Entity::Player(P1))];
    for _ in 0..extra {
        let e = t.battlefield(P0, "Elite Vanguard");
        attackers.push((e, Entity::Player(P1)));
    }
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    attack_with(&mut t, &attackers);
    t.resolve_all();
    let b = t.g.current(bears);
    assert!(t.on_battlefield(b));
    assert!(t.obj_now(b).tapped);
    let c = t.g.combat.as_ref().unwrap();
    assert!(!c.attacker(b).unwrap().declared);
    c.attack_target(b)
}

#[test]
fn shadowfax_and_taggerdy_creatures_may_attack_someone_else() {
    cr!("508.4");
    ruling!(
        "Shadowfax, Lord of Horses",
        "You choose which player, planeswalker, or battle the creature you put onto the battlefield is attacking as it enters the battlefield. It doesn't have to be attacking the same player, planeswalker, or battle that Shadowfax, Lord of Horses is attacking."
    );
    ruling!(
        "Paladin Elizabeth Taggerdy",
        "You choose which player, planeswalker, or battle the creature you put onto the battlefield is attacking as it enters the battlefield. It doesn’t have to be attacking the same player, planeswalker, or battle that Paladin Elizabeth Taggerdy is attacking."
    );
    supported("Shadowfax, Lord of Horses");
    supported("Paladin Elizabeth Taggerdy");
    assert_eq!(
        put_in_attacking_p2("Shadowfax, Lord of Horses", 0),
        Some(Entity::Player(P2))
    );
    assert_eq!(
        put_in_attacking_p2("Paladin Elizabeth Taggerdy", 2),
        Some(Entity::Player(P2))
    );
}

// --- mana abilities and costs --------------------------------------------------------------

#[test]
fn power_based_mana_abilities_dont_use_the_stack() {
    cr!("605.1a", "605.3a");
    ruling!(
        "Woodland Weavemaster",
        "Woodland Weavemaster's last ability is a mana ability. It doesn't use the stack and can't be responded to."
    );
    ruling!(
        "Gyre Sage",
        "Gyre Sage's last ability is a mana ability. It doesn't use the stack and can't be responded to."
    );
    // (card, +1/+1 counters, mana added)
    let cases = [("Woodland Weavemaster", 2, 3), ("Gyre Sage", 2, 2)];
    for (name, n, mana) in cases {
        supported(name);
        let mut t = TestGame::new(2);
        let c = with_counters(&mut t, P0, name, n);
        let r = t.activate(P0, c, 0, &[]).unwrap();
        assert!(r.is_none(), "{name}: nothing put on the stack");
        assert_eq!(stack(&t), 0);
        assert_eq!(pool_total(&t, P0), mana, "{name}");
    }
}

#[test]
fn helga_mana_can_be_split_between_spells() {
    cr!("106.4", "106.6");
    ruling!(
        "Helga, Skittish Seer",
        "You don't have to spend all the mana added by Helga's last ability on the same spell."
    );
    supported("Helga, Skittish Seer");
    let mut t = TestGame::new(2);
    let helga = with_counters(&mut t, P0, "Helga, Skittish Seer", 11);
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    t.activate(P0, helga, 0, &[]).unwrap();
    assert_eq!(pool_total(&t, P0), 12);
    let a = t.hand(P0, "Craw Wurm");
    let b = t.hand(P0, "Craw Wurm");
    t.cast(P0, a).go();
    assert_eq!(pool_total(&t, P0), 6);
    t.resolve_all();
    t.cast(P0, b).go();
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 0);
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 2);
}

#[test]
fn vodalian_mindsinger_kicks_once_for_each_kicker_cost() {
    cr!("702.33c", "702.33d");
    ruling!(
        "Vodalian Mindsinger",
        "You may kick Vodalian Mindsinger only once for its {1}{R} cost and only once for its {1}{G} cost. You can kick it twice by paying {2}{R}{G}, but you can't kick it twice by paying {2}{R}{R} or {2}{G}{G}. You can't kick it more than twice."
    );
    supported("Vodalian Mindsinger");
    // Both kickers: {1}{U}{U} + {1}{R} + {1}{G}.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 3);
    let from = n_asked(&t);
    let m = t.hand(P0, "Vodalian Mindsinger");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, m).go();
    let kicker_questions: Vec<bool> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::OptionalCost { repeatable, .. } => Some(*repeatable),
            _ => None,
        })
        .collect();
    assert_eq!(kicker_questions, vec![false, false], "each kicker once");
    t.resolve_all();
    let m = t.named_on_battlefield("Vodalian Mindsinger")[0];
    assert_eq!(plus1(&t, m), 4);
    // {2}{R}{R} pays for one kicker only.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 3);
    let m = t.hand(P0, "Vodalian Mindsinger");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, m).try_go().expect("castable kicked once");
    t.resolve_all();
    let m = t.named_on_battlefield("Vodalian Mindsinger")[0];
    assert_eq!(plus1(&t, m), 2, "kicked once: {{1}}{{R}} can't pay for {{1}}{{G}}");
}

// --- "~ deals damage equal to its power to that creature" ------------------------------

/// Damage marked on `id`.
fn damage_on(t: &TestGame, id: ObjectId) -> u32 {
    t.g.obj(t.g.current(id)).damage
}

#[test]
fn palazzo_archers_deals_its_own_last_known_power_to_the_attacker() {
    cr!("608.2h");
    ruling!(
        "Palazzo Archers",
        "If Palazzo Archers is no longer on the battlefield when its triggered ability resolves, use its power as it last existed on the battlefield to determine how much damage is dealt."
    );
    supported("Palazzo Archers");
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        // A 3/4 Archers; "its power" is the Archers', not the 4/4 attacker's.
        let archers = with_counters(&mut t, P0, "Palazzo Archers", 1);
        let elemental = t.battlefield(P1, "Air Elemental");
        t.g.turn.active = P1;
        attack_with(&mut t, &[(elemental, Entity::Player(P0))]);
        assert_eq!(stack(&t), 1);
        if leaves {
            destroy(&mut t, archers);
        }
        t.resolve_all();
        assert!(t.on_battlefield(elemental));
        assert_eq!(damage_on(&t, elemental), 3, "leaves: {leaves}");
    }
}

#[test]
fn abyssal_hunter_deals_its_own_power_to_a_tapped_target() {
    cr!("115.1");
    ruling!("Abyssal Hunter", "The ability can target an already tapped creature.");
    supported("Abyssal Hunter");
    let mut t = TestGame::new(2);
    let hunter = t.battlefield(P0, "Abyssal Hunter");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.tap(giant);
    t.lands(P0, "Swamp", 1);
    t.activate(P0, hunter, 0, &[obj(giant)]).unwrap();
    t.resolve_all();
    // 1 damage (the Hunter's power), not 3 (the Giant's).
    assert!(t.on_battlefield(giant));
    assert_eq!(damage_on(&t, giant), 1);
}
