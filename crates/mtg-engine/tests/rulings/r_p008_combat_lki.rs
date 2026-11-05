//! Rulings batch P008 — burn that can hit planeswalkers, from creatures: last known
//! information for sources that left the battlefield (CR 608.2h, 113.7a), triggers of
//! creatures dealt lethal damage (CR 603.10), attack triggers and the planeswalker they
//! attack (CR 506.4c), noncombat damage from combat triggers (CR 120.2b), blocks that
//! stay (CR 509.1h... 506.4a), simultaneous state-based actions (CR 704.3), and the
//! controller of "each opponent" (CR 109.5).

use crate::r_p007_common::*;
use crate::r_p008_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s02_common::destroy;
use crate::r_s06_common::give_control;
use crate::r_s25_common::cast_new;
use mtg_engine::decision::Answer;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const NORMAL: CastMethod = CastMethod::Normal;

#[test]
fn dragons_fire_uses_the_chosen_dragons_power_as_it_resolves() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Dragon's Fire",
        "If you chose a Dragon you control, use that Dragon's power as Dragon's Fire resolves to determine how much damage is dealt. If the Dragon you chose is no longer on the battlefield as Dragon's Fire resolves, Dragon's Fire will deal damage equal to the power it had the last time it was on the battlefield."
    );
    // (pumped before it resolves, then left the battlefield)
    for (pump, gone) in [(false, false), (true, false), (true, true)] {
        let mut t = TestGame::new(2);
        let dragon = t.battlefield(P0, "Shivan Dragon");
        let target = t.battlefield(P1, "Indomitable Ancients");
        t.lands(P0, "Mountain", 2);
        let spell = t.hand(P0, "Dragon's Fire");
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
        t.answer_choose(P0, &[obj(dragon)]);
        t.cast(P0, spell).target(target).go();
        if pump {
            cast_new(&mut t, P0, "Giant Growth", &[obj(dragon)]);
            t.resolve();
        }
        if gone {
            destroy(&mut t, dragon);
        }
        t.resolve_all();
        let expected = if pump { 8 } else { 5 };
        assert_eq!(
            dmg(&t, target),
            expected,
            "pump {pump} gone {gone}: Indomitable Ancients (2/10)"
        );
    }
}

#[test]
fn sacrificed_sources_deal_damage_equal_to_their_last_known_power() {
    cr!("608.2h", "113.7a", "602.2b");
    ruling!(
        "Hellhole Flailer",
        "The damage dealt by the last ability is equal to Hellhole Flailer's power when it was last on the battlefield. For example, if it had a +1/+1 counter on it, it will deal 4 damage."
    );
    ruling!(
        "Heartfire Immolator",
        "Players can't take any actions between the time you begin to activate Heartfire Immolator's last ability and the time it's sacrificed. Notably, opponents can't try to lower its power."
    );
    supported("Hellhole Flailer");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Hellhole Flailer");
    t.g.objects[f.0 as usize].counters.insert("+1/+1".into(), 1);
    t.g.recompute();
    assert_eq!(t.pt(f), (4, 3));
    t.lands(P0, "Wastes", 2);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    t.activate(P0, f, 0, &[pl(P1)]).expect("Hellhole Flailer");
    assert!(!t.on_battlefield(f), "sacrificed as a cost");
    t.resolve_all();
    assert_eq!(t.life(P1), 16);

    // Heartfire Immolator pumped by prowess (3/3): it's sacrificed as the ability is
    // activated, so its power is locked in at 3 before anyone gets priority.
    supported("Heartfire Immolator");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Heartfire Immolator");
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_x(&mut t, P0, "Opt", NORMAL, None, &[]);
    t.resolve_all();
    assert_eq!(t.pt(h), (3, 3));
    t.lands(P0, "Mountain", 1);
    let from = t.asked().len();
    t.activate(P0, h, 0, &[obj(wurm)]).expect("Heartfire Immolator");
    assert!(!t.on_battlefield(h));
    assert!(
        !t.asked()[from..]
            .iter()
            .any(|(p, d)| *p == P1 && matches!(d, mtg_engine::decision::Decision::Priority { .. })),
        "no priority before the sacrifice"
    );
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 3);
}

#[test]
fn mogg_bombers_damage_doesnt_depend_on_the_sacrifice() {
    cr!("608.2c", "701.21a");
    ruling!(
        "Mogg Bombers",
        "The damage is dealt whether or not you sacrifice the Mogg Bombers. For instance, if Mogg Bombers is destroyed or sacrificed in between the time when the ability triggers and when it resolves, then the targeted player will still be dealt 3 damage."
    );
    let mut t = TestGame::new(2);
    let bombers = t.battlefield(P0, "Mogg Bombers");
    t.answer_targets(P0, &[pl(P1)]);
    t.enter(P1, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, bombers);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn mogg_bombers_sees_creatures_entering_with_it() {
    cr!("603.6a");
    ruling!(
        "Mogg Bombers",
        "The Mogg Bombers do trigger on any creatures that enter at the same time they do."
    );
    // Living Death returns Mogg Bombers and Grizzly Bears at the same time.
    supported("Living Death");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Mogg Bombers");
    t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[pl(P1)]);
    cast_x(&mut t, P0, "Living Death", NORMAL, None, &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_graveyard(P0, "Mogg Bombers"));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn spiteful_sliver_triggers_when_dealt_lethal_damage() {
    cr!("603.2", "113.7a", "704.5g", "510.2");
    ruling!(
        "Spiteful Sliver",
        "The ability granted by Spiteful Sliver will trigger even if the Sliver is dealt lethal damage. For example, if a 2/2 Sliver blocks a 7/7 creature, the Sliver’s ability will trigger and it will deal 7 damage."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Spiteful Sliver");
    let sliver = t.battlefield(P1, "Sliver Construct");
    let colossus = t.battlefield(P0, "Krosan Colossus");
    t.answer_targets(P1, &[pl(P0)]);
    attack_with(&mut t, &[(colossus, pl(P1))]);
    block_and_finish(&mut t, P1, &[(sliver, colossus)]);
    t.resolve_all();
    assert!(!t.on_battlefield(sliver));
    assert_eq!(t.life(P0), 20 - 9, "Krosan Colossus is a 9/9");
}

#[test]
fn spiteful_prankster_controller_loses_before_its_trigger() {
    cr!("704.3", "704.5a", "603.3");
    ruling!(
        "Spiteful Prankster",
        "If your life total is brought to 0 or less at the same time that a creature is dealt lethal damage, you lose the game before Spiteful Prankster’s ability goes on the stack."
    );
    // Three players: P1's Earthquake (X = 1) brings P0 to 0 and kills Llanowar Elves.
    let mut t = TestGame::new(3);
    t.g.players[0].life = 1;
    t.battlefield(P0, "Spiteful Prankster");
    let elves = t.battlefield(P2, "Llanowar Elves");
    t.answer_targets(P0, &[pl(P1)]);
    t.set_step(P1, Step::PrecombatMain);
    cast_x(&mut t, P1, "Earthquake", NORMAL, Some(1), &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(elves));
    assert!(t.has_lost(P0));
    assert_eq!((t.life(P1), t.life(P2)), (19, 19), "no trigger damage");
}

#[test]
fn scorch_spitter_keeps_attacking_after_the_planeswalker_dies() {
    cr!("506.4c", "704.5i");
    ruling!(
        "Scorch Spitter",
        "If the planeswalker Scorch Spitter is attacking is reduced to 0 loyalty from the damage caused by its ability, Scorch Spitter continues to attack. It can be blocked, and it won’t deal any combat damage if it’s not blocked."
    );
    for blocked in [false, true] {
        let mut t = TestGame::new(2);
        let theirs = pw(&mut t, P1, "Chandra, Torch of Defiance", 1);
        let spitter = t.battlefield(P0, "Scorch Spitter");
        let bears = t.battlefield(P1, "Grizzly Bears");
        attack_with(&mut t, &[(spitter, obj(theirs))]);
        t.resolve_all();
        assert!(!t.on_battlefield(theirs), "0 loyalty");
        let combat = t.g.combat.as_ref().expect("in combat");
        assert!(
            combat.attackers.iter().any(|a| a.id == spitter),
            "still attacking"
        );
        let from = t.g.turn_events.len();
        let blocks: Vec<(ObjectId, ObjectId)> = if blocked {
            vec![(bears, spitter)]
        } else {
            vec![]
        };
        block_and_finish(&mut t, P1, &blocks);
        assert_eq!(t.life(P1), 20, "blocked {blocked}");
        if blocked {
            assert_eq!(dmg(&t, bears), 1);
        } else {
            assert!(damage_events(&t, from).is_empty(), "no combat damage");
        }
    }
}

#[test]
fn attack_trigger_damage_isnt_combat_damage() {
    cr!("120.2a", "120.2b", "510.2");
    ruling!(
        "Scorch Spitter",
        "The damage Scorch Spitter deals as its ability resolves isn’t combat damage."
    );
    ruling!(
        "Questing Beast",
        "The damage Questing Beast deals to the target planeswalker as its last ability resolves isn't combat damage."
    );
    ruling!(
        "Questing Beast",
        "If the opponent dealt damage controls no planeswalkers, Questing Beast's last ability simply does nothing."
    );
    let mut t = TestGame::new(2);
    let spitter = t.battlefield(P0, "Scorch Spitter");
    let from = t.g.turn_events.len();
    attack_with(&mut t, &[(spitter, pl(P1))]);
    t.resolve_all();
    assert_eq!(damage_to(&t, from, pl(P1)), (1, false));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(damage_to(&t, from, pl(P1)), (2, true));

    // Questing Beast deals 4 combat damage to P1, then 4 noncombat to P1's planeswalker.
    let mut t = TestGame::new(2);
    let beast = t.battlefield(P0, "Questing Beast");
    let theirs = pw(&mut t, P1, "Chandra, Pyrogenius", 5);
    t.answer_targets(P0, &[obj(theirs)]);
    attack_with(&mut t, &[(beast, pl(P1))]);
    let from = t.g.turn_events.len();
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(damage_to(&t, from, pl(P1)), (4, true));
    assert_eq!(damage_to(&t, from, obj(theirs)), (4, false));
    assert_eq!(t.counters(theirs, "loyalty"), 1);
    // No planeswalkers: the trigger does nothing.
    let mut t = TestGame::new(2);
    let beast = t.battlefield(P0, "Questing Beast");
    let wurm = t.battlefield(P1, "Craw Wurm");
    attack_with(&mut t, &[(beast, pl(P1))]);
    let from = t.g.turn_events.len();
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(damage_events(&t, from).len(), 1, "only the combat damage");
    assert_eq!(dmg(&t, wurm), 0);
}

#[test]
fn scuttling_doom_engine_block_stays_after_the_blockers_power_drops() {
    cr!("509.1b", "506.4a");
    ruling!(
        "Scuttling Doom Engine",
        "Once Scuttling Doom Engine has been blocked by a creature, reducing that creature’s power to 2 or less won’t change or undo that block."
    );
    let mut t = TestGame::new(2);
    let engine = t.battlefield(P0, "Scuttling Doom Engine");
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(engine, pl(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(giant, engine)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    cast_new(&mut t, P0, "Dizzy Spell", &[obj(giant)]);
    t.resolve();
    assert_eq!(t.pt(giant).0, 0);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20, "still blocked");
    assert!(!t.on_battlefield(giant), "dealt 6");
}

#[test]
fn the_fallen_damages_only_its_current_controllers_opponents() {
    cr!("109.5", "102.2");
    ruling!(
        "The Fallen",
        "The Fallen’s ability causes it to deal damage only to opponents of its current controller. If The Fallen has dealt damage to you, and then you gain control of it, you won’t be damaged during your upkeep."
    );
    let mut t = TestGame::new(2);
    let fallen = t.battlefield(P0, "The Fallen");
    hit(&mut t, fallen, pl(P1), 2);
    assert_eq!(t.life(P1), 18);
    give_control(&mut t, fallen, P1);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 18));
    // Back under P0's control, it damages P1 again (it still remembers P1).
    give_control(&mut t, fallen, P0);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 17));
}

#[test]
fn raid_bombardment_triggers_for_each_attacker() {
    cr!("603.2", "508.3a");
    ruling!(
        "Raid Bombardment",
        "If you attack with multiple creatures with power 2 or less, Raid Bombardment's ability triggers for each of them separately."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raid Bombardment");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Llanowar Elves");
    let theirs = pw(&mut t, P1, "Chandra, Pyrogenius", 5);
    attack_with(&mut t, &[(a, pl(P1)), (b, obj(theirs))]);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.counters(theirs, "loyalty"), 4);
}
