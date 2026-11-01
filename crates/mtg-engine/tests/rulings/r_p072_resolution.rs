//! Rulings batch P072 — what a resolving spell or ability affects: the set of affected
//! objects and values like X are locked in as it resolves (CR 608.2h, 611.2c); a spell or
//! ability whose targets are all illegal doesn't resolve (CR 608.2b), and one with a legal
//! target left still does everything it can.

use crate::r_p007_common::loyalty;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s02_common::destroy;
use crate::r_s04_common::crew;
use crate::r_s06_common::{activate_containing, has_kw};
use crate::r_s19_common::add_lore;
use crate::r_s25_common::{cast_new, lands_for_cost};
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::{AbilityKind, Modification};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{counters, Color, ColorSet};
use mtg_engine::*;

fn make_vampire(t: &mut TestGame, id: ObjectId) {
    modify_until_eot(t, id, vec![Modification::AddSubtypes(vec!["Vampire".into()])]);
}

fn pump(t: &mut TestGame, id: ObjectId, p: i32) {
    modify_until_eot(t, id, vec![Modification::ModifyPT(mtg_engine::ability::Value::c(p), mtg_engine::ability::Value::c(0))]);
}

fn has_destroy_ability(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .any(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.contains("Destroy"))
}

// ---------------------------------------------------------------------------------------
// The affected set is locked in as the spell or ability resolves (CR 611.2c)
// ---------------------------------------------------------------------------------------

#[test]
fn mass_grants_affect_only_creatures_you_control_as_they_resolve() {
    cr!("611.2c");
    ruling!(
        "Flying Crane Technique",
        "Only creatures you control as Flying Crane Technique resolves will gain flying and double strike. Creatures that come under your control later in the turn will not."
    );
    ruling!(
        "Eagles of the North",
        "Eagles of the North's triggered ability affects only creatures you control at the time it resolves. Any creatures that come under your control later in the turn won't be affected."
    );
    ruling!(
        "Savage Offensive",
        "Only affects creatures you control when this spell resolves."
    );
    // Flying Crane Technique: "Untap all creatures you control. They gain flying and
    // double strike until end of turn."
    supported("Flying Crane Technique");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(bears);
    cast_new(&mut t, P0, "Flying Crane Technique", &[]);
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
    assert!(has_kw(&t, bears, KeywordKind::Flying));
    assert!(has_kw(&t, bears, KeywordKind::DoubleStrike));
    let later = t.battlefield(P0, "Hill Giant");
    assert!(!has_kw(&t, later, KeywordKind::Flying));
    assert!(!has_kw(&t, later, KeywordKind::DoubleStrike));
    // Eagles of the North: "When this creature enters, creatures you control get +1/+0
    // and gain first strike until end of turn."
    supported("Eagles of the North");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.enter(P0, "Eagles of the North");
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 2));
    assert!(has_kw(&t, bears, KeywordKind::FirstStrike));
    let later = t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(later), (3, 3));
    assert!(!has_kw(&t, later, KeywordKind::FirstStrike));
    // Savage Offensive (kicked): first strike and +1/+1.
    supported("Savage Offensive");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Savage Offensive");
    t.lands(P0, "Forest", 1);
    let spell = t.hand(P0, "Savage Offensive");
    t.cast(P0, spell).kicked(true).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
    assert!(has_kw(&t, bears, KeywordKind::FirstStrike));
    let later = t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(later), (3, 3));
    assert!(!has_kw(&t, later, KeywordKind::FirstStrike));
}

#[test]
fn great_train_heist_second_mode_locks_in_its_creatures() {
    cr!("611.2c", "702.172a");
    ruling!(
        "Great Train Heist",
        "Great Train Heist’s second mode affects only creatures you control at the time Great Train Heist resolves. Creatures you begin to control later in the turn and noncreature permanents that become creatures later in the turn won’t get +1/+0 or first strike."
    );
    supported("Great Train Heist");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let copter = t.battlefield(P0, "Smuggler's Copter");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 2);
    let spell = t.hand(P0, "Great Train Heist");
    t.cast(P0, spell).modes(&[1]).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 2));
    assert!(has_kw(&t, bears, KeywordKind::FirstStrike));
    let later = t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(later), (3, 3));
    assert!(!has_kw(&t, later, KeywordKind::FirstStrike));
    // A Vehicle crewed later becomes a creature without the bonus.
    assert!(crew(&mut t, P0, copter, &[bears]));
    t.resolve_all();
    assert!(t.obj_now(copter).is(mtg_engine::types::CardType::Creature));
    assert_eq!(t.pt(copter), (3, 3));
    assert!(!has_kw(&t, copter, KeywordKind::FirstStrike));
}

#[test]
fn vampire_bonuses_are_locked_in_as_the_spell_resolves() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Vampiric Fury",
        "Only Vampires you control when Vampiric Fury resolves will receive the bonus. Creatures that enter later in the turn, that you gain control of later in the turn, or that become Vampires later in the turn won't be affected."
    );
    ruling!(
        "Vampire's Zeal",
        "If the creature is a Vampire, it gets +2/+2 and gains first strike. If the creature becomes a Vampire later in the turn, it won’t gain first strike."
    );
    // Vampiric Fury: "Vampire creatures you control get +2/+0 and gain first strike until
    // end of turn." Blood Artist is a 0/1 Vampire.
    supported("Vampiric Fury");
    let mut t = TestGame::new(2);
    let artist = t.battlefield(P0, "Blood Artist");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Vampiric Fury", &[]);
    t.resolve_all();
    assert_eq!(t.pt(artist), (2, 1));
    assert!(has_kw(&t, artist, KeywordKind::FirstStrike));
    make_vampire(&mut t, bears);
    assert_eq!(t.pt(bears), (2, 2));
    assert!(!has_kw(&t, bears, KeywordKind::FirstStrike));
    let later = t.battlefield(P0, "Blood Artist");
    assert_eq!(t.pt(later), (0, 1));
    let stolen = t.battlefield(P1, "Blood Artist");
    crate::r_s06_common::give_control(&mut t, stolen, P0);
    assert_eq!(t.pt(stolen), (0, 1));
    assert!(!has_kw(&t, stolen, KeywordKind::FirstStrike));
    // Vampire's Zeal: "Target creature gets +2/+2 until end of turn. If it's a Vampire,
    // it gains first strike until end of turn."
    supported("Vampire's Zeal");
    let mut t = TestGame::new(2);
    let artist = t.battlefield(P0, "Blood Artist");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Vampire's Zeal", &[Entity::Object(artist)]);
    t.resolve_all();
    assert_eq!(t.pt(artist), (2, 3));
    assert!(has_kw(&t, artist, KeywordKind::FirstStrike));
    cast_new(&mut t, P0, "Vampire's Zeal", &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    assert!(!has_kw(&t, bears, KeywordKind::FirstStrike));
    make_vampire(&mut t, bears);
    assert!(!has_kw(&t, bears, KeywordKind::FirstStrike));
}

#[test]
fn swarm_surge_first_strike_stays_after_the_creature_gains_a_color() {
    cr!("611.2c");
    ruling!(
        "Swarm Surge",
        "A colorless creature given first strike by Swarm Surge won’t lose first strike if it stops being colorless later in the turn."
    );
    supported("Swarm Surge");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Swarm Surge", &[]);
    t.resolve_all();
    assert_eq!(t.pt(thopter), (2, 2));
    assert!(has_kw(&t, thopter, KeywordKind::FirstStrike));
    assert_eq!(t.pt(bears), (4, 2));
    assert!(!has_kw(&t, bears, KeywordKind::FirstStrike));
    modify_until_eot(
        &mut t,
        thopter,
        vec![Modification::SetColors(ColorSet::single(Color::Red))],
    );
    assert!(has_kw(&t, thopter, KeywordKind::FirstStrike));
}

#[test]
fn onward_locks_in_x_as_it_resolves() {
    cr!("608.2h");
    ruling!(
        "Onward // Victory",
        "The value of X is determined only as Onward resolves. It won't change later in the turn if other effects modify the creature's power."
    );
    supported("Onward // Victory");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let card = t.hand(P0, "Onward // Victory");
    t.cast(P0, card)
        .method(mtg_engine::object::CastMethod::Half(0))
        .target(bears)
        .go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    pump(&mut t, bears, 3);
    assert_eq!(t.pt(bears), (7, 2));
}

#[test]
fn rally_the_forces_affects_attackers_as_it_resolves_and_lasts_the_turn() {
    cr!("611.2c", "506.4");
    ruling!(
        "Rally the Forces",
        "Even though Rally the Forces affects only attacking creatures, the bonus will remain for the rest of the turn."
    );
    ruling!(
        "Rally the Forces",
        "If Rally the Forces is cast before attackers are declared or after combat ends, it won’t do anything."
    );
    supported("Rally the Forces");
    // During combat: the bonus outlasts combat.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    cast_new(&mut t, P0, "Rally the Forces", &[]);
    t.resolve_all();
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.pt(bears), (3, 2));
    assert!(has_kw(&t, bears, KeywordKind::FirstStrike));
    // Before attackers are declared: nothing.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Rally the Forces", &[]);
    t.resolve_all();
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert_eq!(t.pt(bears), (2, 2));
    assert!(!has_kw(&t, bears, KeywordKind::FirstStrike));
}

#[test]
fn firjas_retribution_chapters_affect_only_angels_you_control_as_they_resolve() {
    cr!("611.2c", "714.2b");
    ruling!(
        "Firja's Retribution",
        "Only Angels you control as the chapter II ability resolves will gain the activated ability. Similarly, only Angels you control as the chapter III ability resolves will gain double strike. In both cases, Angels that come under your control later in the turn won't gain the ability."
    );
    supported("Firja's Retribution");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Firja's Retribution");
    t.g.objects[saga.0 as usize]
        .counters
        .insert(counters::LORE.into(), 1);
    let angel = t.battlefield(P0, "Serra Angel");
    add_lore(&mut t, saga, 1);
    t.resolve_all();
    assert!(has_destroy_ability(&t, angel));
    let later = t.battlefield(P0, "Serra Angel");
    assert!(!has_destroy_ability(&t, later));
    // Chapter III (a new turn would end chapter II's effect; double strike is the point).
    let third = t.battlefield(P0, "Serra Angel");
    add_lore(&mut t, saga, 1);
    t.resolve_all();
    assert!(has_kw(&t, angel, KeywordKind::DoubleStrike));
    assert!(has_kw(&t, third, KeywordKind::DoubleStrike));
    let last = t.battlefield(P0, "Serra Angel");
    assert!(!has_kw(&t, last, KeywordKind::DoubleStrike));
}

#[test]
fn firjas_retribution_angel_power_is_compared_again_on_resolution() {
    cr!("608.2b", "113.7a");
    ruling!(
        "Firja's Retribution",
        "The power of the target creature must be less than the power of the Angel as you activate the ability. As the ability tries to resolve, the powers are compared again. If the power of the target creature is no longer less than the power of the Angel, the ability won't resolve. If the Angel is no longer on the battlefield at that time, use its power as it last existed on the battlefield to determine if the ability resolves."
    );
    // `case`: 0 = nothing changes, 1 = the target grows to the Angel's power, 2 = the
    // Angel leaves (its last known power, 4, still counts).
    for case in 0..3 {
        let mut t = TestGame::new(2);
        let saga = t.battlefield(P0, "Firja's Retribution");
        t.g.objects[saga.0 as usize]
            .counters
            .insert(counters::LORE.into(), 1);
        let angel = t.battlefield(P0, "Serra Angel");
        let bears = t.battlefield(P1, "Grizzly Bears");
        add_lore(&mut t, saga, 1);
        t.resolve_all();
        t.answer_targets(P0, &[Entity::Object(bears)]);
        activate_containing(&mut t, P0, angel, "Destroy").unwrap();
        match case {
            1 => pump(&mut t, bears, 2),
            2 => destroy(&mut t, angel),
            _ => {}
        }
        t.resolve_all();
        assert_eq!(t.on_battlefield(bears), case == 1, "case {case}");
    }
}

// ---------------------------------------------------------------------------------------
// Illegal targets (CR 608.2b)
// ---------------------------------------------------------------------------------------

#[test]
fn spells_whose_only_target_is_illegal_do_nothing() {
    cr!("608.2b");
    ruling!(
        "Guided Strike",
        "If the creature is an illegal target when Guided Strike tries to resolve, it won’t resolve and none of its effects will happen. You won’t draw a card."
    );
    ruling!(
        "Battlefield Promotion",
        "If the target creature is an illegal target by the time Battlefield Promotion tries to resolve, the spell doesn't resolve. You don't gain 2 life."
    );
    ruling!(
        "Angelic Gift",
        "If the target creature is an illegal target by the time Angelic Gift tries to resolve, it doesn't resolve. It won't enter the battlefield, so its enters-the-battlefield ability won't trigger."
    );
    ruling!(
        "Flight of Fancy",
        "If the target creature is an illegal target by the time Flight of Fancy tries to resolve, the Aura spell doesn't resolve. It won't enter the battlefield, so its enters-the-battlefield ability won't trigger."
    );
    for spell in [
        "Guided Strike",
        "Battlefield Promotion",
        "Angelic Gift",
        "Flight of Fancy",
    ] {
        supported(spell);
        for kill in [false, true] {
            let mut t = TestGame::new(2);
            let bears = t.battlefield(P0, "Grizzly Bears");
            cast_new(&mut t, P0, spell, &[Entity::Object(bears)]);
            let hand = t.hand_size(P0);
            if kill {
                destroy(&mut t, bears);
            }
            t.resolve_all();
            let drew = t.hand_size(P0) - hand;
            let gained = t.life(P0) - 20;
            if kill {
                assert_eq!((drew, gained), (0, 0), "{spell}");
                assert!(t.in_graveyard(P0, spell), "{spell}");
            } else {
                assert!(drew + gained as usize > 0, "{spell}");
            }
        }
    }
}

#[test]
fn ajani_inspiring_leader_targets_and_life_gain() {
    cr!("608.2b", "115.10a", "606.3");
    ruling!(
        "Ajani, Inspiring Leader",
        "If the target creature is an illegal target by the time Ajani’s second ability tries to resolve, the ability doesn’t resolve. No player will gain life."
    );
    ruling!(
        "Ajani, Inspiring Leader",
        "You may activate Ajani’s first ability without choosing any target. If you do, you will gain 2 life. However, if you do choose a target and the target creature is an illegal target by the time the ability tries to resolve, the ability doesn’t resolve. You won’t gain 2 life."
    );
    supported("Ajani, Inspiring Leader");
    // +2 with no target: gain 2 life.
    let mut t = TestGame::new(2);
    let ajani = t.battlefield(P0, "Ajani, Inspiring Leader");
    loyalty(&mut t, P0, ajani, 0, &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // +2 targeting a creature that's gone: nothing.
    let mut t = TestGame::new(2);
    let ajani = t.battlefield(P0, "Ajani, Inspiring Leader");
    let bears = t.battlefield(P0, "Grizzly Bears");
    loyalty(&mut t, P0, ajani, 0, &[Entity::Object(bears)]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // −3 targeting P1's creature that's gone: P1 gains no life.
    let mut t = TestGame::new(2);
    let ajani = t.battlefield(P0, "Ajani, Inspiring Leader");
    let bears = t.battlefield(P1, "Grizzly Bears");
    loyalty(&mut t, P0, ajani, 1, &[Entity::Object(bears)]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // Control: −3 resolving exiles it and P1 gains 2.
    let mut t = TestGame::new(2);
    let ajani = t.battlefield(P0, "Ajani, Inspiring Leader");
    t.battlefield(P1, "Grizzly Bears");
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    loyalty(&mut t, P0, ajani, 1, &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.life(P1), 22);
}

#[test]
fn triumph_of_gerrard_target_must_still_have_the_greatest_power() {
    cr!("608.2b", "714.2b");
    ruling!(
        "Triumph of Gerrard",
        "Triumph of Gerrard’s chapter abilities each target any creature you control with the greatest power among creatures you control. If the power of another creature you control becomes greater before that ability resolves, the target is illegal."
    );
    supported("Triumph of Gerrard");
    for grow in [false, true] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let saga = t.battlefield(P0, "Triumph of Gerrard");
        t.answer_targets(P0, &[Entity::Object(giant)]);
        add_lore(&mut t, saga, 1);
        if grow {
            pump(&mut t, bears, 3);
        }
        t.resolve_all();
        let expected = if grow { 0 } else { 1 };
        assert_eq!(t.counters(giant, counters::PLUS1), expected, "grow {grow}");
    }
}

#[test]
fn phyrexian_splicer_still_grants_when_the_first_target_lost_the_ability() {
    cr!("608.2b");
    ruling!(
        "Phyrexian Splicer",
        "If the target which is having the ability removed does not have that ability during the resolution of this effect, then this effect still grants the chosen ability. The reason is that the second target is still legal even if the first one is not."
    );
    supported("Phyrexian Splicer");
    // Swab Goblin (an outlaw) has first strike from At Knifepoint on P0's turn.
    let mut t = TestGame::new(2);
    let splicer = t.battlefield(P0, "Phyrexian Splicer");
    let knifepoint = t.battlefield(P0, "At Knifepoint");
    let pirate = t.battlefield(P0, "Swab Goblin");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    assert!(has_kw(&t, pirate, KeywordKind::FirstStrike));
    // "Choose flying, first strike, trample, or shadow".
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer_targets(P0, &[Entity::Object(pirate)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_containing(&mut t, P0, splicer, "Choose").unwrap();
    destroy(&mut t, knifepoint);
    assert!(!has_kw(&t, pirate, KeywordKind::FirstStrike));
    t.resolve_all();
    assert!(has_kw(&t, bears, KeywordKind::FirstStrike));
}
