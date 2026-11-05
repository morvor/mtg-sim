//! Rulings batch P002 — Sin's counters (CR 122.8 and last known information), Crafty
//! Cutpurse and abilities granted to tokens (CR 603.3a), Garruk, Apex Predator's abilities
//! and emblem (CR 114, 306.4, 800.4a), counting cards in hand on resolution, hone counters,
//! Slithermuse and Reign of the Pit.

use crate::r_p003_common::*;
use crate::r_s01_common::{supported, tokens};
use mtg_engine::decision::Decision;
use mtg_engine::object::{ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Sin, Unending Cataclysm: "When Sin dies, put its counters on target creature you
// control, then shuffle this card into its owner's library."
// ---------------------------------------------------------------------------------------

/// P0's Sin with three +1/+1 counters and two charge counters.
fn sin(t: &mut TestGame) -> ObjectId {
    supported("Sin, Unending Cataclysm");
    let s = add(t, P0, "Sin, Unending Cataclysm");
    put(t, s, counters::PLUS1, 3);
    put(t, s, "charge", 2);
    s
}

#[test]
fn sin_puts_the_counters_it_had_on_the_target() {
    cr!("603.10a", "608.2h", "122.6");
    ruling!(
        "Sin, Unending Cataclysm",
        "Sin's last ability doesn't cause you to move counters from Sin onto the target creature. Rather, you put the same number of each kind of counter Sin had when it died onto the target creature."
    );
    let mut t = TestGame::new(2);
    let s = sin(&mut t);
    let giant = add(&mut t, P0, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    kill(&mut t, s);
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::PLUS1), 3);
    assert_eq!(t.counters(giant, "charge"), 2);
    assert!(!t.in_graveyard(P0, "Sin, Unending Cataclysm"), "shuffled away");
}

#[test]
fn sin_with_an_illegal_target_stays_in_the_graveyard() {
    cr!("608.2b");
    ruling!(
        "Sin, Unending Cataclysm",
        "If the target creature is an illegal target as Sin's last ability tries to resolve, it won't resolve and none of its effects will happen. Sin will remain in the zone it went to when it died"
    );
    let mut t = TestGame::new(2);
    let s = sin(&mut t);
    let giant = add(&mut t, P0, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    kill(&mut t, s);
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, giant);
    let library = t.library_size(P0);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Sin, Unending Cataclysm"));
    assert_eq!(t.library_size(P0), library);
}

#[test]
fn sin_and_the_ozolith_both_get_the_counters() {
    cr!("603.10a", "603.4");
    ruling!(
        "Sin, Unending Cataclysm",
        "you'll put the appropriate number of each kind of counter onto both The Ozolith and the target creature."
    );
    supported("The Ozolith");
    let mut t = TestGame::new(2);
    let oz = add(&mut t, P0, "The Ozolith");
    let s = sin(&mut t);
    let giant = add(&mut t, P0, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    kill(&mut t, s);
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::PLUS1), 3);
    assert_eq!(t.counters(giant, "charge"), 2);
    assert_eq!(t.counters(oz, counters::PLUS1), 3);
    assert_eq!(t.counters(oz, "charge"), 2);
}

// ---------------------------------------------------------------------------------------
// Crafty Cutpurse: "When this creature enters, each token that would be created under an
// opponent's control this turn is created under your control instead."
// ---------------------------------------------------------------------------------------

#[test]
fn crafty_cutpurse_a_stolen_token_s_own_trigger_is_yours() {
    cr!("603.3a", "614.1");
    ruling!(
        "Crafty Cutpurse",
        "the player who controls the token at the time the ability triggers will be the player who controls that ability. For example, if your opponent activates the second ability of Jace, Cunning Castaway"
    );
    supported("Crafty Cutpurse");
    supported("Jace, Cunning Castaway");
    let mut t = TestGame::new(2);
    enter(&mut t, P0, "Crafty Cutpurse");
    t.resolve_all();
    let jace = add(&mut t, P1, "Jace, Cunning Castaway");
    t.set_step(P1, Step::PrecombatMain);
    activate_resolve(&mut t, P1, jace, "Illusion", "", &[]);
    let illusion = tokens(&t, P0);
    assert_eq!(illusion.len(), 1);
    assert_eq!(tokens(&t, P1).len(), 0);
    // P1 targets the Illusion with Giant Growth: P0's trigger, P0 sacrifices it.
    cast(&mut t, P1, "Giant Growth", "G", &[obj(illusion[0])]);
    let trig = t.g.stack.last().copied().expect("the trigger");
    assert_eq!(t.obj_now(trig).controller, P0);
    t.resolve_all();
    assert!(!t.on_battlefield(illusion[0]));
    assert!(tokens(&t, P0).is_empty());
}

// ---------------------------------------------------------------------------------------
// Garruk, Apex Predator.
// ---------------------------------------------------------------------------------------

fn emblems(t: &TestGame) -> Vec<(PlayerId, PlayerId)> {
    t.g.objects
        .iter()
        .filter(|o| o.kind == ObjKind::Emblem && o.zone == Zone::Command)
        .map(|o| (o.owner, o.controller))
        .collect()
}

/// P0's Garruk with 8 loyalty uses its −8 on `p`.
fn garruk_emblem(t: &mut TestGame, p: PlayerId) {
    supported("Garruk, Apex Predator");
    let g = add(t, P0, "Garruk, Apex Predator");
    t.g.objects[g.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 8);
    activate_resolve(t, P0, g, "emblem", "", &[pl(p)]);
}

#[test]
fn garruk_s_emblem_belongs_to_its_target_and_stays_after_garruk_s_owner_leaves() {
    cr!("114.2", "114.4", "800.4a");
    ruling!(
        "Garruk, Apex Predator",
        "The emblem is owned and controlled by the target of the ability that created it. In a multiplayer game, if Garruk’s owner leaves the game, the emblem does not."
    );
    let mut t = TestGame::new(3);
    garruk_emblem(&mut t, P1);
    assert_eq!(emblems(&t), vec![(P1, P1)]);
    t.g.lose_game(P0);
    t.settle();
    assert_eq!(emblems(&t), vec![(P1, P1)]);
    // P2 attacks P1: +5/+5 and trample.
    let bears = add(&mut t, P2, "Grizzly Bears");
    t.set_step(P2, Step::BeginningOfCombat);
    crate::r_s01_common::attack_with(&mut t, &[(bears, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (7, 7));
    assert!(t
        .obj_now(bears)
        .has_keyword(mtg_engine::keywords::KeywordKind::Trample));
}

#[test]
fn garruk_s_emblem_ignores_attacks_on_planeswalkers() {
    cr!("508.1b", "603.2");
    ruling!(
        "Garruk, Apex Predator",
        "The emblem’s ability won’t trigger if a creature attacks a planeswalker."
    );
    let mut t = TestGame::new(2);
    garruk_emblem(&mut t, P1);
    let jace = add(&mut t, P1, "Jace Beleren");
    let bears = add(&mut t, P0, "Grizzly Bears");
    let giant = add(&mut t, P0, "Hill Giant");
    crate::r_s01_common::attack_with(&mut t, &[(bears, obj(jace)), (giant, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(giant), (8, 8));
}

#[test]
fn garruk_can_destroy_another_player_s_garruk() {
    cr!("306.4", "704.5j");
    ruling!(
        "Garruk, Apex Predator",
        "The first ability can target and destroy a Garruk planeswalker controlled by another player."
    );
    let mut t = TestGame::new(2);
    let mine = add(&mut t, P0, "Garruk, Apex Predator");
    let theirs = add(&mut t, P1, "Garruk, Apex Predator");
    assert!(t.on_battlefield(mine) && t.on_battlefield(theirs));
    activate_resolve(&mut t, P0, mine, "planeswalker", "", &[obj(theirs)]);
    assert!(!t.on_battlefield(theirs));
    assert!(t.on_battlefield(mine));
    assert_eq!(t.counters(mine, counters::LOYALTY), 6);
}

// ---------------------------------------------------------------------------------------
// Counting cards in hand on resolution: Balance of Power, Wheel of Torture.
// ---------------------------------------------------------------------------------------

#[test]
fn balance_of_power_and_wheel_of_torture_count_cards_on_resolution() {
    cr!("608.2h");
    ruling!(
        "Balance of Power",
        "The number of cards is counted during resolution."
    );
    ruling!(
        "Wheel of Torture",
        "The number of cards is counted during resolution."
    );
    supported("Balance of Power");
    supported("Wheel of Torture");
    // Balance of Power cast with P1 holding 5 cards to P0's 0; P1 casts two Giant Growths
    // in response (5 → 3): P0 draws 3, not 5.
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.hand(P1, "Grizzly Bears");
    }
    let bears = add(&mut t, P1, "Grizzly Bears");
    let b = t.hand(P0, "Balance of Power");
    mana(&mut t, P0, "3UU");
    t.answer_targets(P0, &[pl(P1)]);
    t.cast(P0, b).go();
    assert_eq!(t.hand_size(P0), 0);
    let growths = [t.hand(P1, "Giant Growth"), t.hand(P1, "Giant Growth")];
    assert_eq!(t.hand_size(P1), 5);
    for g in growths {
        mana(&mut t, P1, "G");
        t.cast(P1, g).target(obj(bears)).go();
    }
    assert_eq!(t.hand_size(P1), 3);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 3);
    // Wheel of Torture: P1 has 1 card as their upkeep begins; with the trigger on the
    // stack they cast it: 3 damage, not 2.
    let mut t = TestGame::new(2);
    add(&mut t, P0, "Wheel of Torture");
    let bears = add(&mut t, P1, "Grizzly Bears");
    let growth = t.hand(P1, "Giant Growth");
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(t.stack_len(), 1, "Wheel's trigger");
    mana(&mut t, P1, "G");
    t.cast(P1, growth).target(obj(bears)).go();
    assert_eq!(t.hand_size(P1), 0);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

// ---------------------------------------------------------------------------------------
// Hone counters: Sting, Bilbo's Sword and Dwalin, Weaponmaster.
// ---------------------------------------------------------------------------------------

#[test]
fn hone_counters_apply_only_while_the_equipment_is_attached() {
    cr!("122.1", "301.5c", "613.4c");
    ruling!(
        "Sting, Bilbo's Sword",
        "The power boost applies only while the Equipment with hone counters is attached to a creature. If an Equipment with hone counters on it becomes unattached from a creature, leaves the battlefield, or has its hone counters removed, the power of the creature changes immediately."
    );
    ruling!(
        "Dwalin, Weaponmaster",
        "The power boost applies only while the Equipment with hone counters is attached to a creature."
    );
    supported("Sting, Bilbo's Sword");
    supported("Dwalin, Weaponmaster");
    // Sting enters with P1 controlling two creatures: two hone counters, attached to the
    // Bears (2/2 → 4/2).
    let mut t = TestGame::new(2);
    add(&mut t, P1, "Hill Giant");
    add(&mut t, P1, "Hill Giant");
    let bears = add(&mut t, P0, "Grizzly Bears");
    let elves = add(&mut t, P0, "Llanowar Elves");
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[obj(bears)]);
    let sting = enter(&mut t, P0, "Sting, Bilbo's Sword");
    t.resolve_all();
    assert_eq!(t.counters(sting, "hone"), 2);
    assert_eq!(t.pt(bears), (4, 2));
    // Moved to the Elves.
    activate_resolve(&mut t, P0, sting, "Equip", "3", &[obj(elves)]);
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(elves), (3, 1));
    // A hone counter removed.
    let s = t.g.current(sting);
    t.g.remove_counters(obj(s), "hone", 1);
    t.g.recompute();
    assert_eq!(t.pt(elves), (2, 1));
    // Sting leaves.
    kill(&mut t, sting);
    assert_eq!(t.pt(elves), (1, 1));
    // Dwalin attacks: a hone counter on Bonesplitter; equipped Bears 2+2+1.
    let mut t = TestGame::new(2);
    let dwalin = add(&mut t, P0, "Dwalin, Weaponmaster");
    let bears = add(&mut t, P0, "Grizzly Bears");
    let elves = add(&mut t, P0, "Llanowar Elves");
    let split = add(&mut t, P0, "Bonesplitter");
    assert!(t.g.attach(split, obj(bears)));
    crate::r_s01_common::attack_with(&mut t, &[(dwalin, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.counters(split, "hone"), 1);
    assert_eq!(t.pt(bears), (5, 2));
    t.set_step(P0, Step::PostcombatMain);
    activate_resolve(&mut t, P0, split, "Equip", "1", &[obj(elves)]);
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(elves), (4, 1));
}

// ---------------------------------------------------------------------------------------
// Slithermuse, Reign of the Pit.
// ---------------------------------------------------------------------------------------

#[test]
fn slithermuse_chooses_the_opponent_on_resolution_and_locks_the_number() {
    cr!("608.2h", "603.3d");
    ruling!(
        "Slithermuse",
        "You choose an opponent when the ability resolves. Once you determine how many more cards than you that player has, that number is locked in"
    );
    supported("Slithermuse");
    // P0 holds 1 card, P1 holds 5 and P2 holds 2.
    let mut t = TestGame::new(3);
    t.hand(P0, "Grizzly Bears");
    for _ in 0..5 {
        t.hand(P1, "Grizzly Bears");
    }
    for _ in 0..2 {
        t.hand(P2, "Grizzly Bears");
    }
    let muse = add(&mut t, P0, "Slithermuse");
    let from = t.asked().len();
    kill(&mut t, muse);
    assert_eq!(t.stack_len(), 1);
    let chose_opponent = |t: &TestGame, from: usize| {
        t.asked()[from..].iter().any(|(p, d)| {
            *p == P0
                && matches!(d, Decision::ChooseEntities { candidates, .. }
                    if candidates.contains(&pl(P1)) && candidates.contains(&pl(P2)))
        })
    };
    assert!(!chose_opponent(&t, from), "not chosen as it triggers");
    t.answer_choose(P0, &[pl(P1)]);
    t.resolve_all();
    assert!(chose_opponent(&t, from));
    assert_eq!(t.hand_size(P0), 5, "1 + (5 - 1), not stopping once even");
}

#[test]
fn reign_of_the_pit_choices_in_turn_order_then_all_at_once() {
    cr!("101.4", "701.21a");
    ruling!(
        "Reign of the Pit",
        "You choose which creature you are sacrificing first, then each other player in turn order does the same. The creatures are all sacrificed simultaneously."
    );
    supported("Reign of the Pit");
    let mut t = TestGame::new(3);
    t.set_step(P1, Step::PrecombatMain);
    let mut chosen = vec![];
    for (p, a, b) in [
        (P0, "Grizzly Bears", "Hill Giant"),
        (P1, "Llanowar Elves", "Craw Wurm"),
        (P2, "Grizzly Bears", "Shivan Dragon"),
    ] {
        let x = add(&mut t, p, a);
        add(&mut t, p, b);
        t.answer_choose(p, &[obj(x)]);
        chosen.push(x);
    }
    // P0's choice is still on the battlefield as P2 chooses.
    let seen = crate::r_s01_common::watch(
        &mut t,
        P2,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| g.battlefield.len(),
    );
    let from = t.asked().len();
    cast_resolve(&mut t, P1, "Reign of the Pit", "4BB", &[]);
    let order: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order, vec![P1, P2, P0], "the caster first, then turn order");
    assert_eq!(seen.lock().unwrap().clone(), vec![6]);
    for x in &chosen {
        assert!(!t.on_battlefield(*x));
    }
    let demons = tokens(&t, P1);
    assert_eq!(demons.len(), 1);
    assert_eq!(t.pt(demons[0]), (5, 5), "2 + 1 + 2");
}

// ---------------------------------------------------------------------------------------
// Mutiny: "Target creature an opponent controls deals damage equal to its power to another
// target creature that player controls."
// ---------------------------------------------------------------------------------------

#[test]
fn mutiny_with_either_target_illegal_deals_no_damage() {
    cr!("608.2b", "115.1");
    ruling!(
        "Mutiny",
        "If either or both targets are illegal when Mutiny resolves, no creature will deal or be dealt damage."
    );
    supported("Mutiny");
    // 0: both legal (the Giant deals 3 to the Craw Wurm); 1: the Giant leaves; 2: the Wurm
    // leaves; 3: P2 gains control of the Wurm (no longer a creature "that player"
    // controls).
    for case in 0..4 {
        let mut t = TestGame::new(3);
        let giant = add(&mut t, P1, "Hill Giant");
        let wurm = add(&mut t, P1, "Craw Wurm");
        let spell = cast(&mut t, P0, "Mutiny", "R", &[obj(giant), obj(wurm)]);
        assert_eq!(crate::r_s25_common::targets_of(&t, spell), vec![obj(giant), obj(wurm)]);
        match case {
            1 => kill(&mut t, giant),
            2 => kill(&mut t, wurm),
            3 => crate::r_s06_common::give_control(&mut t, wurm, P2),
            _ => {}
        }
        t.resolve_all();
        let dealt = crate::r_s30_common::damage_events(&t);
        if case == 0 {
            assert_eq!(dealt, vec![(giant, obj(wurm), 3, false)]);
        } else {
            assert!(dealt.is_empty(), "case {case}: {dealt:?}");
        }
    }
}
