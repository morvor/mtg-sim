//! Rulings batch P214 — fight (CR 701.14): "Each deals damage equal to its power to the
//! other." A fight with a creature that left the battlefield, or with an illegal target,
//! doesn't happen (CR 701.14b, 608.2b).

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s05_common::*;
use crate::r_s06_common::activate_containing;
use crate::r_s07_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Gives `id` (followed across zone changes) hexproof until end of turn, as an effect of
/// P1's: an illegal target for P0's spells and abilities.
fn hexproof(t: &mut TestGame, id: ObjectId) {
    use mtg_engine::ability::*;
    run_from(
        t,
        P1,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::AddKeyword(mtg_engine::keywords::Keyword::new(
                KeywordKind::Hexproof,
            ))],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(t.g.current(id))],
    );
}

#[test]
fn dromokas_command_doesnt_prevent_fight_damage() {
    cr!("701.14a", "615.1a", "609.7");
    ruling!(
        "Dromoka's Command",
        "Although most instants and sorceries that cause damage to be dealt are the sources of that damage, a few cause other objects to deal damage (for example, spells that cause creatures to fight). The first ability of Dromoka's Command won't prevent damage caused by those other sources."
    );
    supported("Dromoka's Command");
    supported("Prey Upon");
    // P1 casts Prey Upon (P1's Hill Giant fights P0's Grizzly Bears). P0 responds with
    // Dromoka's Command: prevent all damage Prey Upon would deal this turn, and a +1/+1
    // counter on the Bears (3/3). The Giant still deals its damage.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Forest", 1);
    let pu = t.hand(P1, "Prey Upon");
    let prey = t
        .cast(P1, pu)
        .target(giant)
        .target(bears)
        .go();
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Plains", 1);
    let dc = t.hand(P0, "Dromoka's Command");
    t.cast(P0, dc).modes(&[0, 2]).target(prey).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (3, 3));
    t.resolve_all();
    assert!(resolved(&t, prey));
    // The Bears (3/3) dealt 3 damage to the Giant too.
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(giant));
}

#[test]
fn trample_does_nothing_in_a_fight() {
    cr!("701.14d", "702.19b");
    ruling!(
        "Voracious Hydra",
        "Because fighting isn't combat damage, trample has no effect during the fight."
    );
    supported("Voracious Hydra");
    // Voracious Hydra (0/1, trample, enters with X +1/+1 counters; "When this creature
    // enters, choose one — ... • This creature fights target creature you don't
    // control.") with X = 4 fights Grizzly Bears: none of the excess reaches P1.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 6);
    let vh = t.hand(P0, "Voracious Hydra");
    t.cast(P0, vh).x(4).go();
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 20);
    let hydra = t.g.current(vh);
    assert_eq!(damage_on(&t, hydra), 2);
    assert!(has_kw(&t, hydra, KeywordKind::Trample));
}

/// P0 casts Blizzard Brawl on P0's Grizzly Bears and P1's Grizzly Bears, controlling
/// `snow` Snow-Covered Forests. With `remove_one`, one of them leaves the battlefield
/// before it resolves; with `illegal`, P1's Bears gain hexproof. Returns (mine, theirs).
fn blizzard_brawl(snow: usize, remove_one: bool, illegal: bool) -> (TestGame, ObjectId, ObjectId) {
    supported("Blizzard Brawl");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let lands = t.lands(P0, "Snow-Covered Forest", snow.max(1));
    let bb = t.hand(P0, "Blizzard Brawl");
    t.cast(P0, bb).target(mine).target(theirs).go();
    if remove_one {
        move_to(&mut t, lands[0], Zone::Hand(P0));
    }
    if illegal {
        hexproof(&mut t, theirs);
    }
    t.resolve_all();
    (t, mine, theirs)
}

#[test]
fn blizzard_brawl_checks_snow_as_it_resolves() {
    cr!("608.2h", "701.14a");
    ruling!(
        "Blizzard Brawl",
        "Check whether you control three or more snow permanents as Blizzard Brawl is resolving to see if the target creature you control gets +1/+0 and gains indestructible. That creature will still fight even if it doesn’t get those bonuses."
    );
    // Three snow permanents: 3/2 indestructible; it kills theirs and survives.
    let (t, mine, theirs) = blizzard_brawl(3, false, false);
    assert!(t.on_battlefield(mine));
    assert_eq!(t.pt(mine), (3, 2));
    assert!(!t.on_battlefield(theirs));
    // Three as it was cast, two as it resolves: no bonus, but they still fight.
    let (t, mine, theirs) = blizzard_brawl(3, true, false);
    assert!(!t.on_battlefield(mine));
    assert!(!t.on_battlefield(theirs));
}

#[test]
fn blizzard_brawl_with_an_illegal_target_doesnt_fight() {
    cr!("701.14b", "608.2b");
    ruling!(
        "Blizzard Brawl",
        "If either target is an illegal target as Blizzard Brawl tries to resolve, neither creature will fight and no damage will be dealt."
    );
    let (t, mine, theirs) = blizzard_brawl(3, false, true);
    assert!(t.on_battlefield(mine) && t.on_battlefield(theirs));
    assert_eq!(damage_on(&t, mine), 0);
    assert_eq!(damage_on(&t, theirs), 0);
    // The legal creature still got its bonus.
    assert_eq!(t.pt(mine), (3, 2));
}

#[test]
fn setessan_tactics_fight_with_a_creature_that_left_deals_no_damage() {
    cr!("701.14a", "701.14b");
    ruling!(
        "Setessan Tactics",
        "Each of the two creatures that fight deals damage equal to its power to the other. If either one of the creatures isn't on the battlefield when the ability that instructs them to fight resolves, no damage is dealt."
    );
    supported("Setessan Tactics");
    // Setessan Tactics: "Until end of turn, any number of target creatures each get +1/+1
    // and gain '{T}: This creature fights another target creature.'"
    for leave in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        t.lands(P0, "Forest", 2);
        let st = t.hand(P0, "Setessan Tactics");
        t.cast(P0, st).targets(&[Entity::Object(bears)]).go();
        t.resolve_all();
        assert_eq!(t.pt(bears), (3, 3));
        t.answer_targets(P0, &[Entity::Object(giant)]);
        activate_containing(&mut t, P0, bears, "fights").expect("activate");
        if leave {
            move_to(&mut t, bears, Zone::Hand(P0));
        }
        t.resolve_all();
        if leave {
            assert_eq!(damage_on(&t, giant), 0);
        } else {
            // Each dealt damage equal to its power (3) to the other.
            assert!(!t.on_battlefield(bears) && !t.on_battlefield(giant));
        }
    }
}

#[test]
fn gargoss_controller_chooses_what_it_fights() {
    cr!("603.3d", "701.14a");
    ruling!(
        "Gargos, Vicious Watcher",
        "Gargos's controller chooses which creature (if any) Gargos will fight, not the controller of the spell."
    );
    supported("Gargos, Vicious Watcher");
    supported("Shock");
    // Gargos (8/7): "Whenever a creature you control becomes the target of a spell, Gargos
    // fights up to one target creature you don't control." P1 Shocks P0's Bears.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let gargos = t.battlefield(P0, "Gargos, Vicious Watcher");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.cast(P1, shock).target(bears).go();
    t.settle();
    // P0 (not P1) chose Gargos's target.
    let p0_choices = target_candidates(&t, P0, from);
    assert_eq!(p0_choices.len(), 1);
    assert!(p0_choices[0].contains(&Entity::Object(giant)));
    assert!(!p0_choices[0].contains(&Entity::Object(bears)));
    assert_eq!(target_candidates(&t, P1, from).len(), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert_eq!(damage_on(&t, gargos), 3);
}

#[test]
fn apex_altisaur_may_fight_several_times_in_a_row() {
    cr!("701.14a", "603.2");
    ruling!(
        "Apex Altisaur",
        "If Apex Altisaur fights a creature while either of its abilities is resolving, being dealt damage this way causes its second ability to trigger. It may fight several times in a row this way."
    );
    supported("Apex Altisaur");
    // Apex Altisaur (10/10): "When this creature enters, it fights up to one target
    // creature you don't control. Enrage — Whenever this creature is dealt damage, it
    // fights up to one target creature you don't control."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer_targets(P0, &[Entity::Object(elves)]);
    t.answer_targets(P0, &[]);
    let apex = enter(&mut t, P0, "Apex Altisaur");
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(giant));
    assert!(!t.on_battlefield(elves));
    assert_eq!(damage_on(&t, apex), 2 + 3 + 1);
}

#[test]
fn faunsbane_troll_that_left_doesnt_fight_but_the_exile_applies() {
    cr!("701.14b", "608.2c", "614.1a");
    ruling!(
        "Faunsbane Troll",
        "If Faunsbane Troll is no longer on the battlefield or no longer a creature when its activated ability resolves, it doesn't fight the target creature, but the effect that causes the target creature to be exiled any time it would die that turn still applies."
    );
    supported("Faunsbane Troll");
    // Faunsbane Troll: "{1}, Sacrifice an Aura attached to this creature: This creature
    // fights target creature you don't control. If that creature would die this turn,
    // exile it instead. Activate only as a sorcery."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let troll = enter(&mut t, P0, "Faunsbane Troll");
    t.resolve_all();
    assert_eq!(t.pt(troll), (5, 5));
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    activate_containing(&mut t, P0, troll, "fights").expect("activate");
    destroy(&mut t, troll);
    t.resolve_all();
    assert_eq!(damage_on(&t, giant), 0);
    // Later that turn, the Giant would die: it's exiled instead.
    destroy(&mut t, giant);
    assert!(t.in_exile("Hill Giant"));
    assert!(!t.in_graveyard(P1, "Hill Giant"));
}

/// Two creatures (P0's Grizzly Bears, P1's Hill Giant) fight by `cast`; one of the
/// targets (`which`: 0 = mine, 1 = theirs) becomes illegal in response. Neither deals
/// damage.
fn fight_with_an_illegal_target(cast: fn(&mut TestGame, ObjectId, ObjectId), which: usize) {
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    cast(&mut t, bears, giant);
    if which == 0 {
        // P0's creature left the battlefield.
        move_to(&mut t, bears, Zone::Hand(P0));
    } else {
        hexproof(&mut t, giant);
    }
    t.resolve_all();
    assert_eq!(damage_on(&t, giant), 0);
    if which == 1 {
        assert_eq!(damage_on(&t, bears), 0);
        assert!(t.on_battlefield(bears));
    }
}

#[test]
fn fight_with_an_illegal_target_prepare_fight() {
    cr!("701.14b", "608.2b", "702.127a");
    ruling!(
        "Prepare // Fight",
        "If either or both targets are illegal when Fight tries to resolve, no creature will deal or be dealt damage."
    );
    supported("Prepare // Fight");
    // Fight (aftermath, from the graveyard): "Target creature you control fights target
    // creature an opponent controls."
    fn cast(t: &mut TestGame, mine: ObjectId, theirs: ObjectId) {
        t.lands(P0, "Forest", 4);
        let pf = t.graveyard(P0, "Prepare // Fight");
        t.cast(P0, pf)
            .method(CastMethod::Keyword(KeywordKind::Aftermath))
            .target(mine)
            .target(theirs)
            .go();
    }
    fight_with_an_illegal_target(cast, 1);
    fight_with_an_illegal_target(cast, 0);
}

#[test]
fn fight_with_an_illegal_target_pit_fight() {
    cr!("701.14b", "608.2b");
    ruling!(
        "Pit Fight",
        "If either target of Pit Fight is an illegal target when the ability tries to resolve, neither creature will deal or be dealt damage."
    );
    supported("Pit Fight");
    // Pit Fight: "Target creature you control fights another target creature."
    fn cast(t: &mut TestGame, mine: ObjectId, theirs: ObjectId) {
        t.lands(P0, "Forest", 2);
        let pf = t.hand(P0, "Pit Fight");
        t.cast(P0, pf).target(mine).target(theirs).go();
    }
    fight_with_an_illegal_target(cast, 1);
    fight_with_an_illegal_target(cast, 0);
}

#[test]
fn gruul_ragebeast_fight_with_a_creature_that_left() {
    cr!("701.14b", "608.2b");
    ruling!(
        "Gruul Ragebeast",
        "If one or both of the creatures that are supposed to fight are no longer on the battlefield when the ability resolves, the fight won't happen. Neither creature will deal or be dealt damage."
    );
    supported("Gruul Ragebeast");
    // Gruul Ragebeast: "Whenever this creature or another creature you control enters,
    // that creature fights target creature an opponent controls."
    for leaves in ["entering", "target"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Gruul Ragebeast");
        let giant = t.battlefield(P1, "Hill Giant");
        let other = t.battlefield(P1, "Craw Wurm");
        t.answer_targets(P0, &[Entity::Object(giant)]);
        let bears = enter(&mut t, P0, "Grizzly Bears");
        assert_eq!(t.stack_len(), 1);
        if leaves == "entering" {
            move_to(&mut t, bears, Zone::Hand(P0));
        } else {
            move_to(&mut t, giant, Zone::Hand(P1));
        }
        t.resolve_all();
        assert_eq!(damage_on(&t, other), 0);
        if leaves == "entering" {
            assert_eq!(damage_on(&t, giant), 0);
        } else {
            assert!(t.on_battlefield(bears));
            assert_eq!(damage_on(&t, bears), 0);
        }
    }
}

#[test]
fn wild_instincts_with_an_illegal_opponents_creature_still_pumps() {
    cr!("608.2b", "701.14b");
    ruling!(
        "Wild Instincts",
        "If the creature an opponent controls is an illegal target as Wild Instincts tries to resolve, but the creature you control is still a legal target, the creature you control will get +2/+2, but the creatures won’t fight. Neither creature will deal or be dealt damage during the resolution of Wild Instincts."
    );
    supported("Wild Instincts");
    // Wild Instincts: "Target creature you control gets +2/+2 until end of turn. It fights
    // target creature an opponent controls."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Forest", 4);
    let wi = t.hand(P0, "Wild Instincts");
    t.cast(P0, wi).target(bears).target(giant).go();
    hexproof(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(damage_on(&t, bears), 0);
    assert_eq!(damage_on(&t, giant), 0);
}
