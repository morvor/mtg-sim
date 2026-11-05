//! Rulings batch P043 — combat: "can't be blocked by creatures with power 2 or less"
//! once blocked (CR 509.1b, 509.1h), Old Fat Spider's targeting trigger, Butcher Orgg,
//! Personal Incarnation, "can't attack alone" (Sightless Brawler), the Reconfigure
//! Equipment creatures (Simian Sling, Chainflail Centipede, Cloudsteel Kirin, The Reality
//! Chip), and a Licid ending its effect in response to a spell targeting it.

use crate::r_p007_common::*;
use crate::r_p050_common::two_headed_giant;
use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s02_common::destroy;
use crate::r_s05_common::move_to;
use crate::r_s06_common::give_control;
use crate::r_s25_common::cast_new;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::combat::{attack_declaration_legal, attack_options};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn legal(t: &TestGame, decl: &[(ObjectId, Entity)]) -> bool {
    attack_declaration_legal(&t.g, &attack_options(&t.g), decl)
}

/// P0 attacks P1 with `attacker`; P1 blocks with `blocker`; the game stops in the
/// declare blockers step.
fn attack_blocked(t: &mut TestGame, attacker: ObjectId, blocker: ObjectId) {
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(t, &[(attacker, pl(P1))]);
    t.resolve_all();
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(blocker, attacker)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.resolve_all();
}

fn blocked_by(t: &TestGame, attacker: ObjectId) -> Vec<ObjectId> {
    t.g.combat
        .as_ref()
        .map(|c| c.blockers_of(attacker))
        .unwrap_or_default()
}

#[test]
fn a_block_by_a_bigger_creature_stays_when_its_power_drops() {
    cr!("509.1b", "506.4", "613.4c");
    ruling!(
        "Chomping Kavu",
        "Once Chomping Kavu has been legally blocked by a creature, changing that creature’s power to 2 or less won’t undo that block."
    );
    ruling!(
        "Bristlebane Outrider",
        "Once a creature with power 3 or greater has blocked Bristlebane Outrider, changing the power of the blocking creature won't cause Bristlebane Outrider to become unblocked."
    );
    ruling!(
        "Elegant Edgecrafters",
        "Once a creature with power 3 or greater has blocked Elegant Edgecrafters, changing the power of the blocking creature won't cause Elegant Edgecrafters to become unblocked."
    );
    ruling!(
        "Greater Sandwurm",
        "Once a creature with power 3 or greater has blocked Greater Sandwurm, changing the power of the blocking creature won’t cause Greater Sandwurm to become unblocked."
    );
    ruling!(
        "Outland Boar",
        "Once a creature with power 3 or greater has blocked Outland Boar, changing the power of the blocking creature won’t cause Outland Boar to become unblocked."
    );
    ruling!(
        "Prowler, Misguided Mentor",
        "Prowler's first ability applies only at the time blockers are declared. For example, if Prowler is blocked by a creature with 4 power, and then an effect gives that creature -3/-0, it won't stop blocking Prowler."
    );
    for name in [
        "Chomping Kavu",
        "Bristlebane Outrider",
        "Elegant Edgecrafters",
        "Greater Sandwurm",
        "Outland Boar",
        "Prowler, Misguided Mentor",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let attacker = t.battlefield(P0, name);
        // A 2-power creature can't block it.
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.set_step(P0, Step::BeginningOfCombat);
        attack_with(&mut t, &[(attacker, pl(P1))]);
        t.resolve_all();
        t.answer(
            P1,
            DecisionKind::Blockers,
            Answer::Blockers(vec![(bears, attacker)]),
        );
        t.advance_to(P0, Step::DeclareBlockers);
        assert!(
            blocked_by(&t, attacker).is_empty(),
            "{name}: the Bears can't block"
        );
        // Blocked by the Craw Wurm (6/4), which then gets -5/-0.
        let mut t = TestGame::new(2);
        let attacker = t.battlefield(P0, name);
        let blocker = t.battlefield(P1, "Craw Wurm");
        attack_blocked(&mut t, attacker, blocker);
        assert_eq!(blocked_by(&t, attacker), vec![blocker], "{name}");
        modify_until_eot(
            &mut t,
            blocker,
            vec![Modification::ModifyPT(Value::Const(-5), Value::Const(0))],
        );
        assert_eq!(t.pt(blocker).0, 1);
        assert_eq!(
            blocked_by(&t, attacker),
            vec![blocker],
            "{name}: still blocked"
        );
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(t.life(P1), 20, "{name}: blocked");
    }
}

#[test]
fn verdant_outrider_activating_after_blocks_doesnt_unblock_it() {
    cr!("509.1h", "506.4");
    ruling!(
        "Verdant Outrider",
        "Activating Verdant Outrider's ability after it has become blocked by a creature with power 2 or less won't cause it to become unblocked."
    );
    let mut t = TestGame::new(2);
    let rider = t.battlefield(P0, "Verdant Outrider");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_blocked(&mut t, rider, bears);
    t.lands(P0, "Forest", 2);
    t.activate(P0, rider, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(blocked_by(&t, rider), vec![bears]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(bears));
}

#[test]
fn harvesttide_sentry_coven_checks_on_resolution_and_then_lasts() {
    cr!("603.4", "509.1b");
    ruling!(
        "Harvesttide Sentry",
        "Harvesttide Sentry's triggered ability checks your creatures' powers both when it triggers and as it tries to resolve. If you somehow don't have enough creatures with different powers as it tries to resolve, nothing happens and Harvesttide Sentry can be blocked as normal."
    );
    ruling!(
        "Harvesttide Sentry",
        "Once its ability resolves, removing one of the creatures with different powers won't allow a player to block Harvesttide Sentry with creatures with power 2 or less."
    );
    supported("Harvesttide Sentry");
    for remove_before in [true, false] {
        let mut t = TestGame::new(2);
        let sentry = t.battlefield(P0, "Harvesttide Sentry");
        let goblin = t.battlefield(P0, "Raging Goblin");
        t.battlefield(P0, "Grizzly Bears");
        let blocker = t.battlefield(P1, "Grizzly Bears");
        t.set_step(P0, Step::Upkeep);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.settle();
        assert_eq!(t.stack_len(), 1, "the coven trigger");
        if remove_before {
            destroy(&mut t, goblin);
        }
        t.resolve_all();
        if !remove_before {
            destroy(&mut t, goblin);
        }
        t.answer(
            P0,
            DecisionKind::Attackers,
            Answer::Attackers(vec![(sentry, pl(P1))]),
        );
        t.answer(
            P1,
            DecisionKind::Blockers,
            Answer::Blockers(vec![(blocker, sentry)]),
        );
        t.advance_to(P0, Step::DeclareBlockers);
        assert_eq!(
            blocked_by(&t, sentry).is_empty(),
            !remove_before,
            "removed before it resolved: {remove_before}"
        );
    }
}

#[test]
fn old_fat_spider_triggers_once_and_resolves_first() {
    cr!("603.2", "603.3", "115.3", "701.6a");
    ruling!(
        "Old Fat Spider",
        "If a spell or ability targets Old Fat Spider more than once, its last ability still triggers only once."
    );
    ruling!(
        "Old Fat Spider",
        "Old Fat Spider's last ability resolves before the spell or ability that caused it to trigger. It resolves even if that spell or ability is countered or otherwise leaves the stack without resolving."
    );
    // Seeds of Strength targets it three times: one card drawn.
    let mut t = TestGame::new(2);
    let spider = t.battlefield(P0, "Old Fat Spider");
    t.set_step(P1, Step::PrecombatMain);
    cast_new(
        &mut t,
        P1,
        "Seeds of Strength",
        &[obj(spider), obj(spider), obj(spider)],
    );
    t.settle();
    assert_eq!(t.stack_len(), 2, "one trigger");
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.pt(spider), (9, 10));
    // Shock at it, countered by Cancel above the trigger: the trigger still resolves.
    let mut t = TestGame::new(2);
    let spider = t.battlefield(P0, "Old Fat Spider");
    t.set_step(P1, Step::PrecombatMain);
    let shock = cast_new(&mut t, P1, "Shock", &[obj(spider)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_ne!(
        t.g.stack.last().copied(),
        Some(shock),
        "the trigger is on top"
    );
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Cancel", &[obj(shock)]);
    t.resolve();
    assert!(t.in_graveyard(P1, "Shock"), "countered");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1, "drew a card");
    assert_eq!(dmg(&t, spider), 0);
}

#[test]
fn butcher_orgg_blocked_with_no_blockers_left_deals_no_damage() {
    cr!("506.4", "509.1h", "510.1c");
    ruling!(
        "Butcher Orgg",
        "If it is blocked but then all of its blockers are removed before combat damage is assigned, then it won't be able to deal combat damage and you won't be able to use its ability."
    );
    let mut t = TestGame::new(2);
    let orgg = t.battlefield(P0, "Butcher Orgg");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_blocked(&mut t, orgg, giant);
    destroy(&mut t, giant);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![4, 2]));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(t.on_battlefield(bears));
    assert!(!t.asked()[from..].iter().any(|(p, d)| *p == P0
        && matches!(
            d,
            Decision::YesNo { .. } | Decision::AssignCombatDamage { .. }
        )));
}

#[test]
fn butcher_orgg_must_assign_all_of_its_damage() {
    cr!("510.1b", "510.1c");
    ruling!(
        "Butcher Orgg",
        "If you choose to use the ability to divide up the damage (and there is at least one point of damage to be assigned), you must choose at least one creature or the defending player, and can't choose more total creatures and players than the amount of damage being assigned (minimum 1 point per player or creature)."
    );
    // Dividing 6 as 3 and 0 isn't accepted: all 6 is assigned among the recipients.
    let mut t = TestGame::new(2);
    let orgg = t.battlefield(P0, "Butcher Orgg");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![3, 0]));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(orgg, pl(P1))], &[]);
    let dealt: u32 =
        t.g.turn_events
            .iter()
            .filter_map(|e| match e {
                Event::Damage { source, amount, .. } if *source == orgg => Some(*amount),
                _ => None,
            })
            .sum();
    assert_eq!(dealt, 6, "{}", t.dump_log());
    let _ = bears;
    // Each recipient chosen gets at least 1: with 1 power (Orgg gets -5/-0), only one.
    let mut t = TestGame::new(2);
    let orgg = t.battlefield(P0, "Butcher Orgg");
    let bears = t.battlefield(P1, "Grizzly Bears");
    modify_until_eot(
        &mut t,
        orgg,
        vec![Modification::ModifyPT(Value::Const(-5), Value::Const(0))],
    );
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![0, 1]));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(orgg, pl(P1))], &[]);
    let recipients: Vec<Entity> =
        t.g.turn_events
            .iter()
            .filter_map(|e| match e {
                Event::Damage { source, target, .. } if *source == orgg => Some(*target),
                _ => None,
            })
            .collect();
    assert_eq!(recipients.len(), 1);
    let _ = bears;
}

#[test]
fn personal_incarnation_its_owner_loses_life_when_it_dies() {
    cr!("108.3", "700.4", "603.6c");
    ruling!(
        "Personal Incarnation",
        "The owner of the Incarnation loses life when it is destroyed, not the controller. So if you control your opponent's Incarnation you can let it die to make them lose life."
    );
    ruling!(
        "Personal Incarnation",
        "You do not lose life if this card is exiled or sent someplace without going to the graveyard first."
    );
    supported("Personal Incarnation");
    let mut t = TestGame::new(2);
    let inc = t.battlefield(P1, "Personal Incarnation");
    give_control(&mut t, inc, P0);
    t.g.players[P1.idx()].life = 15;
    destroy(&mut t, inc);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 7));
    let mut t = TestGame::new(2);
    let inc = t.battlefield(P0, "Personal Incarnation");
    move_to(&mut t, inc, Zone::Exile);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn sightless_brawler_cant_attack_alone() {
    cr!("506.5", "508.1c", "805.10a");
    ruling!(
        "Sightless Brawler",
        "Although Sightless Brawler or the creature it enchants can't attack alone, other attacking creatures don't have to attack the same player or planeswalker. For example, Sightless Brawler could attack an opponent and another creature could attack a planeswalker that opponent controls."
    );
    ruling!(
        "Sightless Brawler",
        "If you control more than one creature that can't attack alone, they can both attack together, even if no other creatures attack."
    );
    ruling!(
        "Sightless Brawler",
        "In a Two-Headed Giant game (or in another format using the shared team turns option), Sightless Brawler (or the creature it enchants) can attack with a creature controlled by your teammate, even if no other creatures you control attack."
    );
    supported("Sightless Brawler");
    let mut t = TestGame::new(2);
    let brawler = t.battlefield(P0, "Sightless Brawler");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let jace = pw(&mut t, P1, "Jace Beleren", 3);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!legal(&t, &[(brawler, pl(P1))]));
    assert!(legal(&t, &[(brawler, pl(P1)), (bears, obj(jace))]));
    // Two that can't attack alone: Sightless Brawler and Bears enchanted by a bestowed
    // Sightless Brawler.
    let mut t = TestGame::new(2);
    let brawler = t.battlefield(P0, "Sightless Brawler");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 4);
    let sb = t.hand(P0, "Sightless Brawler");
    t.cast(P0, sb)
        .target(bears)
        .method(mtg_engine::object::CastMethod::Keyword(
            mtg_engine::keywords::KeywordKind::Bestow,
        ))
        .go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 4));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!legal(&t, &[(bears, pl(P1))]));
    assert!(legal(&t, &[(brawler, pl(P1)), (bears, pl(P1))]));
    // Two-Headed Giant: with a creature P1 (P0's teammate) controls.
    let mut t = two_headed_giant();
    let brawler = t.battlefield(P0, "Sightless Brawler");
    let mate = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!legal(&t, &[(brawler, pl(P2))]));
    assert!(legal(&t, &[(brawler, pl(P2)), (mate, pl(P2))]));
}

#[test]
fn simian_sling_the_blocked_creature_deals_the_damage() {
    cr!("301.5c", "702.151b", "509.1h", "603.2");
    ruling!(
        "Simian Sling",
        "If Simian Sling becomes blocked, Simian Sling deals the damage. If the equipped creature becomes blocked, that creature deals the damage, not Simian Sling."
    );
    supported("Simian Sling");
    for equipped in [false, true] {
        let mut t = TestGame::new(2);
        let sling = t.battlefield(P0, "Simian Sling");
        let attacker = if equipped {
            let bears = t.battlefield(P0, "Grizzly Bears");
            assert!(t.g.attach(sling, obj(bears)));
            t.g.recompute();
            bears
        } else {
            sling
        };
        let wall = t.battlefield(P1, "Wall of Stone");
        attack_blocked(&mut t, attacker, wall);
        assert_eq!(t.life(P1), 19, "equipped: {equipped}");
        let src: Vec<ObjectId> =
            t.g.turn_events
                .iter()
                .filter_map(|e| match e {
                    Event::Damage { source, target, .. } if *target == pl(P1) => Some(*source),
                    _ => None,
                })
                .collect();
        assert_eq!(src, vec![attacker], "equipped: {equipped}");
    }
}

#[test]
fn chainflail_centipede_pumps_the_creature_that_attacked() {
    cr!("603.2", "608.2h", "301.5c");
    ruling!(
        "Chainflail Centipede",
        "If the equipped creature is the one that attacks, it gets +2/+0 until end of turn, not Chainflail Centipede. This is true even if Chainflail Centipede is no longer attached to it by the time the ability resolves."
    );
    supported("Chainflail Centipede");
    let mut t = TestGame::new(2);
    let cent = t.battlefield(P0, "Chainflail Centipede");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t.g.attach(cent, obj(bears)));
    t.g.recompute();
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, pl(P1))]);
    assert_eq!(t.stack_len(), 1);
    t.g.unattach(cent);
    t.g.recompute();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    assert!(t.obj_now(cent).chars.is_creature());
    assert_eq!(t.pt(cent), (2, 2));
}

#[test]
fn a_licid_ending_its_effect_makes_an_enchantment_spell_fizzle() {
    cr!("608.2b", "116.2c", "205.1a");
    ruling!(
        "Calming Licid",
        "If a spell that can target enchantments but not creatures (such as Clear) is cast targeting the Licid after it has become an enchantment, but the Licid's controller pays the cost, the Licid will no longer be an enchantment. This will result in the spell failing to resolve for having no legal targets."
    );
    let mut t = TestGame::new(2);
    let licid = t.battlefield(P0, "Calming Licid");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 1);
    t.activate(P0, licid, 0, &[obj(giant)]).unwrap();
    t.resolve();
    assert!(!t.obj_now(licid).chars.is_creature());
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Clear", &[obj(licid)]);
    crate::r_p043_shields::end_licid_effect(&mut t, P0);
    assert!(t.obj_now(licid).chars.is_creature());
    t.resolve_all();
    assert!(t.on_battlefield(licid), "Clear didn't resolve");
    assert!(t.in_graveyard(P1, "Clear"));
}

#[test]
fn cloudsteel_kirin_you_cant_lose_and_opponents_cant_win() {
    cr!("104.3b", "104.3c", "104.3d", "104.3e", "810.8a");
    ruling!(
        "Cloudsteel Kirin",
        "No game effect can cause you to lose the game or cause any opponent to win the game while you control a creature equipped with Cloudsteel Kirin. It doesn't matter whether you have 0 or less life, you're forced to draw a card while your library is empty, you have ten or more poison counters, you're at the end of your Nine Lives, your opponent is a Triskaidekaphile, or so on. You keep playing."
    );
    ruling!(
        "Cloudsteel Kirin",
        "If you control a creature equipped with Cloudsteel Kirin in a Two-Headed Giant game, your team can't lose the game and the opposing team can't win the game."
    );
    supported("Cloudsteel Kirin");
    let mut t = TestGame::new(2);
    let kirin = t.battlefield(P0, "Cloudsteel Kirin");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t.g.attach(kirin, obj(bears)));
    t.g.recompute();
    // 0 life, 10 poison counters, a draw from an empty library.
    t.g.players[P0.idx()].life = 0;
    t.g.add_counters(pl(P0), "poison", 10, None);
    let n = t.library_size(P0) as u32;
    t.g.draw_cards(P0, n + 1);
    t.settle();
    assert!(!t.has_lost(P0));
    // Azor's Elocutors with five filibuster counters: P1 doesn't win.
    let elo = t.battlefield(P1, "Azor's Elocutors");
    t.g.add_counters(obj(elo), "filibuster", 4, None);
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(t.counters(elo, "filibuster") >= 5);
    assert!(t.g.result.is_none(), "{:?}", t.g.result);
    // Two-Headed Giant: P0's team at 0 life doesn't lose.
    let mut t = two_headed_giant();
    let kirin = t.battlefield(P0, "Cloudsteel Kirin");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t.g.attach(kirin, obj(bears)));
    t.g.recompute();
    let giant = t.battlefield(P2, "Hill Giant");
    let life = t.life(P1);
    t.g.deal_damage(giant, pl(P1), life as u32, false);
    t.settle();
    assert!(t.life(P0) <= 0);
    assert!(!t.has_lost(P0) && !t.has_lost(P1));
    assert!(t.g.result.is_none());
}

#[test]
fn the_reality_chip_unattaching_doesnt_affect_a_spell_cast_from_the_library() {
    cr!("601.2", "601.3", "305.1");
    ruling!(
        "The Reality Chip",
        "Once you cast a spell from the top of your library, causing The Reality Chip to no longer be attached to a creature will have no effect on that spell."
    );
    supported("The Reality Chip");
    let mut t = TestGame::new(2);
    let chip = t.battlefield(P0, "The Reality Chip");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t.g.attach(chip, obj(bears)));
    t.g.recompute();
    let giant = t.library_top(P0, "Hill Giant");
    t.lands(P0, "Mountain", 4);
    t.cast(P0, giant).go();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, bears);
    assert_eq!(t.obj_now(chip).attached_to, None);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
}
