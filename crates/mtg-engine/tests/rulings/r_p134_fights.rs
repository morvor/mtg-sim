//! Rulings batch P134 — fights ("removal-fight"): two-target fight spells and abilities
//! whose targets become illegal (CR 608.2b, 701.14b), the bonuses some of them give first,
//! fight triggers whose source leaves the battlefield (CR 701.14b), and the cards around
//! them (Gargos, Gimli, Gruul Ragebeast, Brash Taunter, Primordial Plasm, Skophos
//! Maze-Warden, Cherished Hatchling, Ayula).

use crate::r_p113_common::{cast_modes, spoil, Spoil};
use crate::r_p134_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s05_common::run_from;
use crate::r_s07_common::{damage_on, has_kw, resolved};
use crate::r_s17_common::enter_transformed;
use crate::r_s25_common::lands_for_cost;
use mtg_engine::ability::*;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn spell(
    name: &'static str,
    modes: &'static [usize],
) -> impl Fn(&mut TestGame, ObjectId, ObjectId) {
    move |t, m, o| {
        cast_modes(t, P0, name, modes, &[ob(m), ob(o)]);
    }
}

fn tracker(t: &mut TestGame, m: ObjectId, o: ObjectId) {
    supported("Ulvenwald Tracker");
    let tr = t.battlefield(P0, "Ulvenwald Tracker");
    t.lands(P0, "Forest", 2);
    t.activate(P0, tr, 0, &[ob(m), ob(o)]).expect("Tracker");
}

fn ayula(t: &mut TestGame, m: ObjectId, o: ObjectId) {
    supported("Ayula, Queen Among Bears");
    t.battlefield(P0, "Ayula, Queen Among Bears");
    answer_modes(t, P0, &[1]);
    t.answer_targets(P0, &[ob(m)]);
    t.answer_targets(P0, &[ob(o)]);
    t.enter(P0, "Runeclaw Bear");
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

fn invasion(t: &mut TestGame, m: ObjectId, o: ObjectId) {
    supported("Invasion of Muraganda // Primordial Plasm");
    t.answer_targets(P0, &[ob(m)]);
    t.answer_targets(P0, &[ob(o)]);
    t.enter(P0, "Invasion of Muraganda // Primordial Plasm");
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

/// (name, start, P0's creature, power it deals with both targets legal)
type FightCase = (
    &'static str,
    Box<dyn Fn(&mut TestGame, ObjectId, ObjectId)>,
    &'static str,
    u32,
);

fn fight_cases() -> Vec<FightCase> {
    vec![
        ("Pounce", Box::new(spell("Pounce", &[])), "Grizzly Bears", 2),
        (
            "Inscription of Abundance",
            Box::new(spell("Inscription of Abundance", &[2])),
            "Grizzly Bears",
            2,
        ),
        (
            "Go for Blood",
            Box::new(spell("Go for Blood", &[])),
            "Grizzly Bears",
            2,
        ),
        (
            "Hunt the Weak",
            Box::new(spell("Hunt the Weak", &[])),
            "Grizzly Bears",
            3,
        ),
        ("Joust", Box::new(spell("Joust", &[])), "White Knight", 4),
        (
            "Khalni Ambush",
            Box::new(spell("Khalni Ambush // Khalni Territory", &[])),
            "Grizzly Bears",
            2,
        ),
        (
            "Prey Upon",
            Box::new(spell("Prey Upon", &[])),
            "Grizzly Bears",
            2,
        ),
        (
            "Savage Smash",
            Box::new(spell("Savage Smash", &[])),
            "Grizzly Bears",
            4,
        ),
        (
            "Savage Swipe",
            Box::new(spell("Savage Swipe", &[])),
            "Grizzly Bears",
            4,
        ),
        (
            "Savage Punch",
            Box::new(spell("Savage Punch", &[])),
            "Grizzly Bears",
            2,
        ),
        (
            "Swift Kick",
            Box::new(spell("Swift Kick", &[])),
            "Grizzly Bears",
            3,
        ),
        (
            "Temur Charm",
            Box::new(spell("Temur Charm", &[0])),
            "Grizzly Bears",
            3,
        ),
        (
            "Epic Confrontation",
            Box::new(spell("Epic Confrontation", &[])),
            "Grizzly Bears",
            3,
        ),
        (
            "Time to Feed",
            Box::new(spell("Time to Feed", &[])),
            "Grizzly Bears",
            2,
        ),
        (
            "Unnatural Aggression",
            Box::new(spell("Unnatural Aggression", &[])),
            "Grizzly Bears",
            2,
        ),
        (
            "Struggle for Skemfar",
            Box::new(spell("Struggle for Skemfar", &[])),
            "Grizzly Bears",
            3,
        ),
        (
            "Bridgeworks Battle",
            Box::new(spell("Bridgeworks Battle // Tanglespan Bridgeworks", &[])),
            "Grizzly Bears",
            4,
        ),
        ("Ulvenwald Tracker", Box::new(tracker), "Grizzly Bears", 2),
        ("Ayula", Box::new(ayula), "Grizzly Bears", 2),
        (
            "Invasion of Muraganda",
            Box::new(invasion),
            "Grizzly Bears",
            3,
        ),
    ]
}

#[test]
fn two_target_fights_deal_no_damage_if_either_target_is_illegal() {
    cr!("608.2b", "701.14a", "701.14b");
    ruling!("Pounce", "If either or both targets are illegal when Pounce tries to resolve, no creature will deal or be dealt damage.");
    ruling!("Inscription of Abundance", "If either target creature is an illegal target as Inscription of Abundance resolves with its last mode chosen, no creature will deal or be dealt damage.");
    ruling!("Go for Blood", "If either target is an illegal target as Go for Blood resolves, no creature will deal or be dealt damage.");
    ruling!("Hunt the Weak", "If either target is an illegal target as Hunt the Weak resolves, neither creature will deal or be dealt damage.");
    ruling!("Joust", "If either target is an illegal target as Joust tries to resolve, neither creature will deal or be dealt damage.");
    ruling!("Khalni Ambush // Khalni Territory", "If either target is an illegal target as Khalni Ambush resolves, no creature will deal or be dealt damage.");
    ruling!("Prey Upon", "If either target is an illegal target as Prey Upon resolves, no creature will deal or be dealt damage.");
    ruling!("Savage Smash", "If either target is an illegal target as Savage Smash resolves, neither creature will deal or be dealt damage.");
    ruling!("Savage Swipe", "If either target is an illegal target as Savage Swipe tries to resolve, neither creature will deal or be dealt damage.");
    ruling!("Savage Punch", "If either target is an illegal target when Savage Punch tries to resolve, neither creature will deal or be dealt damage.");
    ruling!("Swift Kick", "If either target is an illegal target when Swift Kick tries to resolve, neither creature will deal or be dealt damage.");
    ruling!("Ulvenwald Tracker", "If either target of Ulvenwald Tracker’s ability is an illegal target when the ability tries to resolve, neither creature will deal or be dealt damage.");
    ruling!("Temur Charm", "If the first mode was chosen and either target is an illegal target when Temur Charm tries to resolve, neither creature will deal or be dealt damage.");
    ruling!("Ayula, Queen Among Bears", "For Ayula's second mode, if either target isn't a creature or is otherwise an illegal target as the ability resolves, no creature will deal or be dealt damage.");
    for (name, start, mine, power) in fight_cases() {
        // Both legal: they fight.
        let (t, m, o) = fight_scenario(mine, "Colossal Dreadmaw", &start, Bad::None);
        assert_eq!(damage_on(&t, o), power, "{name}");
        assert!(!t.on_battlefield(m) || damage_on(&t, m) == 6, "{name}");
        // P0's creature is illegal (gone, stolen, shroud): no damage.
        for s in [Spoil::Gone, Spoil::Stolen, Spoil::Hexproof] {
            let (t, _, o) = fight_scenario(mine, "Colossal Dreadmaw", &start, Bad::Mine(s));
            assert_eq!(damage_on(&t, o), 0, "{name} {s:?}");
        }
        // The other creature is illegal (gone, hexproof): no damage.
        for s in [Spoil::Gone, Spoil::Hexproof] {
            let (t, m, _) = fight_scenario(mine, "Colossal Dreadmaw", &start, Bad::Theirs(s));
            assert!(t.on_battlefield(m), "{name} {s:?}");
            assert_eq!(damage_on(&t, m), 0, "{name} {s:?}");
        }
        let (t, m, o) = fight_scenario(mine, "Colossal Dreadmaw", &start, Bad::Both);
        assert_eq!((damage_on(&t, m), damage_on(&t, o)), (0, 0), "{name}");
    }
}

#[test]
fn the_bonus_applies_only_if_your_creature_is_still_a_legal_target() {
    cr!("608.2b", "608.2c", "701.14b");
    ruling!("Swift Kick", "If just the creature you control is an illegal target when Swift Kick tries to resolve, it won’t get +1/+0 until end of turn. If the creature you control is a legal target but the creature you don’t control isn’t, the creature you control will get +1/+0 until end of turn.");
    ruling!("Bridgeworks Battle // Tanglespan Bridgeworks", "If the creature you control is an illegal target as Bridgeworks Battle tries to resolve, no creature gets +2/+2. If that creature is a legal target but the creature you don't control isn't, the creature you control still gets +2/+2.");
    ruling!("Epic Confrontation", "If the creature you control is an illegal target as Epic Confrontation tries to resolve, the creature you control won't get +1/+2. If that creature is a legal target but the creature you don't control isn't, your creature will still get +1/+2. In both cases, neither creature will deal or be dealt damage.");
    ruling!("Hunt the Weak", "If the creature you control is an illegal target as Hunt the Weak tries to resolve, you won't put a +1/+1 counter on it. If that creature is a legal target but the other creature isn't, you'll still put the counter on the creature you control.");
    ruling!("Joust", "If the creature you control is an illegal target as Joust tries to resolve, no creature gets +2/+1. If that creature is a legal target but the creature you don't control isn't, it still gets +2/+1 until end of turn if it's a Knight.");
    ruling!("Savage Smash", "If the creature you control is an illegal target as Savage Smash tries to resolve, it won’t get +2/+2. If that creature is a legal target but the other creature isn’t, the creature you control still gets +2/+2.");
    ruling!("Savage Swipe", "If the creature you control is an illegal target as Savage Swipe tries to resolve, no creature gets +2/+2. If that creature is a legal target but the creature you don’t control isn’t, it still gets +2/+2 until end of turn if its power is 2.");
    ruling!("Struggle for Skemfar", "If the creature you control is an illegal target as Struggle for Skemfar tries to resolve, you won’t put a +1/+1 counter on it. If that creature is a legal target but the other creature isn’t, you’ll still put the +1/+1 counter on the creature you control.");
    ruling!("Invasion of Muraganda // Primordial Plasm", "If the creature you control is an illegal target as the ability tries to resolve, you won’t put a +1/+1 counter on it. If that creature is a legal target but the other creature isn’t, you’ll still put the +1/+1 counter on the creature you control, but neither creature will deal or be dealt damage.");
    ruling!("Temur Charm", "If the first mode was chosen and just the creature you control is an illegal target when Temur Charm tries to resolve, it won’t get +1/+1 until end of turn. If the creature you control is a legal target but the creature you don’t control isn’t, the creature you control will get +1/+1 until end of turn.");
    // (case, P0's creature, its P/T with only the other creature illegal)
    let bonus: &[(&str, &str, (i32, i32))] = &[
        ("Swift Kick", "Grizzly Bears", (3, 2)),
        ("Bridgeworks Battle", "Grizzly Bears", (4, 4)),
        ("Epic Confrontation", "Grizzly Bears", (3, 4)),
        ("Hunt the Weak", "Grizzly Bears", (3, 3)),
        ("Joust", "White Knight", (4, 3)),
        ("Savage Smash", "Grizzly Bears", (4, 4)),
        ("Savage Swipe", "Grizzly Bears", (4, 4)),
        ("Struggle for Skemfar", "Grizzly Bears", (3, 3)),
        ("Invasion of Muraganda", "Grizzly Bears", (3, 3)),
        ("Temur Charm", "Grizzly Bears", (3, 3)),
    ];
    let cases = fight_cases();
    for (name, mine, pt) in bonus {
        let start = &cases.iter().find(|c| c.0 == *name).unwrap().1;
        for s in [Spoil::Gone, Spoil::Hexproof] {
            let (t, m, _) = fight_scenario(mine, "Colossal Dreadmaw", start, Bad::Theirs(s));
            assert_eq!(t.pt(m), *pt, "{name} {s:?}");
            assert_eq!(damage_on(&t, m), 0, "{name} {s:?}");
        }
        for s in [Spoil::Stolen, Spoil::Hexproof] {
            let (t, m, o) = fight_scenario(mine, "Colossal Dreadmaw", start, Bad::Mine(s));
            assert_eq!(t.pt(m), (2, 2), "{name} {s:?}");
            assert_eq!(damage_on(&t, o), 0, "{name} {s:?}");
        }
    }
    // Joust's bonus is only for a Knight; Savage Swipe's only for power 2.
    let joust = &cases.iter().find(|c| c.0 == "Joust").unwrap().1;
    let (t, m, _) = fight_scenario(
        "Grizzly Bears",
        "Colossal Dreadmaw",
        joust,
        Bad::Theirs(Spoil::Hexproof),
    );
    assert_eq!(t.pt(m), (2, 2));
    let swipe = &cases.iter().find(|c| c.0 == "Savage Swipe").unwrap().1;
    let (t, m, _) = fight_scenario(
        "Hill Giant",
        "Colossal Dreadmaw",
        swipe,
        Bad::Theirs(Spoil::Hexproof),
    );
    assert_eq!(t.pt(m), (3, 3));
}

#[test]
fn unnatural_aggression_still_exiles_the_other_creature_if_yours_is_illegal() {
    cr!("608.2b", "614.1a");
    ruling!("Unnatural Aggression", "If the creature you control becomes an illegal target (perhaps due to being destroyed by another spell or ability), Unnatural Aggression will resolve, but the target creature an opponent controls will neither deal nor receive any damage. If that creature would die later in the turn for another reason, it will be exiled instead.");
    let start = spell("Unnatural Aggression", &[]);
    let (mut t, _, o) = fight_scenario(
        "Grizzly Bears",
        "Colossal Dreadmaw",
        start,
        Bad::Mine(Spoil::Gone),
    );
    assert!(t.in_graveyard(P0, "Unnatural Aggression"));
    assert_eq!(damage_on(&t, o), 0);
    destroy(&mut t, o);
    assert!(t.in_exile("Colossal Dreadmaw"));
    assert!(!t.in_graveyard(P1, "Colossal Dreadmaw"));
}

#[test]
fn time_to_feed_with_neither_target_legal_does_nothing() {
    cr!("608.2b", "603.7c");
    ruling!("Time to Feed", "If neither creature is a legal target when Time to Feed tries to resolve, the spell won't resolve and none of its effects will happen.");
    let start = spell("Time to Feed", &[]);
    let (mut t, m, o) = fight_scenario("Grizzly Bears", "Hill Giant", start, Bad::Both);
    assert_eq!((damage_on(&t, m), damage_on(&t, o)), (0, 0));
    // No delayed "when that creature dies" trigger was created.
    destroy(&mut t, o);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // With both legal, it fights and the delayed trigger gains 3 life.
    let start = spell("Time to Feed", &[]);
    let (t, _, o) = fight_scenario("Hill Giant", "Grizzly Bears", start, Bad::None);
    assert!(!t.on_battlefield(o));
    assert_eq!(t.life(P0), 23);
}

/// Atzocan Archer (1/4) enters targeting `target`; `bad` happens with the trigger on the
/// stack. Returns (game, archer).
fn archer(t: &mut TestGame, target: ObjectId) -> ObjectId {
    supported("Atzocan Archer");
    t.answer_targets(P0, &[ob(target)]);
    t.answer_yes(P0, true);
    let a = t.enter(P0, "Atzocan Archer");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    a
}

/// Territorial Allosaurus (5/5) cast kicked; its trigger targets `target`.
fn allosaurus(t: &mut TestGame, target: ObjectId) -> ObjectId {
    supported("Territorial Allosaurus");
    lands_for_cost(t, P0, "Territorial Allosaurus");
    t.lands(P0, "Wastes", 2);
    t.lands(P0, "Forest", 1);
    let card = t.hand(P0, "Territorial Allosaurus");
    t.answer_targets(P0, &[ob(target)]);
    t.cast(P0, card).kicked(true).go();
    t.resolve();
    assert_eq!(t.stack_len(), 1, "the kicked trigger");
    t.named_on_battlefield("Territorial Allosaurus")[0]
}

#[test]
fn fight_triggers_do_nothing_if_the_source_left_or_the_target_is_illegal() {
    cr!("701.14b", "608.2b");
    ruling!("Atzocan Archer", "If Atzocan Archer isn’t on the battlefield as its triggered ability resolves, or if the target of that ability is illegal, no creature will deal or be dealt damage.");
    ruling!("Territorial Allosaurus", "If Territorial Allosaurus isn't on the battlefield as its triggered ability resolves, or if the target of that ability is illegal, no creature will deal or be dealt damage.");
    let starts: [(&str, fn(&mut TestGame, ObjectId) -> ObjectId, u32); 2] = [
        ("Atzocan Archer", archer, 1),
        ("Territorial Allosaurus", allosaurus, 5),
    ];
    for (name, start, power) in starts {
        // Baseline: it fights.
        let mut t = TestGame::new(2);
        let wurm = t.battlefield(P1, "Craw Wurm");
        let me = start(&mut t, wurm);
        t.resolve_all();
        if power >= 4 {
            assert!(!t.on_battlefield(wurm), "{name}");
        } else {
            assert_eq!(damage_on(&t, wurm), power, "{name}");
        }
        assert!(!t.on_battlefield(me) || damage_on(&t, me) == 6, "{name}");
        // The source leaves: no damage.
        let mut t = TestGame::new(2);
        let wurm = t.battlefield(P1, "Craw Wurm");
        let me = start(&mut t, wurm);
        destroy(&mut t, me);
        t.resolve_all();
        assert_eq!(damage_on(&t, wurm), 0, "{name}");
        // The target gains hexproof: no damage.
        let mut t = TestGame::new(2);
        let wurm = t.battlefield(P1, "Craw Wurm");
        let me = start(&mut t, wurm);
        spoil(&mut t, wurm, Spoil::Hexproof);
        t.resolve_all();
        assert_eq!((damage_on(&t, wurm), damage_on(&t, me)), (0, 0), "{name}");
    }
}

#[test]
fn atzocan_archer_can_fight_your_own_creature() {
    cr!("701.14a", "115.1");
    ruling!("Atzocan Archer", "Atzocan Archer’s ability can target another creature you control (such as a Dinosaur with an enrage ability).");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let a = archer(&mut t, wurm);
    t.resolve_all();
    assert_eq!(damage_on(&t, wurm), 1);
    assert!(!t.on_battlefield(a), "the Wurm dealt 6 to the Archer");
}

#[test]
fn ayula_can_target_any_bear_you_control_including_itself() {
    cr!("603.3c", "115.1");
    ruling!("Ayula, Queen Among Bears", "Either of Ayula's modes can target any Bear you control, not just the Bear that entered the battlefield. Ayula can even be the target of its own ability.");
    // First mode: two +1/+1 counters on Ayula.
    let mut t = TestGame::new(2);
    let ay = t.battlefield(P0, "Ayula, Queen Among Bears");
    answer_modes(&mut t, P0, &[0]);
    t.answer_targets(P0, &[ob(ay)]);
    t.enter(P0, "Runeclaw Bear");
    t.resolve_all();
    assert_eq!(t.counters(ay, "+1/+1"), 2);
    // Second mode: another Bear that was already there fights.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ayula, Queen Among Bears");
    let old = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    answer_modes(&mut t, P0, &[1]);
    t.answer_targets(P0, &[ob(old)]);
    t.answer_targets(P0, &[ob(wurm)]);
    t.enter(P0, "Runeclaw Bear");
    t.resolve_all();
    assert_eq!(damage_on(&t, wurm), 2);
    assert!(!t.on_battlefield(old));
}

#[test]
fn gruul_ragebeast_trigger_is_mandatory_and_removed_without_a_target() {
    cr!("603.3d", "701.14a");
    ruling!("Gruul Ragebeast", "Gruul Ragebeast's triggered ability isn't optional. However, if no opponent controls a creature when a creature enters under your control, the ability will be removed from the stack.");
    supported("Gruul Ragebeast");
    // No opponent creature: nothing goes on the stack.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gruul Ragebeast");
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // With one, the entering creature fights it without being asked.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gruul Ragebeast");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_yes(P0, false);
    let b = t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(damage_on(&t, wurm), 2);
    assert!(!t.on_battlefield(b));
}

#[test]
fn brash_taunter_redirects_all_the_damage_dealt_to_it() {
    cr!("120.3", "701.14a", "702.12b");
    ruling!("Brash Taunter", "A creature can be dealt an amount of damage greater than its toughness. For example, if Brash Taunter is dealt 3 damage, its middle ability deals 3 damage, not 1, to the target opponent.");
    supported("Brash Taunter");
    let mut t = TestGame::new(2);
    let bt = t.battlefield(P0, "Brash Taunter");
    let dm = t.battlefield(P1, "Colossal Dreadmaw");
    t.lands(P0, "Mountain", 3);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, bt, 0, &[ob(dm)]).expect("fight");
    t.resolve_all();
    assert!(t.on_battlefield(bt));
    assert_eq!(damage_on(&t, dm), 1);
    assert_eq!(t.life(P1), 14);
}

/// Primordial Plasm (P0's) at the beginning of P0's combat targets `target`.
fn slurp(t: &mut TestGame, target: ObjectId) {
    supported("Invasion of Muraganda // Primordial Plasm");
    enter_transformed(t, P0, "Invasion of Muraganda // Primordial Plasm");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[ob(target)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
}

#[test]
fn primordial_plasm_removes_characteristic_defining_abilities_and_allows_new_ones() {
    cr!("613.1f", "613.4a", "613.7");
    ruling!("Invasion of Muraganda // Primordial Plasm", "If the creature affected by Primordial Plasm’s ability (slurp) had an ability defining its power and/or toughness, that base value will become 0.");
    ruling!("Invasion of Muraganda // Primordial Plasm", "A creature that loses all abilities because of Primordial Plasm’s ability (slurp) may later gain abilities.");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let nm = t.battlefield(P0, "Nightmare");
    t.g.recompute();
    assert_eq!(t.pt(nm), (4, 4));
    slurp(&mut t, nm);
    assert_eq!(t.pt(nm), (2, 2), "0/0 base, +2/+2");
    assert!(!has_kw(&t, nm, KeywordKind::Flying));
    // A later effect grants an ability again.
    run_from(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::AddKeyword(Keyword::new(KeywordKind::Flying))],
            duration: Duration::EndOfTurn,
        },
        &[ob(nm)],
    );
    assert!(has_kw(&t, nm, KeywordKind::Flying));
}

#[test]
fn gargos_reduces_only_generic_mana_and_only_from_the_battlefield() {
    cr!("601.2f", "118.7d");
    ruling!(
        "Gargos, Vicious Watcher",
        "Gargos's cost-reduction ability affects only generic mana in a Hydra's cost."
    );
    ruling!("Gargos, Vicious Watcher", "Gargos's cost-reduction ability applies only while it's on the battlefield. Gargos doesn't reduce its own cost.");
    supported("Gargos, Vicious Watcher");
    supported("Bristling Hydra");
    // Bristling Hydra ({2}{G}{G}) costs {G}{G}, not less.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gargos, Vicious Watcher");
    t.lands(P0, "Forest", 1);
    let h = t.hand(P0, "Bristling Hydra");
    assert!(t.cast(P0, h).try_go().is_err(), "{{G}} isn't enough");
    t.lands(P0, "Forest", 1);
    t.cast(P0, h).go();
    // In hand, Gargos reduces neither a Hydra nor itself.
    let mut t = TestGame::new(2);
    t.hand(P0, "Gargos, Vicious Watcher");
    t.lands(P0, "Forest", 2);
    let h = t.hand(P0, "Bristling Hydra");
    assert!(t.cast(P0, h).try_go().is_err());
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let g = t.hand(P0, "Gargos, Vicious Watcher");
    assert!(t.cast(P0, g).try_go().is_err());
}

#[test]
fn gargos_x_is_chosen_before_the_reduction() {
    cr!("601.2b", "601.2f", "107.3");
    ruling!("Gargos, Vicious Watcher", "If a spell has {X} in its mana cost, you choose the value for X before applying cost reductions.");
    supported("Hydroid Krasis");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gargos, Vicious Watcher");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    let h = t.hand(P0, "Hydroid Krasis");
    t.cast(P0, h).x(6).go();
    t.resolve_all();
    let k = t.named_on_battlefield("Hydroid Krasis")[0];
    assert_eq!(t.counters(k, "+1/+1"), 6);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn gargos_trigger_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5", "701.14a");
    ruling!("Gargos, Vicious Watcher", "Gargos's triggered ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered.");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Gargos, Vicious Watcher");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    lands_for_cost(&mut t, P0, "Giant Growth");
    let gg = t.hand(P0, "Giant Growth");
    t.answer_targets(P0, &[ob(bears)]);
    t.answer_targets(P0, &[ob(giant)]);
    let s = t.cast(P0, gg).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    lands_for_cost(&mut t, P1, "Counterspell");
    let cs = t.hand(P1, "Counterspell");
    t.answer_targets(P1, &[ob(s)]);
    t.cast(P1, cs).go();
    t.resolve(); // Counterspell
    assert!(!resolved(&t, s));
    t.resolve_all();
    assert!(!t.on_battlefield(giant), "Gargos fought the Giant");
    assert_eq!(damage_on(&t, g), 3);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn gargos_triggers_once_per_creature_targeted() {
    cr!("603.2", "115.3");
    ruling!("Gargos, Vicious Watcher", "If a spell targets one creature you control multiple times, Gargos's ability triggers once. If that spell targets multiple creatures you control, Gargos's ability triggers once for each of those creatures.");
    supported("Seeds of Strength");
    for spread in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Gargos, Vicious Watcher");
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Runeclaw Bear");
        let c = t.battlefield(P0, "Hill Giant");
        let tg = if spread { [a, b, c] } else { [a, a, a] };
        cast_modes(
            &mut t,
            P0,
            "Seeds of Strength",
            &[],
            &[ob(tg[0]), ob(tg[1]), ob(tg[2])],
        );
        t.settle();
        assert_eq!(t.stack_len(), if spread { 4 } else { 2 }, "spread {spread}");
    }
}

#[test]
fn gimli_counts_creatures_that_died_before_it_was_on_the_battlefield() {
    cr!("700.4", "611.3a");
    ruling!("Gimli, Mournful Avenger", "Gimli, Mournful Avenger doesn't need to have been on the battlefield when creatures died under your control for the first ability to count them.");
    supported("Gimli, Mournful Avenger");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, a);
    let gimli = t.battlefield(P0, "Gimli, Mournful Avenger");
    t.g.recompute();
    assert!(
        !has_kw(&t, gimli, KeywordKind::Indestructible),
        "only one died"
    );
    let mut t = TestGame::new(2);
    let a2 = t.battlefield(P0, "Grizzly Bears");
    let b2 = t.battlefield(P0, "Runeclaw Bear");
    destroy(&mut t, a2);
    destroy(&mut t, b2);
    let gimli = t.battlefield(P0, "Gimli, Mournful Avenger");
    t.g.recompute();
    assert!(has_kw(&t, gimli, KeywordKind::Indestructible));
}

#[test]
fn cherished_hatchling_flash_dinosaurs_and_their_fight_trigger() {
    cr!("702.8a", "608.2b", "701.14b");
    ruling!("Cherished Hatchling", "During the turn Cherished Hatchling dies, you may cast any number of Dinosaurs as though they had flash.");
    ruling!("Cherished Hatchling", "For the triggered ability that the entering Dinosaur gains, if the target is illegal when it tries to resolve or if the Dinosaur that entered the battlefield has left the battlefield, no creature will deal or be dealt damage.");
    supported("Cherished Hatchling");
    // 0: baseline; 1: target gains hexproof; 2: the Dinosaur leaves.
    for case in 0..3 {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        let h = t.battlefield(P0, "Cherished Hatchling");
        let wurm = t.battlefield(P1, "Craw Wurm");
        t.answer_yes(P0, true);
        destroy(&mut t, h);
        t.resolve_all();
        // Dinosaurs cast during the opponent's turn.
        lands_for_cost(&mut t, P0, "Colossal Dreadmaw");
        let d = t.hand(P0, "Colossal Dreadmaw");
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[ob(wurm)]);
        t.cast(P0, d).go();
        t.settle();
        assert_eq!(t.stack_len(), 2, "case {case}: the cast trigger");
        t.resolve();
        t.resolve();
        let dm = t.named_on_battlefield("Colossal Dreadmaw")[0];
        assert_eq!(t.stack_len(), 1, "case {case}: the gained trigger");
        match case {
            1 => spoil(&mut t, wurm, Spoil::Hexproof),
            2 => destroy(&mut t, dm),
            _ => {}
        }
        t.resolve_all();
        if case == 0 {
            assert!(!t.on_battlefield(wurm));
            assert!(!t.on_battlefield(dm), "the Wurm dealt 6 back");
        } else {
            assert_eq!(damage_on(&t, wurm), 0, "case {case}");
            if case == 1 {
                assert_eq!(damage_on(&t, dm), 0);
            }
        }
        // Another Dinosaur at instant speed this turn.
        if case == 0 {
            supported("Territorial Allosaurus");
            lands_for_cost(&mut t, P0, "Territorial Allosaurus");
            let al = t.hand(P0, "Territorial Allosaurus");
            t.cast(P0, al).kicked(false).go();
        }
    }
}

/// P1's Grizzly Bears attacks P0; P0 has Skophos Maze-Warden (3/4) and Labyrinth of
/// Skophos with mana. Returns (warden, labyrinth, bears).
fn labyrinth_scene(t: &mut TestGame) -> (ObjectId, ObjectId, ObjectId) {
    supported("Skophos Maze-Warden");
    let w = t.battlefield(P0, "Skophos Maze-Warden");
    let lab = t.battlefield(P0, "Labyrinth of Skophos");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 4);
    t.set_step(P1, Step::DeclareBlockers);
    t.g.combat
        .as_mut()
        .unwrap()
        .attackers
        .push(mtg_engine::combat::AttackerInfo {
            id: bear,
            target: Some(Entity::Player(P0)),
            original_target: Some(Entity::Player(P0)),
            declared: true,
            blocked: false,
            blockers: vec![],
            band: None,
            defending_player: Some(P0),
        });
    (w, lab, bear)
}

#[test]
fn skophos_maze_warden_trigger_resolves_first_and_needs_both_creatures() {
    cr!("603.3", "701.14b", "608.2b");
    ruling!("Skophos Maze-Warden", "An ability that triggers when a creature becomes the target of an ability resolves before the ability that caused it to trigger. It resolves even if that ability is countered or if the creature is no longer a legal target.");
    ruling!("Skophos Maze-Warden", "If either Skophos Maze-Warden or the creature targeted by Labyrinth of Skophos has left the battlefield, the other won't deal or be dealt damage.");
    // The trigger resolves first: the Bears dies, and the Labyrinth's ability then has no
    // legal target.
    let mut t = TestGame::new(2);
    let (w, lab, bear) = labyrinth_scene(&mut t);
    t.answer_yes(P0, true);
    t.activate(P0, lab, 1, &[ob(bear)]).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert!(!t.on_battlefield(bear));
    assert_eq!(damage_on(&t, w), 2);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    // The Warden leaves before its trigger resolves: no damage.
    let mut t = TestGame::new(2);
    let (w, lab, bear) = labyrinth_scene(&mut t);
    t.answer_yes(P0, true);
    t.activate(P0, lab, 1, &[ob(bear)]).unwrap();
    t.settle();
    destroy(&mut t, w);
    t.resolve_all();
    assert_eq!(damage_on(&t, bear), 0);
    // The creature leaves (bounced): the Warden isn't dealt damage.
    let mut t = TestGame::new(2);
    let (w, lab, bear) = labyrinth_scene(&mut t);
    t.answer_yes(P0, true);
    t.activate(P0, lab, 1, &[ob(bear)]).unwrap();
    t.settle();
    t.g.move_object(bear, Zone::Hand(P1), events::MoveCause::Effect, None);
    t.resolve_all();
    assert_eq!(damage_on(&t, w), 0);
}
