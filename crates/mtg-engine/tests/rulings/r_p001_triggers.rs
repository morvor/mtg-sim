//! Rulings batch P001 — triggered abilities: targets chosen as the ability is put on the
//! stack (CR 603.3d, 115.1), a random choice made as it resolves among the legal targets
//! (CR 608.2b), a target that's no longer legal (CR 608.2b), choices made by another
//! player on resolution (CR 608.2d), "sacrifice it" (CR 701.21), "other creatures" as the
//! ability resolves (CR 611.2c), and opponents choosing in turn order (CR 101.4).

use crate::r_p001_common::*;
use crate::r_p006_common::{choices_since, in_graveyard, in_hand};
use crate::r_s01_common::{stack_library, supported, watch};
use crate::r_s02_common::{create_token, destroy, target_candidates};
use crate::r_s04_common::add_mana;
use crate::r_s25_common::lands_for_cost;
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Ends P1's turn and goes to P0's upkeep (its beginning-of-upkeep triggers on the stack).
fn to_p0_upkeep(t: &mut TestGame) {
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
}

// ---------------------------------------------------------------------------
// Capricious Efreet: "At the beginning of your upkeep, choose target nonland permanent
// you control and up to two target nonland permanents you don't control. Destroy one of
// them at random."
// ---------------------------------------------------------------------------

const EFREET: &str = "Capricious Efreet";

#[test]
fn capricious_efreet_must_target_itself_if_its_your_only_nonland_permanent() {
    cr!("603.3d", "115.1");
    ruling!(
        "Capricious Efreet",
        "If Capricious Efreet is the only nonland permanent you control when its ability triggers, you’ll have to target it."
    );
    supported(EFREET);
    let mut t = TestGame::new(2);
    let efreet = t.battlefield(P0, EFREET);
    t.lands(P0, "Mountain", 2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(efreet)]);
    t.answer_targets(P0, &[obj(a)]);
    to_p0_upkeep(&mut t);
    let offered = target_candidates(&t, P0, from);
    assert_eq!(offered[0], vec![obj(efreet)]);
    assert_eq!(t.stack_len(), 1);
    // Both targets are destroyed by one at random: one of them is gone.
    t.resolve_all();
    assert_eq!(
        [efreet, a].iter().filter(|c| t.on_battlefield(**c)).count(),
        1
    );
}

#[test]
fn capricious_efreet_may_target_no_permanents_you_dont_control() {
    cr!("115.1", "601.2c", "608.2b");
    ruling!(
        "Capricious Efreet",
        "You may target zero, one, or two nonland permanents you don’t control."
    );
    for n in 0..=2usize {
        let mut t = TestGame::new(2);
        let efreet = t.battlefield(P0, EFREET);
        let theirs: Vec<ObjectId> = (0..2).map(|_| t.battlefield(P1, "Grizzly Bears")).collect();
        let chosen: Vec<Entity> = theirs[..n].iter().map(|c| obj(*c)).collect();
        t.answer_targets(P0, &[obj(efreet)]);
        t.answer_targets(P0, &chosen);
        to_p0_upkeep(&mut t);
        let ability = *t.g.stack.last().unwrap();
        let mut targeted = crate::r_s25_common::targets_of(&t, ability);
        targeted.sort();
        let mut expected = vec![obj(efreet)];
        expected.extend(chosen.clone());
        expected.sort();
        assert_eq!(targeted, expected);
        t.resolve_all();
        let left = std::iter::once(efreet)
            .chain(theirs[..n].iter().copied())
            .filter(|c| t.on_battlefield(*c))
            .count();
        assert_eq!(left, n, "one of the {} targets is destroyed", n + 1);
        // Untargeted permanents are safe.
        assert!(theirs[n..].iter().all(|c| t.on_battlefield(*c)));
    }
}

#[test]
fn capricious_efreet_chooses_at_random_among_the_targets_still_legal() {
    cr!("608.2b");
    ruling!(
        "Capricious Efreet",
        "You target between one and three permanents as you put the ability on the stack. You don’t randomly choose which one will be destroyed until the ability resolves. If one of those permanents has become an illegal target by then, you randomly choose between the remaining ones."
    );
    // The Efreet and one of the two others leave before it resolves: the third one is
    // the one destroyed, every time.
    for seed in 0..6u64 {
        let mut t = TestGame::new(2);
        t.g.rng = <rand_chacha::ChaCha8Rng as rand::SeedableRng>::seed_from_u64(seed);
        let efreet = t.battlefield(P0, EFREET);
        let a = t.battlefield(P1, "Grizzly Bears");
        let b = t.battlefield(P1, "Hill Giant");
        t.answer_targets(P0, &[obj(efreet)]);
        t.answer_targets(P0, &[obj(a), obj(b)]);
        to_p0_upkeep(&mut t);
        assert_eq!(t.stack_len(), 1);
        let e = t.g.current(efreet);
        t.g.move_object(e, Zone::Hand(P0), mtg_engine::events::MoveCause::Effect, None);
        let a2 = t.g.current(a);
        t.g.move_object(a2, Zone::Hand(P1), mtg_engine::events::MoveCause::Effect, None);
        t.resolve_all();
        assert!(in_graveyard(&t, b), "seed {seed}");
        assert!(in_hand(&t, efreet) && in_hand(&t, a));
    }
}

#[test]
fn capricious_efreet_regeneration_must_be_set_up_before_it_resolves() {
    cr!("608.2", "701.19a", "701.19c");
    ruling!(
        "Capricious Efreet",
        "As the ability resolves, there is no time to react between the time a permanent is chosen at random and the time it’s destroyed."
    );
    supported("Regenerate");
    // A regeneration shield on each of the three targets, made before the ability
    // resolves: whichever is chosen regenerates (it's tapped), and nobody is asked
    // anything as it resolves.
    let mut t = TestGame::new(2);
    let efreet = t.battlefield(P0, EFREET);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(efreet)]);
    t.answer_targets(P0, &[obj(a), obj(b)]);
    to_p0_upkeep(&mut t);
    let ability = *t.g.stack.last().unwrap();
    for c in [efreet, a, b] {
        let card = t.hand(P0, "Regenerate");
        add_mana(&mut t, P0, ManaType::G, 2);
        t.cast(P0, card).target(c).go();
        t.resolve();
    }
    assert_eq!(t.g.stack, vec![ability]);
    let from = t.asked().len();
    t.resolve();
    assert!(t.asked()[from..].is_empty(), "{:?}", &t.asked()[from..]);
    assert!([efreet, a, b].iter().all(|c| t.on_battlefield(*c)));
    assert_eq!(
        [efreet, a, b]
            .iter()
            .filter(|c| t.obj_now(**c).tapped)
            .count(),
        1
    );
}

// ---------------------------------------------------------------------------
// Crumbling Ashes: "At the beginning of your upkeep, destroy target creature with a
// -1/-1 counter on it."
// ---------------------------------------------------------------------------

fn with_minus_counter(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.battlefield(p, name);
    t.g.add_counters(obj(id), counters::MINUS1, 1, None);
    t.g.recompute();
    id
}

#[test]
fn crumbling_ashes_does_nothing_if_the_counter_is_gone() {
    cr!("608.2b", "704.5q");
    ruling!(
        "Crumbling Ashes",
        "If the targeted creature loses all its -1/-1 counters by the time the ability resolves, the ability won’t resolve because its only target is illegal. The creature won’t be destroyed."
    );
    supported("Crumbling Ashes");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Crumbling Ashes");
    let giant = with_minus_counter(&mut t, P1, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    to_p0_upkeep(&mut t);
    assert_eq!(t.stack_len(), 1);
    // In response, a +1/+1 counter: the two counters are removed (CR 704.5q).
    t.g.add_counters(obj(giant), counters::PLUS1, 1, None);
    t.settle();
    assert_eq!(t.counters(giant, counters::MINUS1), 0);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
}

#[test]
fn crumbling_ashes_must_target_your_own_creature_if_its_the_only_one() {
    cr!("603.3d", "115.1");
    ruling!(
        "Crumbling Ashes",
        "This ability is mandatory. If you’re the only player who controls a creature with a -1/-1 counter on it as your upkeep begins, you must target one of your own creatures."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Crumbling Ashes");
    let mine = with_minus_counter(&mut t, P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    to_p0_upkeep(&mut t);
    assert_eq!(t.stack_len(), 1);
    for c in target_candidates(&t, P0, from) {
        assert_eq!(c, vec![obj(mine)]);
    }
    t.resolve_all();
    assert!(in_graveyard(&t, mine));
    assert!(t.on_battlefield(theirs));
}

#[test]
fn crumbling_ashes_needs_the_counter_as_the_upkeep_begins() {
    cr!("603.3d", "503.1a");
    ruling!(
        "Crumbling Ashes",
        "You can’t target a creature unless it already has a -1/-1 counter on it as your upkeep begins. Putting a -1/-1 counter on a creature during your upkeep won’t let Crumbling Ashes destroy it that turn."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Crumbling Ashes");
    let giant = t.battlefield(P1, "Hill Giant");
    to_p0_upkeep(&mut t);
    // No legal target as it triggered: it's removed from the stack.
    assert_eq!(t.stack_len(), 0);
    t.g.add_counters(obj(giant), counters::MINUS1, 1, None);
    t.settle();
    t.advance_to(P0, Step::PrecombatMain);
    assert!(t.on_battlefield(giant));
}

// ---------------------------------------------------------------------------
// Neyam Shai Murad: "Whenever Neyam Shai Murad deals combat damage to a player, you may
// have that player return target permanent card from their graveyard to their hand. If
// you do, that player chooses a permanent card in your graveyard, then you put it onto
// the battlefield under your control."
// ---------------------------------------------------------------------------

#[test]
fn neyam_shai_murad_target_and_choice() {
    cr!("603.3d", "608.2d", "601.2c");
    ruling!(
        "Neyam Shai Murad",
        "You choose the target card in your opponent's graveyard as you put the triggered ability on the stack. That opponent chooses the card in your graveyard as the ability resolves."
    );
    supported("Neyam Shai Murad");
    let mut t = TestGame::new(2);
    let neyam = t.battlefield(P0, "Neyam Shai Murad");
    let giant = t.graveyard(P1, "Hill Giant");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let wurm = t.graveyard(P0, "Craw Wurm");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(giant)]);
    t.g.deal_damage(neyam, pl(P1), 3, true);
    t.settle();
    // The target was chosen by P0 as the trigger was put on the stack.
    assert_eq!(t.stack_len(), 1);
    let targets_from = target_candidates(&t, P0, from);
    assert_eq!(targets_from.len(), 1);
    assert!(targets_from[0].contains(&obj(giant)));
    assert!(choices_since(&t, P1, from).is_empty());
    // As it resolves, P1 chooses among P0's permanent cards.
    t.answer_yes(P0, true);
    t.answer_choose(P1, &[obj(bears)]);
    let from = t.asked().len();
    t.resolve_all();
    let p1_choices = choices_since(&t, P1, from);
    assert_eq!(p1_choices.len(), 1);
    assert!(p1_choices[0].0.contains(&obj(bears)) && p1_choices[0].0.contains(&obj(wurm)));
    assert!(t.in_hand(P1, "Hill Giant"));
    assert!(controlled_by(&t, bears, P0));
    assert!(in_graveyard(&t, wurm));
}

#[test]
fn neyam_shai_murad_does_nothing_if_its_target_is_gone() {
    cr!("608.2b");
    ruling!(
        "Neyam Shai Murad",
        "If the card in your opponent's graveyard isn't a legal target as the ability tries to resolve, it will be removed from the stack and your opponent won't get to choose a permanent card from your graveyard."
    );
    let mut t = TestGame::new(2);
    let neyam = t.battlefield(P0, "Neyam Shai Murad");
    let giant = t.graveyard(P1, "Hill Giant");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(giant)]);
    t.g.deal_damage(neyam, pl(P1), 3, true);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let g = t.g.current(giant);
    t.g.exile_object(g, None);
    t.answer_yes(P0, true);
    let from = t.asked().len();
    t.resolve_all();
    assert!(choices_since(&t, P1, from).is_empty());
    assert!(in_graveyard(&t, bears));
    assert_eq!(t.zone(giant), Zone::Exile);
}

#[test]
fn aspiring_champion_must_be_sacrificed() {
    cr!("701.21a", "603.5");
    ruling!(
        "Aspiring Champion",
        "If you still control Aspiring Champion as its triggered ability resolves, you must sacrifice it. You can't choose not to sacrifice it."
    );
    supported("Aspiring Champion");
    let mut t = TestGame::new(2);
    let champ = t.battlefield(P0, "Aspiring Champion");
    stack_library(&mut t, P0, &["Island", "Hill Giant"]);
    t.answer_yes(P0, false);
    let from = t.asked().len();
    t.g.deal_damage(champ, pl(P1), 3, true);
    t.settle();
    t.resolve_all();
    let yes_no = t.asked()[from..]
        .iter()
        .filter(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. }))
        .count();
    assert_eq!(yes_no, 0);
    assert!(in_graveyard(&t, champ));
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn harvester_of_misery_affects_only_creatures_there_as_it_resolves() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Harvester of Misery",
        "Harvester of Misery's second ability affects only other creatures that are on the battlefield at the time it resolves. Creatures that enter the battlefield later in the turn and noncreature permanents that become creatures later in the turn won't get -2/-2."
    );
    supported("Harvester of Misery");
    supported("Mutavault");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let vault = t.battlefield(P0, "Mutavault");
    lands_for_cost(&mut t, P0, "Harvester of Misery");
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Harvester of Misery");
    let harvester = t.cast(P0, card).go();
    t.resolve_all();
    assert_eq!(t.pt(giant), (1, 1));
    assert_eq!(t.pt(harvester), (5, 4));
    // Later: a creature enters, and Mutavault becomes a 2/2 creature.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, vault, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(vault), (2, 2));
}

#[test]
fn refurbished_familiars_opponents_choose_in_turn_order_then_discard_together() {
    cr!("101.4", "608.2d", "701.9a");
    ruling!(
        "Refurbished Familiar",
        "When Refurbished Familiar's last ability resolves, the next opponent in turn order (or, if it's an opponent's turn, that opponent) chooses a card in hand without revealing it, then each other opponent in turn order (if any) does the same. All chosen cards are then discarded at the same time."
    );
    supported("Refurbished Familiar");
    // It's P2's turn: P2 chooses first, then P3 (no cards), then P1.
    let mut t = TestGame::new(4);
    t.set_step(P2, Step::PrecombatMain);
    for p in [P1, P2] {
        t.hand(p, "Island");
        t.hand(p, "Grizzly Bears");
    }
    // What's in P2's hand when P1 (the last to choose) chooses.
    let seen = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| g.player(P2).hand.len(),
    );
    let from = t.asked().len();
    let p0_hand = t.hand_size(P0);
    t.enter(P0, "Refurbished Familiar");
    t.settle();
    t.resolve_all();
    let order: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order, vec![P2, P1]);
    assert_eq!(*seen.lock().unwrap(), vec![2]);
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(t.hand_size(P2), 1);
    // P3 couldn't discard: P0 draws a card.
    assert_eq!(t.hand_size(P0), p0_hand + 1);
}

#[test]
fn exiled_until_leaves_auras_fall_off_equipment_stays_counters_vanish() {
    cr!("400.7", "704.5m", "704.5n", "122.2", "610.3");
    ruling!(
        "Bishop of Binding",
        "Auras attached to the exiled creature will be put into their owners’ graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled creature will cease to exist."
    );
    ruling!(
        "Trapjaw Tyrant",
        "Auras attached to the exiled creature will be put into their owners’ graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled creature will cease to exist."
    );
    supported("Bishop of Binding");
    supported("Trapjaw Tyrant");
    for name in ["Bishop of Binding", "Trapjaw Tyrant"] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P1, "Hill Giant");
        let aura = t.battlefield(P1, "Holy Strength");
        t.g.attach(aura, obj(giant));
        let sword = t.battlefield(P1, "Bonesplitter");
        t.g.attach(sword, obj(giant));
        t.g.add_counters(obj(giant), counters::PLUS1, 2, None);
        t.g.recompute();
        assert_eq!(t.pt(giant), (8, 7));
        t.answer_targets(P0, &[obj(giant)]);
        let exiler = if name == "Bishop of Binding" {
            let b = t.enter(P0, name);
            t.settle();
            b
        } else {
            let tyrant = t.battlefield(P0, name);
            let bolt = t.hand(P1, "Shock");
            t.set_step(P1, Step::PrecombatMain);
            add_mana(&mut t, P1, ManaType::R, 1);
            t.cast(P1, bolt).target(tyrant).go();
            t.resolve();
            tyrant
        };
        t.resolve_all();
        assert_eq!(t.zone(giant), Zone::Exile, "{name}");
        assert!(in_graveyard(&t, aura), "{name}");
        assert!(t.on_battlefield(sword), "{name}");
        assert!(t.obj_now(sword).attached_to.is_none(), "{name}");
        // The creature comes back as a new object, without the counters.
        destroy(&mut t, exiler);
        t.resolve_all();
        assert!(t.on_battlefield(giant), "{name}");
        assert_eq!(t.counters(giant, counters::PLUS1), 0);
        assert_eq!(t.pt(giant), (3, 3));
    }
}

#[test]
fn ravenous_slime_exiles_a_dying_token() {
    cr!("614.1a", "111.7", "700.4");
    ruling!(
        "Ravenous Slime",
        "A token creature can die just like a nontoken creature before ceasing to exist. Ravenous Slime can replace this by exiling the token."
    );
    supported("Ravenous Slime");
    let mut t = TestGame::new(2);
    let slime = t.battlefield(P0, "Ravenous Slime");
    let token = create_token(&mut t, P1, "Soldier");
    t.g.add_counters(obj(token), counters::PLUS1, 2, None);
    t.g.recompute();
    assert_eq!(t.pt(token), (3, 3));
    destroy(&mut t, token);
    // Exiled instead of dying (no token went to a graveyard), then it ceased to exist.
    assert!(t.g.player(P1).graveyard.is_empty());
    assert!(!t.on_battlefield(token));
    assert_eq!(t.counters(slime, counters::PLUS1), 3);
    assert_eq!(t.pt(slime), (4, 4));
}

#[test]
fn hone_counters_grant_power_whatever_the_equipment_says() {
    cr!("122.1j");
    ruling!(
        "Dwalin, Weaponmaster",
        "Hone counters can be put on any Equipment and each one will grant +1/+0 to that Equipment's equipped creature, even if the Equipment the hone counters are on doesn't reference hone counters or have an ability that puts them on itself."
    );
    supported("Dwalin, Weaponmaster");
    // Bonesplitter ("+2/+0") doesn't mention hone counters.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sword = t.battlefield(P0, "Bonesplitter");
    t.g.attach(sword, obj(bears));
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 2));
    t.enter(P0, "Dwalin, Weaponmaster");
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(sword, "hone"), 1);
    assert_eq!(t.pt(bears), (5, 2));
}

#[test]
fn hone_counters_grant_power_even_if_the_equipment_loses_its_abilities() {
    cr!("122.1j", "613.1f");
    ruling!(
        "Sting, Bilbo's Sword",
        "A creature equipped by an Equipment with hone counters on it gets +1/+0 for each hone counter on that Equipment. This effect is from the counter itself rather than an ability on the Equipment, so even if the Equipment loses some of its abilities, each hone counter will still grant +1/+0 to the creature."
    );
    supported("Sting, Bilbo's Sword");
    supported("Stop Cold");
    // Sting enters with a hone counter for each of P1's two creatures and is attached to
    // the Bears.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    let card = t.hand(P0, "Sting, Bilbo's Sword");
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[obj(bears)]);
    let sting = t.cast(P0, card).go();
    t.resolve_all();
    assert_eq!(t.counters(sting, "hone"), 2);
    assert_eq!(t.pt(bears), (4, 2));
    // Stop Cold: "Enchanted permanent loses all abilities ..." (and it's tapped).
    t.set_step(P1, Step::PrecombatMain);
    let cold = t.hand(P1, "Stop Cold");
    add_mana(&mut t, P1, ManaType::U, 4);
    let sting = t.g.current(sting);
    t.cast(P1, cold).target(sting).go();
    t.resolve_all();
    assert!(t.obj_now(sting).chars.abilities.is_empty());
    assert_eq!(t.obj_now(sting).attached_to, Some(obj(t.g.current(bears))));
    assert_eq!(t.pt(bears), (4, 2));
}
