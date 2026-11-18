//! Rulings batch P135 — fights ("removal-fight"): fight abilities whose target becomes
//! illegal or whose source leaves (CR 608.2b, 701.14b), fight spells that need both
//! targets to be cast (CR 601.2c) or may have just one, Barroom Brawl's copies, Kraul
//! Harpooner, Scab-Clan Giant's random target, Wayta, Gimli's reflexive ability, Temur
//! Charm's "can't block", and the modal fight cards.

use crate::r_p135_common::*;
use crate::r_s07_common::chosen_modes;
use mtg_engine::decision::Decision;
use mtg_engine::object::StackKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Fight abilities: an illegal target, or a source that left the battlefield.
// ---------------------------------------------------------------------------------------

/// Puts a fight ability on the stack targeting P1's `target`; returns the creature that
/// would fight it.
type FightStart = fn(&mut TestGame, ObjectId) -> ObjectId;

fn indrik(t: &mut TestGame, o: ObjectId) -> ObjectId {
    t.answer_targets(P0, &[ob(o)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Affectionate Indrik")
}

fn taunter(t: &mut TestGame, o: ObjectId) -> ObjectId {
    let bt = t.battlefield(P0, "Brash Taunter");
    t.lands(P0, "Mountain", 3);
    // Its damage trigger, if the fight happens, targets P1.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, bt, 0, &[ob(o)]).expect("Brash Taunter");
    bt
}

fn gargos(t: &mut TestGame, o: ObjectId) -> ObjectId {
    let g = t.battlefield(P0, "Gargos, Vicious Watcher");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_modes(t, P0, "Giant Growth", &[], &[ob(bears)]);
    t.answer_targets(P0, &[ob(o)]);
    t.settle();
    g
}

fn brawler(t: &mut TestGame, o: ObjectId) -> ObjectId {
    t.answer_targets(P0, &[ob(o)]);
    t.enter(P0, "Pheres-Band Brawler")
}

fn mammoth(t: &mut TestGame, o: ObjectId) -> ObjectId {
    t.answer_targets(P0, &[ob(o)]);
    t.enter(P0, "Thorn Mammoth")
}

fn stag(t: &mut TestGame, o: ObjectId) -> ObjectId {
    t.answer_targets(P0, &[ob(o)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Somberwald Stag")
}

fn ragebeast(t: &mut TestGame, o: ObjectId) -> ObjectId {
    t.battlefield(P0, "Gruul Ragebeast");
    t.answer_targets(P0, &[ob(o)]);
    t.enter(P0, "Grizzly Bears")
}

fn harpooner(t: &mut TestGame, o: ObjectId) -> ObjectId {
    t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[ob(o)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Kraul Harpooner")
}

fn ravager(t: &mut TestGame, o: ObjectId) -> ObjectId {
    answer_modes(t, P0, &[0]);
    t.answer_targets(P0, &[ob(o)]);
    t.enter(P0, "Ravager Wurm")
}

#[test]
fn fight_abilities_do_nothing_if_the_target_is_illegal_or_the_source_left() {
    cr!("608.2b", "701.14a", "701.14b");
    ruling!("Affectionate Indrik", "If the target creature is an illegal target when Affectionate Indrik's ability tries to resolve, the ability doesn't resolve. If Affectionate Indrik is no longer on the battlefield, the target creature won't deal or be dealt damage.");
    ruling!("Brash Taunter", "If the target creature is an illegal target when Brash Taunter's last ability tries to resolve, the ability doesn't resolve. If Brash Taunter is no longer on the battlefield, the target creature won't deal or be dealt damage.");
    ruling!("Gargos, Vicious Watcher", "If the target creature is an illegal target when Gargos's last ability tries to resolve, the ability doesn't resolve. If Gargos is no longer on the battlefield, the target creature won't deal or be dealt damage.");
    ruling!("Pheres-Band Brawler", "If the target creature is an illegal target when Pheres-Band Brawler’s ability tries to resolve, the ability doesn’t resolve. If Pheres-Band Brawler is no longer on the battlefield, the target creature won’t deal or be dealt damage.");
    ruling!("Thorn Mammoth", "If the target creature is an illegal target when Thorn Mammoth's last ability tries to resolve, the ability doesn't resolve. If Thorn Mammoth is no longer on the battlefield, the target creature won't deal or be dealt damage.");
    ruling!("Somberwald Stag", "If the targeted creature is an illegal target when Somberwald Stag’s ability tries to resolve, the ability doesn’t resolve. If Somberwald Stag is no longer on the battlefield, the targeted creature won’t deal or be dealt damage.");
    ruling!("Gruul Ragebeast", "If the target creature is an illegal target when the ability tries to resolve, it won't resolve and none of its effects will happen. Neither creature will deal or be dealt damage.");
    ruling!("Kraul Harpooner", "If you choose a target and the target creature is an illegal target when Kraul Harpooner's ability tries to resolve, the ability doesn't resolve and Kraul Harpooner doesn't get +X/+0. If the target creature is legal but Kraul Harpooner is no longer on the battlefield, the target creature won't deal or be dealt damage.");
    ruling!("Ravager Wurm", "If the target of Ravager Wurm's first mode isn't a legal target as that ability resolves, or if Ravager Wurm has left the battlefield, neither creature will deal or be dealt damage.");
    let cases: [(&str, FightStart); 9] = [
        ("Affectionate Indrik", indrik),
        ("Brash Taunter", taunter),
        ("Gargos, Vicious Watcher", gargos),
        ("Pheres-Band Brawler", brawler),
        ("Thorn Mammoth", mammoth),
        ("Somberwald Stag", stag),
        ("Gruul Ragebeast", ragebeast),
        ("Kraul Harpooner", harpooner),
        ("Ravager Wurm", ravager),
    ];
    for (name, start) in cases {
        supported(name);
        // 0: both there, they fight; 1: the target gains hexproof; 2: the fighter leaves.
        for case in 0..3 {
            let mut t = TestGame::new(2);
            // A 4/4 flier (Kraul Harpooner's target must fly).
            let o = t.battlefield(P1, "Serra Angel");
            let me = start(&mut t, o);
            t.settle();
            match case {
                1 => spoil(&mut t, o, Spoil::Hexproof),
                // Brash Taunter is indestructible: it's put into the graveyard instead.
                2 => kill(&mut t, me),
                _ => {}
            }
            t.resolve_all();
            match case {
                0 => {
                    assert!(dealt(&t, o) > 0, "{name}: it fought");
                    assert!(dealt(&t, me) > 0, "{name}: it fought back");
                }
                _ => {
                    assert_eq!(dealt(&t, o), 0, "{name} case {case}");
                    assert_eq!(dealt(&t, me), 0, "{name} case {case}");
                }
            }
            if name == "Kraul Harpooner" && case == 1 {
                assert_eq!(
                    t.pt(me),
                    (3, 2),
                    "no +X/+0 when the ability doesn't resolve"
                );
            }
        }
    }
}

#[test]
fn kraul_harpooner_without_a_target_just_gets_the_bonus() {
    cr!("601.2c", "603.3d", "115.4");
    ruling!(
        "Kraul Harpooner",
        "If you don't choose a target creature, Kraul Harpooner simply gets +X/+0 until end of turn."
    );
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Grizzly Bears");
    let angel = t.battlefield(P1, "Serra Angel");
    t.answer_targets(P0, &[]);
    let h = t.enter(P0, "Kraul Harpooner");
    t.resolve_all();
    assert_eq!(t.pt(h), (5, 2));
    assert_eq!(dealt(&t, angel), 0);
}

#[test]
fn ravager_wurm_may_choose_neither_mode() {
    cr!("700.2", "603.3c");
    ruling!(
        "Ravager Wurm",
        "You may choose neither mode for Ravager Wurm's triggered ability. In this case, the ability is removed from the stack with no effect."
    );
    supported("Ravager Wurm");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let land = t.battlefield(P1, "Mutavault");
    answer_modes(&mut t, P0, &[]);
    t.enter(P0, "Ravager Wurm");
    t.resolve_all();
    assert!(t.g.stack.is_empty());
    assert!(t.on_battlefield(bears) && t.on_battlefield(land));
    assert_eq!(dealt(&t, bears), 0);
}

// ---------------------------------------------------------------------------------------
// Two-target fight spells: both targets needed to cast, or just yours.
// ---------------------------------------------------------------------------------------

#[test]
fn two_target_fight_spells_need_both_targets_to_be_cast() {
    cr!("601.2c", "115.1");
    ruling!("Epic Confrontation", "You can't cast Epic Confrontation unless you choose both a creature you control and a creature you don't control as targets.");
    ruling!("Hunt the Weak", "You can't cast Hunt the Weak unless you choose both a creature you control and a creature you don't control as targets.");
    ruling!("Joust", "You can't cast Joust unless you choose both a creature you control and a creature you don't control as targets.");
    ruling!("Savage Punch", "You can’t cast Savage Punch unless you target both a creature you control and a creature you don’t control.");
    ruling!("Savage Smash", "You can’t cast Savage Smash unless you choose both a creature you control and a creature you don’t control as targets.");
    ruling!("Savage Swipe", "You can’t cast Savage Swipe unless you choose both a creature you control and a creature you don’t control as targets.");
    ruling!("Swift Kick", "You can’t cast Swift Kick unless you target both a creature you control and a creature you don’t control.");
    ruling!("Temur Charm", "You can’t cast Temur Charm choosing the first mode unless you target both a creature you control and a creature you don’t control.");
    ruling!("Wild Instincts", "You must target both a creature you control and a creature an opponent controls to cast Wild Instincts.");
    let cases: [(&str, &[usize]); 9] = [
        ("Epic Confrontation", &[]),
        ("Hunt the Weak", &[]),
        ("Joust", &[]),
        ("Savage Punch", &[]),
        ("Savage Smash", &[]),
        ("Savage Swipe", &[]),
        ("Swift Kick", &[]),
        ("Temur Charm", &[0]),
        ("Wild Instincts", &[]),
    ];
    for (name, modes) in cases {
        // Only your creature: can't be cast (Temur Charm: the first mode can't be chosen).
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, "Grizzly Bears");
        let from = t.asked().len();
        let r = try_cast(&mut t, P0, name, modes, &[&[ob(mine)], &[]]);
        if modes.is_empty() {
            assert!(r.is_err(), "{name}: cast with one target");
        } else {
            assert!(!modes_offered(&t, from).0.contains(&0), "{name}");
        }
        // Only the opponent's creature: can't be cast either.
        let mut t = TestGame::new(2);
        let theirs = t.battlefield(P1, "Grizzly Bears");
        let from = t.asked().len();
        let r = try_cast(&mut t, P0, name, modes, &[&[], &[ob(theirs)]]);
        if modes.is_empty() {
            assert!(r.is_err(), "{name}: cast without your creature");
        } else {
            assert!(!modes_offered(&t, from).0.contains(&0), "{name}");
        }
        // Both: it's cast and they fight.
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, "Grizzly Bears");
        let theirs = t.battlefield(P1, "Colossal Dreadmaw");
        try_cast(&mut t, P0, name, modes, &[&[ob(mine)], &[ob(theirs)]]).expect(name);
        t.resolve_all();
        assert!(dealt(&t, theirs) >= 2, "{name}");
    }
}

#[test]
fn up_to_one_fight_spells_can_target_just_your_creature() {
    cr!("601.2c", "115.4", "608.2b");
    ruling!(
        "Bridgeworks Battle // Tanglespan Bridgeworks",
        "You can cast Bridgeworks Battle targeting only the creature you control."
    );
    ruling!(
        "Struggle for Skemfar",
        "You can cast Struggle for Skemfar targeting only a creature you control."
    );
    ruling!("Invasion of Muraganda // Primordial Plasm", "You can choose only a creature you control as a target for Invasion of Muraganda’s enters-the-battlefield ability.");
    // Bridgeworks Battle: +2/+2, no fight.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let name = "Bridgeworks Battle // Tanglespan Bridgeworks";
    try_cast(&mut t, P0, name, &[], &[&[ob(mine)], &[]]).expect("Bridgeworks Battle");
    t.resolve_all();
    assert_eq!(t.pt(mine), (4, 4));
    assert_eq!((dealt(&t, mine), dealt(&t, theirs)), (0, 0));
    // Struggle for Skemfar: a +1/+1 counter, no fight.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    try_cast(&mut t, P0, "Struggle for Skemfar", &[], &[&[ob(mine)], &[]]).expect("Struggle");
    t.resolve_all();
    assert_eq!(t.counters(mine, "+1/+1"), 1);
    assert_eq!((dealt(&t, mine), dealt(&t, theirs)), (0, 0));
    // Invasion of Muraganda: the counter, no fight.
    supported("Invasion of Muraganda // Primordial Plasm");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[ob(mine)]);
    t.answer_targets(P0, &[]);
    t.enter(P0, "Invasion of Muraganda // Primordial Plasm");
    t.resolve_all();
    assert_eq!(t.counters(mine, "+1/+1"), 1);
    assert_eq!((dealt(&t, mine), dealt(&t, theirs)), (0, 0));
}

#[test]
fn two_target_fight_spells_with_either_target_illegal_deal_no_damage() {
    cr!("608.2b", "701.14b");
    ruling!("Bridgeworks Battle // Tanglespan Bridgeworks", "If you choose two target creatures and either target is an illegal target as Bridgeworks Battle tries to resolve, neither creature will deal or be dealt damage.");
    ruling!("Struggle for Skemfar", "If you choose two target creatures and either target is an illegal target as Struggle for Skemfar tries to resolve, neither creature will deal or be dealt damage.");
    for name in [
        "Bridgeworks Battle // Tanglespan Bridgeworks",
        "Struggle for Skemfar",
    ] {
        for bad in [
            Bad::Mine(Spoil::Hexproof),
            Bad::Mine(Spoil::Gone),
            Bad::Theirs(Spoil::Hexproof),
            Bad::Theirs(Spoil::Gone),
        ] {
            let (t, m, o) = fight_scenario(
                "Grizzly Bears",
                "Colossal Dreadmaw",
                |t, m, o| {
                    cast_modes(t, P0, name, &[], &[ob(m), ob(o)]);
                },
                bad,
            );
            assert_eq!((dealt(&t, m), dealt(&t, o)), (0, 0), "{name} {bad:?}");
        }
    }
}

#[test]
fn territorial_allosaurus_may_have_to_fight_your_own_creature() {
    cr!("603.3d", "115.1", "701.14a");
    ruling!("Territorial Allosaurus", "Territorial Allosaurus's ability can target another creature you control (such as a Dinosaur with an enrage ability). If you kicked it but your opponent controls no creatures that are legal targets, the ability must target another one of your creatures. Plan carefully before kicking Dinosaurs.");
    supported("Territorial Allosaurus");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Territorial Allosaurus");
    t.lands(P0, "Wastes", 2);
    t.lands(P0, "Forest", 1);
    let card = t.hand(P0, "Territorial Allosaurus");
    let from = t.asked().len();
    // P0 tries to choose no target: the ability needs one, so it targets the Bears.
    t.answer_targets(P0, &[]);
    t.cast(P0, card).kicked(true).go();
    t.resolve(); // the Allosaurus
    t.settle();
    assert_eq!(t.stack_len(), 1, "the kicked trigger");
    let offered = target_candidates(&t, P0, from);
    assert_eq!(offered.last(), Some(&vec![ob(bears)]));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    let al = t.named_on_battlefield("Territorial Allosaurus")[0];
    assert_eq!(damage_on(&t, al), 2);
}

#[test]
fn ulvenwald_tracker_can_fight_two_of_your_creatures_but_not_one_twice() {
    cr!("115.3", "701.14a");
    ruling!("Ulvenwald Tracker", "The second target of Ulvenwald Tracker’s ability can be another creature you control, but it can’t be the same creature as the first target.");
    supported("Ulvenwald Tracker");
    let mut t = TestGame::new(2);
    let tr = t.battlefield(P0, "Ulvenwald Tracker");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Forest", 2);
    // Answering the same creature twice isn't accepted: the second target is another
    // one, or the activation is illegal.
    match t.activate(P0, tr, 0, &[ob(bears), ob(bears)]) {
        Ok(Some(ab)) => {
            let chosen = crate::r_s25_common::targets_of(&t, ab);
            assert_eq!(chosen.len(), 2);
            assert_ne!(chosen[0], chosen[1]);
        }
        Ok(None) => panic!("no ability"),
        Err(_) => {}
    }
    let mut t = TestGame::new(2);
    let tr = t.battlefield(P0, "Ulvenwald Tracker");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Forest", 2);
    t.activate(P0, tr, 0, &[ob(giant), ob(bears)])
        .expect("two of yours");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(damage_on(&t, giant), 2);
}

// ---------------------------------------------------------------------------------------
// Barroom Brawl's copy that keeps its targets.
// ---------------------------------------------------------------------------------------

#[test]
fn barroom_brawl_copy_with_unchanged_targets_fizzles_and_isnt_copied_again() {
    cr!("707.10c", "608.2b", "115.1");
    ruling!("Barroom Brawl", "If the opponent to your left chooses to copy Barroom Brawl but not change its targets, both targets will probably be illegal as the copy tries to resolve (at a minimum, both targets would have to survive and switch controllers in response to still be legal targets for the copy), and the player to their left will not be able to copy it.");
    supported("Barroom Brawl");
    let mut t = TestGame::new(3);
    // Neither creature dies in the fight.
    let orni = t.battlefield(P0, "Ornithopter");
    let wall = t.battlefield(P1, "Wall of Wood");
    t.battlefield(P1, "Hill Giant");
    t.battlefield(P2, "Grizzly Bears");
    // P1 copies it and keeps the targets; P2 would copy too if asked.
    t.answer_yes(P1, true);
    t.answer_yes(P1, false);
    t.answer_yes(P2, true);
    cast_modes(&mut t, P0, "Barroom Brawl", &[], &[ob(orni), ob(wall)]);
    let from = t.asked().len();
    t.resolve_all();
    assert!(t.g.stack.is_empty());
    assert!(
        t.asked()[from..].iter().all(|(p, _)| *p != P2),
        "P2 is never offered a copy"
    );
    // Only the original's fight happened (Ornithopter deals 0, the Wall deals 0).
    assert!(t.on_battlefield(orni) && t.on_battlefield(wall));
}

// ---------------------------------------------------------------------------------------
// Modal fight cards.
// ---------------------------------------------------------------------------------------

#[test]
fn inscription_of_abundance_negative_power_gains_nothing() {
    cr!("107.1b", "119.3");
    ruling!("Inscription of Abundance", "If the second mode is chosen and each creature the target player controls has negative power, that player doesn't gain or lose life.");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Grizzly Bears");
    let g = t.battlefield(P0, "Hill Giant");
    modify(
        &mut t,
        P1,
        b,
        vec![mtg_engine::ability::Modification::ModifyPT(
            mtg_engine::ability::Value::Const(-4),
            mtg_engine::ability::Value::Const(0),
        )],
    );
    modify(
        &mut t,
        P1,
        g,
        vec![mtg_engine::ability::Modification::ModifyPT(
            mtg_engine::ability::Value::Const(-4),
            mtg_engine::ability::Value::Const(0),
        )],
    );
    assert_eq!(t.pt(b).0, -2);
    cast_modes(
        &mut t,
        P0,
        "Inscription of Abundance",
        &[1],
        &[Entity::Player(P0)],
    );
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

/// The last mode choice offered since decision `from`: (available modes, repeats allowed).
fn modes_offered(t: &TestGame, from: usize) -> (Vec<usize>, bool) {
    t.asked()[from..]
        .iter()
        .rev()
        .find_map(|(_, d)| match d {
            Decision::ChooseModes {
                available,
                allow_repeat,
                ..
            } => Some((available.clone(), *allow_repeat)),
            _ => None,
        })
        .expect("a mode choice")
}

#[test]
fn modal_spells_choose_each_mode_at_most_once() {
    cr!("700.2", "700.2d", "601.2b");
    ruling!(
        "Inscription of Abundance",
        "If you kick Inscription of Abundance, you can't choose any one mode more than once."
    );
    ruling!(
        "Casualties of War",
        "You can choose any number of the modes for Casualties of War, but you can't choose a mode more than once."
    );
    ruling!("Rain of Thorns", "You can choose just one mode, any two of the modes, or all three. You make this choice as you cast Rain of Thorns.");
    // Inscription kicked: the same mode twice isn't a legal choice (the answer is
    // rejected and a single mode is used); two different modes are fine.
    supported("Inscription of Abundance");
    for (modes, want) in [(vec![0, 0], vec![0]), (vec![0, 1], vec![0, 1])] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        lands_for_cost(&mut t, P0, "Inscription of Abundance");
        t.lands(P0, "Forest", 3);
        let card = t.hand(P0, "Inscription of Abundance");
        t.answer_targets(P0, &[ob(bears)]);
        t.answer_targets(P0, &[Entity::Player(P0)]);
        let from = t.asked().len();
        let s = t.cast(P0, card).kicked(true).modes(&modes).go();
        assert!(!modes_offered(&t, from).1, "no repeats");
        assert_eq!(chosen_modes(&t, s), want);
        t.resolve_all();
        assert_eq!(t.counters(bears, "+1/+1"), 2);
    }
    // Casualties of War: the same.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    lands_for_cost(&mut t, P0, "Casualties of War");
    let card = t.hand(P0, "Casualties of War");
    t.answer_targets(P0, &[ob(bears)]);
    t.answer_targets(P0, &[ob(giant)]);
    let from = t.asked().len();
    let s = t.cast(P0, card).modes(&[1, 1]).go();
    assert!(!modes_offered(&t, from).1);
    assert_eq!(chosen_modes(&t, s).len(), 1);
    t.resolve_all();
    assert!(t.on_battlefield(bears) != t.on_battlefield(giant));
    // Rain of Thorns: one, two, or all three modes, chosen as it's cast.
    for modes in [vec![2], vec![0, 2], vec![0, 1, 2]] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P1, "Sol Ring");
        let e = t.battlefield(P1, "Glorious Anthem");
        let l = t.battlefield(P1, "Mutavault");
        let all = [ob(a), ob(e), ob(l)];
        let targets: Vec<Entity> = modes.iter().map(|m| all[*m]).collect();
        let s = cast_modes(&mut t, P0, "Rain of Thorns", &modes, &targets);
        assert_eq!(chosen_modes(&t, s), modes);
        t.resolve_all();
        for (i, id) in [a, e, l].into_iter().enumerate() {
            assert_eq!(t.on_battlefield(id), !modes.contains(&i), "{modes:?} {i}");
        }
    }
}

#[test]
fn fissure_vent_modes_need_legal_targets() {
    cr!("700.2a", "601.2c");
    ruling!("Fissure Vent", "You may choose just the first mode (targeting an artifact), just the second mode (targeting a nonbasic land), or both modes (targeting an artifact and a nonbasic land). You can’t choose a mode unless there’s a legal target for it.");
    // Each mode alone, and both.
    for modes in [vec![0], vec![1], vec![0, 1]] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P1, "Sol Ring");
        let l = t.battlefield(P1, "Mutavault");
        let all = [ob(a), ob(l)];
        let targets: Vec<Entity> = modes.iter().map(|m| all[*m]).collect();
        cast_modes(&mut t, P0, "Fissure Vent", &modes, &targets);
        t.resolve_all();
        assert_eq!(t.on_battlefield(a), !modes.contains(&0), "{modes:?}");
        assert_eq!(t.on_battlefield(l), !modes.contains(&1), "{modes:?}");
    }
    // No nonbasic land (only a basic one): the second mode isn't available.
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P1, "Sol Ring");
    let f = t.battlefield(P1, "Forest");
    lands_for_cost(&mut t, P0, "Fissure Vent");
    let card = t.hand(P0, "Fissure Vent");
    t.answer_targets(P0, &[ob(f)]);
    let from = t.asked().len();
    let s = t.cast(P0, card).modes(&[0, 1]).go();
    assert_eq!(modes_offered(&t, from).0, vec![0]);
    assert_eq!(chosen_modes(&t, s), vec![0]);
    t.resolve_all();
    assert!(!t.on_battlefield(ring));
    assert!(t.on_battlefield(f));
    // No artifact and no nonbasic land: it can't be cast at all.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Forest");
    assert!(try_cast(&mut t, P0, "Fissure Vent", &[1], &[&[]]).is_err());
}

#[test]
fn temur_charm_third_mode_affects_creatures_by_their_power_as_blockers_are_declared() {
    cr!("611.2c", "509.1b");
    ruling!("Temur Charm", "If the third mode was chosen, creatures with power 3 or less that enter the battlefield after Temur Charm resolves also can’t block. A creature whose power was 4 or greater when Temur Charm resolved but is 3 or less as the declare blockers step begins also can’t block.");
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, "Grizzly Bears");
    let dreadmaw = t.battlefield(P1, "Colossal Dreadmaw");
    let big = t.battlefield(P1, "Craw Wurm");
    cast_modes(&mut t, P0, "Temur Charm", &[2], &[]);
    t.resolve_all();
    // After it resolves: a new 2/2, and the 6/4 Craw Wurm shrinks to 2/4.
    let late = t.battlefield(P1, "Grizzly Bears");
    modify(
        &mut t,
        P0,
        big,
        vec![mtg_engine::ability::Modification::ModifyPT(
            mtg_engine::ability::Value::Const(-4),
            mtg_engine::ability::Value::Const(0),
        )],
    );
    attack_with(&mut t, &[(attacker, Entity::Player(P1))]);
    t.set_step(P0, Step::DeclareBlockers);
    t.g.recompute();
    assert!(!t.g.can_block_at_all(t.g.current(late)), "entered later");
    assert!(!t.g.can_block_at_all(t.g.current(big)), "shrank later");
    assert!(t.g.can_block_at_all(t.g.current(dreadmaw)), "power 6");
}

// ---------------------------------------------------------------------------------------
// Triggered abilities and their sources.
// ---------------------------------------------------------------------------------------

#[test]
fn brash_taunter_controller_loses_before_its_trigger_goes_on_the_stack() {
    cr!("704.5a", "704.3", "800.4a", "603.3");
    ruling!("Brash Taunter", "If your life total is brought to 0 or less at the same time that Brash Taunter is dealt damage, you lose the game before its middle ability goes on the stack.");
    supported("Brash Taunter");
    supported("Earthquake");
    // Three players: the game goes on after P0 loses.
    let mut t = TestGame::new(3);
    t.set_step(P2, Step::PrecombatMain);
    t.battlefield(P0, "Brash Taunter");
    t.g.players[0].life = 2;
    t.answer_targets(P0, &[Entity::Player(P1)]);
    lands_for_cost(&mut t, P2, "Earthquake");
    t.lands(P2, "Wastes", 2);
    let eq = t.hand(P2, "Earthquake");
    t.cast(P2, eq).x(2).go();
    t.resolve_all();
    assert!(t.has_lost(P0));
    // Earthquake dealt 2 to P1; Brash Taunter's ability never dealt its 2.
    assert_eq!(t.life(P1), 18);
}

#[test]
fn scab_clan_giant_chooses_a_random_target_among_all_opponents_creatures() {
    cr!("115.1", "603.3d", "705.1");
    ruling!("Scab-Clan Giant", "In a multiplayer game, you don't choose a single opponent at random. Any creature controlled by any of your opponents may be chosen at random.");
    ruling!("Scab-Clan Giant", "The target is chosen at random as part of putting the ability on the stack. Players can respond to the ability knowing what the target of the ability is.");
    ruling!("Scab-Clan Giant", "To choose a target at random, all legal targets must have an equal chance of being chosen. There are many ways to do this, including assigning each possible legal target a number and rolling a die.");
    supported("Scab-Clan Giant");
    // P1 has one creature, P2 has two: each of the three is chosen about a third of the
    // time (picking an opponent first would give P1's creature half).
    let mut hits = [0u32; 3];
    let trials = 150;
    for seed in 0..trials {
        let mut t = TestGame::with_config(
            3,
            mtg_engine::game::GameConfig {
                seed,
                ..Default::default()
            },
        );
        let a = t.battlefield(P1, "Colossal Dreadmaw");
        let b = t.battlefield(P2, "Craw Wurm");
        let c = t.battlefield(P2, "Serra Angel");
        t.battlefield(P0, "Wall of Wood");
        t.enter(P0, "Scab-Clan Giant");
        t.settle();
        // The target is on the stack before anyone gets priority.
        assert_eq!(t.stack_len(), 1);
        let top = *t.g.stack.last().unwrap();
        let chosen = crate::r_s25_common::targets_of(&t, top);
        assert_eq!(chosen.len(), 1);
        let i = [ob(a), ob(b), ob(c)]
            .iter()
            .position(|e| *e == chosen[0])
            .expect("an opponent's creature");
        hits[i] += 1;
        if seed == 0 {
            // A response that knows the target: it gains hexproof, the fight is off.
            let target = match chosen[0] {
                Entity::Object(id) => id,
                _ => unreachable!(),
            };
            spoil(&mut t, target, Spoil::Hexproof);
            t.resolve_all();
            assert_eq!(dealt(&t, target), 0);
        }
    }
    for h in hits {
        assert!(h >= 30 && h <= 70, "uneven random choice: {hits:?}");
    }
}

/// Ripjaw Raptor's enrage ability triggers this many times when it's dealt damage.
fn enrage_triggers(waytas: usize) -> usize {
    let mut t = TestGame::new(2);
    if waytas > 1 {
        // "Somehow": the legend rule doesn't apply.
        t.battlefield(P0, "Mirror Gallery");
    }
    for _ in 0..waytas {
        t.battlefield(P0, "Wayta, Trainer Prodigy");
    }
    let raptor = t.battlefield(P0, "Ripjaw Raptor");
    let hand = t.hand_size(P0);
    crate::r_s06_common::damage(&mut t, raptor, 1, raptor);
    t.resolve_all();
    t.hand_size(P0) - hand
}

#[test]
fn wayta_adds_one_trigger_per_copy() {
    cr!("603.2d", "603.2");
    ruling!("Wayta, Trainer Prodigy", "If you somehow control two copies of Wayta, Trainer Prodigy, a creature you control being dealt damage causes abilities to trigger three times, not four. A third Wayta, Trainer Prodigy causes abilities to trigger four times, a fourth causes abilities to trigger five times, and so on.");
    supported("Wayta, Trainer Prodigy");
    supported("Mirror Gallery");
    supported("Ripjaw Raptor");
    assert_eq!(enrage_triggers(0), 1);
    assert_eq!(enrage_triggers(1), 2);
    assert_eq!(enrage_triggers(2), 3);
    assert_eq!(enrage_triggers(3), 4);
}

#[test]
fn wayta_extra_triggers_choose_targets_separately() {
    cr!("603.2d", "603.3d");
    ruling!("Wayta, Trainer Prodigy", "Wayta, Trainer Prodigy's last ability doesn't copy the triggered ability; it just causes the ability to trigger twice. Any choices made as you put the ability onto the stack, such as modes and targets, are made separately for each instance of the ability. Any choices made on resolution, such as whether to put counters on a permanent, are also made individually.");
    supported("Frilled Deathspitter");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Wayta, Trainer Prodigy");
    let fd = t.battlefield(P0, "Frilled Deathspitter");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P2)]);
    crate::r_s06_common::damage(&mut t, fd, 1, fd);
    t.settle();
    assert_eq!(t.stack_len(), 2, "two instances, not a copy");
    assert!(crate::r_s25_common::spell_copies(&t).is_empty());
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (18, 18));
}

#[test]
fn gimli_third_resolution_triggers_a_reflexive_fight() {
    cr!("603.12", "603.3", "701.14a");
    ruling!("Gimli, Mournful Avenger", "You don't choose a target for Gimli, Mournful Avenger's last ability at the time it triggers. Rather, a second \"reflexive\" ability triggers when its last ability resolves for the third time in a turn. You may choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal.");
    supported("Gimli, Mournful Avenger");
    let mut t = TestGame::new(2);
    let gimli = t.battlefield(P0, "Gimli, Mournful Avenger");
    let giant = t.battlefield(P1, "Hill Giant");
    let from = t.asked().len();
    for name in ["Grizzly Bears", "Runeclaw Bear"] {
        let c = t.battlefield(P0, name);
        destroy(&mut t, c);
        assert_eq!(t.stack_len(), 1);
        t.resolve_all();
    }
    assert!(target_candidates(&t, P0, from).is_empty(), "no target yet");
    let c = t.battlefield(P0, "Wall of Wood");
    destroy(&mut t, c);
    assert_eq!(t.stack_len(), 1);
    let top = *t.g.stack.last().unwrap();
    assert!(crate::r_s25_common::targets_of(&t, top).is_empty());
    t.answer_targets(P0, &[ob(giant)]);
    t.resolve();
    t.settle();
    // The reflexive ability is waiting on the stack with its target; nothing happened yet.
    assert_eq!(t.stack_len(), 1);
    let top = *t.g.stack.last().unwrap();
    assert!(matches!(
        t.g.obj(top).stack.as_deref().map(|s| &s.kind),
        Some(StackKind::Triggered { .. })
    ));
    assert_eq!(crate::r_s25_common::targets_of(&t, top), vec![ob(giant)]);
    assert_eq!(dealt(&t, giant), 0);
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.counters(gimli, "+1/+1"), 3);
}

#[test]
fn cherished_hatchling_dinosaurs_still_cost_their_costs() {
    cr!("702.8a", "118.9", "601.2f");
    ruling!("Cherished Hatchling", "You must pay the costs for spells you cast this way. If there's an alternative cost you can pay instead of the mana cost for a Dinosaur spell, you may pay that cost instead.");
    supported("Cherished Hatchling");
    supported("Omniscience");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let h = t.battlefield(P0, "Cherished Hatchling");
    destroy(&mut t, h);
    t.resolve_all();
    // No mana: the Dinosaur can't be cast even with flash.
    let d = t.hand(P0, "Colossal Dreadmaw");
    t.answer_yes(P0, false);
    assert!(t.cast(P0, d).try_go().is_err());
    t.clear_answers();
    // Omniscience's alternative cost (without paying its mana cost) can be used.
    t.battlefield(P0, "Omniscience");
    t.answer_yes(P0, false);
    let methods = crate::r_s08_common::legal_cast_methods(&mut t, P0, d);
    assert!(!methods.is_empty(), "castable for its alternative cost");
    let d = t.g.current(d);
    t.cast(P0, d).method(methods[0].clone()).go();
    t.resolve_all();
    assert_eq!(count_named(&t, "Colossal Dreadmaw"), 1);
}

#[test]
fn gargos_reduction_applies_after_cost_increases_and_keeps_mana_value() {
    cr!("601.2f", "202.3");
    ruling!("Gargos, Vicious Watcher", "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Gargos). The mana value of the spell remains unchanged, no matter what the total cost to cast it was.");
    supported("Lodestone Golem");
    supported("Bristling Hydra");
    // Bristling Hydra {2}{G}{G}: +{1} from Lodestone Golem, then -{4}: {G}{G}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gargos, Vicious Watcher");
    t.battlefield(P1, "Lodestone Golem");
    t.lands(P0, "Forest", 2);
    let h = t.hand(P0, "Bristling Hydra");
    let s = t.cast(P0, h).go();
    assert_eq!(crate::r_s08_common::mana_value(&t, s), 4);
}
