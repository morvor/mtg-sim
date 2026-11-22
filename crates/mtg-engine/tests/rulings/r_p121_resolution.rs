//! Rulings batch P121 — "+1/+1 counters matter" effects whose values or affected sets
//! are fixed as they resolve (CR 608.2h, 611.2c), abilities that use last known
//! information (CR 603.10a, 608.2h), state-based actions waiting for a spell to finish
//! resolving (CR 704.3), and counts of all the counters on a permanent.

use crate::r_p121_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 casts The Crowd Goes Wild with X = 0 (nobody assists) and resolves it.
fn crowd_x0(t: &mut TestGame) {
    lands_for_cost(t, P0, "The Crowd Goes Wild");
    let card = t.hand(P0, "The Crowd Goes Wild");
    t.answer_yes(P1, false);
    t.cast(P0, card).x(0).go();
    t.resolve_all();
}

#[test]
fn the_crowd_goes_wild_affects_only_creatures_with_counters_as_it_resolves() {
    cr!("611.2c", "107.3");
    ruling!(
        "The Crowd Goes Wild",
        "The final ability of The Crowd Goes Wild affects only creatures with a +1/+1 counter on them at the time it’s applied. Creatures that get a +1/+1 counter later in the turn won’t gain trample, and creatures that lose their +1/+1 counters later in the turn won’t lose trample."
    );
    ruling!(
        "The Crowd Goes Wild",
        "You can cast The Crowd Goes Wild with 0 as the value of X. The spell won’t target any creatures, and creatures with +1/+1 counters on them will gain trample."
    );
    supported("The Crowd Goes Wild");
    let mut t = TestGame::new(2);
    let with = with_counters(&mut t, P0, "Grizzly Bears", 1);
    let theirs = with_counters(&mut t, P1, "Grizzly Bears", 1);
    let without = t.battlefield(P0, "Hill Giant");
    crowd_x0(&mut t);
    assert!(has_kw(&mut t, with, KeywordKind::Trample));
    assert!(has_kw(&mut t, theirs, KeywordKind::Trample));
    assert!(!has_kw(&mut t, without, KeywordKind::Trample));
    give_plus1(&mut t, without, 1);
    remove_plus1(&mut t, with, 1);
    assert!(!has_kw(&mut t, without, KeywordKind::Trample));
    assert!(has_kw(&mut t, with, KeywordKind::Trample));
}

#[test]
fn oran_rief_ooze_determines_the_creatures_as_its_ability_resolves() {
    cr!("608.2c", "611.2c");
    ruling!(
        "Oran-Rief Ooze",
        "The group of creatures that receive a +1/+1 counter from Oran-Rief Ooze's second ability is determined only as the ability resolves. Other abilities that trigger on attacking (such as that of Paired Tactician) may add counters to creatures before that ability resolves."
    );
    supported("Oran-Rief Ooze");
    let mut t = TestGame::new(2);
    let ooze = t.battlefield(P0, "Oran-Rief Ooze");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = with_counters(&mut t, P0, "Grizzly Bears", 1);
    attack_with(
        &mut t,
        &[
            (ooze, Entity::Player(P1)),
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
        ],
    );
    assert_eq!(stack(&t), 1);
    // In response, a counter is put on the first Bears and removed from the second.
    give_plus1(&mut t, a, 1);
    remove_plus1(&mut t, b, 1);
    t.resolve_all();
    assert_eq!((plus1(&t, a), plus1(&t, b), plus1(&t, ooze)), (2, 0, 0));
}

#[test]
fn foundry_hornet_affects_the_opponents_creatures_as_it_resolves() {
    cr!("611.2c", "603.4");
    ruling!(
        "Foundry Hornet",
        "The set of creatures affected by Foundry Hornet’s triggered ability is determined as the ability resolves. Creatures opponents begin to control later in the turn and permanents opponents control that become creatures later in the turn won’t get -1/-1. The effect continues until end of turn even if you stop controlling a creature with a +1/+1 counter on it."
    );
    supported("Foundry Hornet");
    let mut t = TestGame::new(2);
    let mine = with_counters(&mut t, P0, "Grizzly Bears", 1);
    let theirs = t.battlefield(P1, "Hill Giant");
    let giant = t.battlefield(P0, "Hill Giant");
    t.enter(P0, "Foundry Hornet");
    t.resolve_all();
    assert_eq!(t.pt(theirs), (2, 2));
    // P1 gains control of a Hill Giant, and another one enters under P1's control.
    crate::r_p117_common::gain_control(&mut t, P1, giant);
    let later = t.enter(P1, "Hill Giant");
    t.resolve_all();
    assert_eq!(t.g.obj(t.g.current(giant)).controller, P1);
    assert_eq!(t.pt(giant), (3, 3));
    assert_eq!(t.pt(later), (3, 3));
    // P0 stops controlling a creature with a counter: the effect continues.
    destroy(&mut t, mine);
    assert_eq!(t.pt(theirs), (2, 2));
}

#[test]
fn the_lord_master_of_hell_deals_the_same_x_to_each_opponent() {
    cr!("608.2h", "107.3a");
    ruling!(
        "The Emperor of Palamecia // The Lord Master of Hell",
        "The value of X is calculated only once, as The Lord Master of Hell's ability resolves."
    );
    supported("The Emperor of Palamecia // The Lord Master of Hell");
    let mut t = crate::r_p177_common::ffa(3);
    let lord = enter_with(
        &mut t,
        P0,
        "The Emperor of Palamecia // The Lord Master of Hell",
        0,
        true,
    );
    t.resolve_all();
    let lord = t.g.current(lord);
    t.g.objects[lord.0 as usize].summoning_sick = false;
    t.graveyard(P0, "Opt");
    t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P0, "Grizzly Bears");
    attack_with(&mut t, &[(lord, Entity::Player(P1))]);
    assert_eq!(stack(&t), 1);
    // In response, another instant is put into P0's graveyard; X is counted once, on
    // resolution.
    t.graveyard(P0, "Shock");
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (17, 17));
}

#[test]
fn prairie_dog_activated_twice_adds_two() {
    cr!("614.1a", "616.1");
    ruling!(
        "Prairie Dog",
        "You can activate Prairie Dog’s last ability multiple times in a single turn. If it resolves twice, the number of +1/+1 counters put on a creature you control is two plus the original number. Three activations and resolutions add three, and so on."
    );
    supported("Prairie Dog");
    let mut t = TestGame::new(2);
    let dog = t.battlefield(P0, "Prairie Dog");
    t.lands(P0, "Plains", 10);
    let bears = t.battlefield(P0, "Grizzly Bears");
    for _ in 0..2 {
        t.activate(P0, dog, 0, &[]).expect("activate");
        t.resolve_all();
    }
    give_plus1(&mut t, bears, 1);
    assert_eq!(plus1(&t, bears), 3);
}

#[test]
fn lifecrafters_gift_counters_annihilate_only_after_it_resolves() {
    cr!("704.3", "704.5q", "608.2c");
    ruling!(
        "Lifecrafter's Gift",
        "The state-based action that removes matching +1/+1 and -1/-1 counters won't check until after Lifecrafter's Gift finishes resolving. It's possible put a +1/+1 counter on a creature with a -1/-1 counter on it and then give it a second +1/+1 counter."
    );
    supported("Lifecrafter's Gift");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    give_minus1(&mut t, giant, 1);
    cast_resolve(&mut t, P0, "Lifecrafter's Gift", &[obj(giant)]);
    assert_eq!((plus1(&t, giant), t.counters(giant, MINUS1)), (1, 0));
    assert_eq!(t.pt(giant), (4, 4));
}

#[test]
fn the_flesh_is_weak_kills_a_damaged_creature_before_its_counter() {
    cr!("704.5g", "704.3", "613.4c");
    ruling!(
        "The Flesh Is Weak",
        "The same happens if a nonartifact creature you control with no +1/+1 counters on it has damage marked on it equal to its toughness minus 1. For example, if a 5/5 creature was dealt 4 damage earlier in the turn and its controller resolves The Flesh Is Weak, that creature will die before the +1/+1 counter is placed on it."
    );
    supported("The Flesh Is Weak");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hill Giant");
    let giant = t.named_on_battlefield("Hill Giant")[0];
    damage(&mut t, giant, 2);
    cast_new(&mut t, P0, "The Flesh Is Weak", &[]);
    t.resolve();
    // The enchantment is on the battlefield; its enters trigger hasn't resolved yet.
    assert!(t.in_graveyard(P0, "Hill Giant"));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn avatar_of_the_resolute_doesnt_count_creatures_entering_with_it() {
    cr!("614.12", "614.1c");
    ruling!(
        "Avatar of the Resolute",
        "When determining how many +1/+1 counters Avatar of the Resolute enters the battlefield with, don't count any creatures entering the battlefield at the same time, even if those creatures enter the battlefield with +1/+1 counters on them."
    );
    supported("Avatar of the Resolute");
    let mut t = TestGame::new(2);
    with_counters(&mut t, P0, "Grizzly Bears", 1);
    // Swarm Shambler enters with a +1/+1 counter on it.
    let ids = enter_together(
        &mut t,
        &[(P0, "Avatar of the Resolute"), (P0, "Swarm Shambler")],
    );
    assert_eq!(plus1(&t, ids[1]), 1);
    assert_eq!(plus1(&t, ids[0]), 1);
}

#[test]
fn thought_gorger_draws_for_all_the_counters_it_had() {
    cr!("603.10a", "608.2h");
    ruling!(
        "Thought Gorger",
        "When Thought Gorger's second ability resolves, its last existence on the battlefield is checked to see how many +1/+1 counters it had on it. All of them are counted, not just the ones put on it by its \"enters\" ability."
    );
    supported("Thought Gorger");
    let mut t = TestGame::new(2);
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Grizzly Bears");
    let gorger = t.enter(P0, "Thought Gorger");
    t.resolve_all();
    assert_eq!(plus1(&t, gorger), 2);
    assert_eq!(t.hand_size(P0), 0);
    give_plus1(&mut t, gorger, 3);
    exile_now(&mut t, gorger);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 5);
}

#[test]
fn dies_abilities_use_last_known_counters_and_power() {
    cr!("603.10a", "608.2h");
    ruling!(
        "Hooded Hydra",
        "Use the number of +1/+1 counters that were on Hooded Hydra the last time it was on the battlefield to determine how many Snake tokens to create."
    );
    ruling!(
        "Conclave Mentor",
        "Use Conclave Mentor's power as it last existed on the battlefield to determine how much life you gain."
    );
    supported("Hooded Hydra");
    supported("Conclave Mentor");
    let mut t = TestGame::new(2);
    let hydra = with_counters(&mut t, P0, "Hooded Hydra", 3);
    destroy(&mut t, hydra);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Snake"), 3);

    let mut t = TestGame::new(2);
    let mentor = t.battlefield(P0, "Conclave Mentor");
    cast_resolve(&mut t, P0, "Giant Growth", &[obj(mentor)]);
    destroy(&mut t, mentor);
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
}

#[test]
fn all_the_counters_on_it_count_not_just_its_own() {
    cr!("122.1", "601.2f");
    ruling!(
        "Joraga Warcaller",
        "The last ability counts the total number of +1/+1 counters on Joraga Warcaller, not just the ones placed on it as a result of its \"enters\" ability."
    );
    ruling!(
        "Herald of War",
        "The cost reduction is based on the total number of +1/+1 counters on Herald of War, not just the ones put on it by its own ability."
    );
    supported("Joraga Warcaller");
    supported("Herald of War");
    let mut t = TestGame::new(2);
    let warcaller = with_counters(&mut t, P0, "Joraga Warcaller", 1);
    let elf = t.battlefield(P0, "Llanowar Elves");
    give_plus1(&mut t, warcaller, 2);
    assert_eq!(t.pt(elf), (4, 4));

    let mut t = TestGame::new(2);
    with_counters(&mut t, P0, "Herald of War", 3);
    t.lands(P0, "Plains", 2);
    let angel = t.hand(P0, "Serra Angel");
    t.cast(P0, angel).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Serra Angel").len(), 1);
}

#[test]
fn emil_counts_differently_named_lands() {
    cr!("201.2", "608.2h");
    ruling!(
        "Emil, Vastlands Roamer",
        "To determine the number of differently named lands you control, count each land you control once, but only if its English name isn't exactly the same as another land you've already counted this way."
    );
    supported("Emil, Vastlands Roamer");
    let mut t = TestGame::new(2);
    let emil = t.battlefield(P0, "Emil, Vastlands Roamer");
    t.lands(P0, "Forest", 3);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Plains", 1);
    t.activate(P0, emil, 0, &[]).expect("activate");
    t.resolve_all();
    let fractal = t
        .g
        .permanents()
        .find(|o| o.chars.has_subtype("Fractal"))
        .map(|o| o.id)
        .expect("a Fractal");
    assert_eq!(plus1(&t, fractal), 3);
}

#[test]
fn hangarback_walker_pays_both_xs() {
    cr!("107.3", "601.2f");
    ruling!(
        "Hangarback Walker",
        "The value of each X in Hangarback Walker's mana cost must be equal. For example, if X is 2, you'll pay {4} to cast Hangarback Walker and it will enter the battlefield with two +1/+1 counters on it."
    );
    supported("Hangarback Walker");
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 3);
    let walker = t.hand(P0, "Hangarback Walker");
    assert!(t.cast(P0, walker).x(2).try_go().is_err());
    t.lands(P0, "Wastes", 1);
    t.cast(P0, walker).x(2).go();
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 4);
    t.resolve_all();
    let w = t.named_on_battlefield("Hangarback Walker")[0];
    assert_eq!(plus1(&t, w), 2);
}

#[test]
fn toph_power_is_defined_in_every_zone() {
    cr!("604.3", "208.3");
    ruling!(
        "Toph, the Blind Bandit",
        "The ability that defines Toph, the Blind Bandit's power works in all zones, not just the battlefield."
    );
    supported("Toph, the Blind Bandit");
    let mut t = TestGame::new(2);
    let forest = t.lands(P0, "Forest", 1)[0];
    give_plus1(&mut t, forest, 2);
    let toph = t.hand(P0, "Toph, the Blind Bandit");
    let gy = t.graveyard(P0, "Toph, the Blind Bandit");
    t.g.recompute();
    assert_eq!(t.obj_now(toph).chars.power, Some(2));
    assert_eq!(t.obj_now(gy).chars.power, Some(2));
}

#[test]
fn weapon_rack_counters_matter_only_if_it_becomes_a_creature() {
    cr!("122.1a", "613.4c");
    ruling!(
        "Weapon Rack",
        "The +1/+1 counters on Weapon Rack won't affect it unless it somehow becomes a creature."
    );
    supported("Weapon Rack");
    supported("March of the Machines");
    let mut t = TestGame::new(2);
    let rack = t.enter(P0, "Weapon Rack");
    assert_eq!(plus1(&t, rack), 3);
    assert_eq!(t.obj_now(rack).chars.power, None);
    // March of the Machines: it's a 4/4 artifact creature (its mana value), plus three.
    t.battlefield(P0, "March of the Machines");
    t.g.recompute();
    assert_eq!(t.pt(rack), (7, 7));
}

#[test]
fn sigardas_summons_reverts_creatures_that_lose_their_counters() {
    cr!("613.4b", "509.1b", "704.5g");
    ruling!(
        "Sigarda's Summons",
        "Removing all +1/+1 counters from a creature affected by this ability or removing Sigarda's Summons from the battlefield will cause that creature to revert back to what it was. Notably, this may cause a creature to lose flying after it has been declared as an attacker but before blockers are declared. This may also result in a creature with damage already marked on it being destroyed if that damage is greater than the creature's toughness."
    );
    supported("Sigarda's Summons");
    // Losing flying after attacking: a creature without flying or reach can block it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sigarda's Summons");
    let bears = with_counters(&mut t, P0, "Grizzly Bears", 1);
    t.g.recompute();
    assert!(t.obj_now(bears).chars.has_subtype("Angel"));
    assert_eq!(t.pt(bears), (5, 5));
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    remove_plus1(&mut t, bears, 1);
    assert!(!has_kw(&mut t, bears, KeywordKind::Flying));
    block_and_finish(&mut t, P1, &[(giant, bears)]);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // Damage marked on it that's lethal once it's a 3/3 again.
    let mut t = TestGame::new(2);
    let summons = t.battlefield(P0, "Sigarda's Summons");
    let bears = with_counters(&mut t, P0, "Grizzly Bears", 1);
    damage(&mut t, bears, 4);
    assert!(t.on_battlefield(bears));
    exile_now(&mut t, summons);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn cytoplast_manipulator_control_lasts_while_it_remains() {
    cr!("611.2b", "613.2");
    ruling!(
        "Cytoplast Manipulator",
        "The control change lasts as long as Cytoplast Manipulator is on the battlefield. It doesn't end if the creature loses all its +1/+1 counters, it stops being a creature, or if Cytoplast Manipulator becomes untapped."
    );
    supported("Cytoplast Manipulator");
    let mut t = TestGame::new(2);
    let manip = with_counters(&mut t, P0, "Cytoplast Manipulator", 2);
    t.lands(P0, "Island", 1);
    let theirs = with_counters(&mut t, P1, "Grizzly Bears", 1);
    t.activate(P0, manip, 0, &[obj(theirs)]).expect("activate");
    t.resolve_all();
    let controller = |t: &TestGame| t.g.obj(t.g.current(theirs)).controller;
    assert_eq!(controller(&t), P0);
    remove_plus1(&mut t, theirs, 1);
    let m = t.g.current(manip);
    t.g.untap(m);
    t.g.recompute();
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(controller(&t), P0);
    exile_now(&mut t, manip);
    t.g.recompute();
    assert_eq!(controller(&t), P1);
}
