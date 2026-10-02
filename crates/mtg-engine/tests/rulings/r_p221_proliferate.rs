//! Rulings batch P221 — proliferate (CR 701.34a): proliferating twice, proliferating
//! after lethal damage, proliferate on spells whose targets all became illegal, and
//! proliferating from triggered abilities.

use crate::r_s01_common::*;
use crate::r_s02_common::{destroy, target_candidates};
use crate::r_s03_common::choice_candidates;
use crate::r_s06_common::damage;
use crate::r_s13_common::add;
use crate::r_s21_common::castable;
use crate::r_s04_common::crew;
use crate::r_s06_common::activate_containing;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const PROLIFERATE: &str = "Proliferate: choose";

/// The candidates of each proliferate choice asked since decision `from`.
fn proliferations(t: &TestGame, from: usize) -> Vec<Vec<Entity>> {
    choice_candidates(t, from, PROLIFERATE)
}

/// Indices (in the decision log) of the proliferate choices since `from`.
fn proliferate_choices(t: &TestGame, from: usize) -> Vec<usize> {
    t.asked()
        .iter()
        .enumerate()
        .skip(from)
        .filter(|(_, (_, d))| {
            matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.starts_with(PROLIFERATE))
        })
        .map(|(i, _)| i)
        .collect()
}

/// Whether anyone was asked for priority between decisions `a` and `b`.
fn priority_between(t: &TestGame, a: usize, b: usize) -> bool {
    t.asked()[a + 1..b]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. }))
}

/// A Hill Giant (3/3) with `n` counters of `kind`.
fn giant_with(t: &mut TestGame, p: PlayerId, kind: &str, n: u32) -> ObjectId {
    let b = t.battlefield(p, "Hill Giant");
    add(t, b, kind, n);
    b
}

#[test]
fn contagion_engine_proliferates_twice_choosing_anew_with_no_window_between() {
    cr!("701.34a", "608.2c", "117.3b");
    ruling!(
        "Contagion Engine",
        "As Contagion Engine's activated ability resolves, you'll complete an entire proliferate action, then you'll complete a second proliferate action. You may choose different players and/or permanents"
    );
    ruling!(
        "Contagion Engine",
        "Players can't respond between the first and the second proliferate actions."
    );
    supported("Contagion Engine");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Wastes", 4);
    let engine = t.battlefield(P0, "Contagion Engine");
    let mine = giant_with(&mut t, P0, counters::PLUS1, 1);
    let theirs = giant_with(&mut t, P1, counters::PLUS1, 1);
    // First: only my Bears; second: only the opponent's.
    t.answer_choose(P0, &[Entity::Object(mine)]);
    t.answer_choose(P0, &[Entity::Object(theirs)]);
    t.activate(P0, engine, 0, &[]).expect("Contagion Engine");
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(t.counters(mine, counters::PLUS1), 2);
    assert_eq!(t.counters(theirs, counters::PLUS1), 2);
    let i = proliferate_choices(&t, from);
    assert_eq!(i.len(), 2);
    assert!(!priority_between(&t, i[0], i[1]));
}

#[test]
fn proliferating_twice_lets_you_choose_different_sets() {
    cr!("701.34a", "603.2");
    ruling!(
        "Ezuri, Stalker of Spheres",
        "If you proliferate twice, you don't have to choose the same set of players and/or permanents to get additional counters each time."
    );
    ruling!(
        "Agent Frank Horrigan",
        "If you proliferate twice, you don’t have to choose the same set of players and/or permanents to get additional counters each time."
    );
    supported("Ezuri, Stalker of Spheres");
    supported("Agent Frank Horrigan");
    // Ezuri: "When Ezuri enters, you may pay {3}. If you do, proliferate twice." and
    // "Whenever you proliferate, draw a card."
    let mut t = TestGame::new(2);
    let a = giant_with(&mut t, P0, counters::PLUS1, 1);
    let b = giant_with(&mut t, P0, counters::MINUS1, 1);
    t.lands(P0, "Wastes", 3);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.answer_choose(P0, &[Entity::Object(b)]);
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    t.enter(P0, "Ezuri, Stalker of Spheres");
    t.resolve_all();
    assert_eq!(proliferations(&t, from).len(), 2);
    assert_eq!(t.counters(a, counters::PLUS1), 2);
    assert_eq!(t.counters(b, counters::MINUS1), 2);
    // Each proliferation is a separate "you proliferate": two cards.
    assert_eq!(t.hand_size(P0), hand + 2);
    // Agent Frank Horrigan ("Whenever ~ enters or attacks, proliferate twice."): the same
    // permanent both times, and a player only the second time.
    let mut t = TestGame::new(2);
    let a = giant_with(&mut t, P0, counters::PLUS1, 1);
    add(&mut t, P1, counters::POISON, 1);
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.answer_choose(P0, &[Entity::Object(a), Entity::Player(P1)]);
    let from = t.asked().len();
    t.enter(P0, "Agent Frank Horrigan");
    t.resolve_all();
    assert_eq!(proliferations(&t, from).len(), 2);
    assert_eq!(t.counters(a, counters::PLUS1), 3);
    assert_eq!(t.g.player(P1).counter(counters::POISON), 2);
}

#[test]
fn roalesk_proliferates_twice_with_nothing_in_between() {
    cr!("701.34a", "608.2c", "117.3b");
    ruling!(
        "Roalesk, Apex Hybrid",
        "You proliferate twice all while Roalesk's last ability is resolving. Nothing can happen between the two proliferations, and no player may choose to take actions."
    );
    supported("Roalesk, Apex Hybrid");
    let mut t = TestGame::new(2);
    let roalesk = t.battlefield(P0, "Roalesk, Apex Hybrid");
    let a = giant_with(&mut t, P0, counters::PLUS1, 1);
    let b = giant_with(&mut t, P1, counters::MINUS1, 1);
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    destroy(&mut t, roalesk);
    assert!(t.in_graveyard(P0, "Roalesk, Apex Hybrid"));
    let from = t.asked().len();
    t.resolve_all();
    let i = proliferate_choices(&t, from);
    assert_eq!(i.len(), 2);
    assert!(!priority_between(&t, i[0], i[1]));
    assert_eq!(t.counters(a, counters::PLUS1), 3);
    assert_eq!(t.counters(b, counters::MINUS1), 2);
}

#[test]
fn a_creature_dealt_lethal_damage_dies_before_its_proliferate_trigger_resolves() {
    cr!("704.5g", "704.3", "603.3", "701.34a");
    ruling!(
        "Park Heights Maverick",
        "If Park Heights Maverick has a +1/+1 counter on it and is dealt lethal damage, it will die before its triggered ability resolves and you proliferate."
    );
    ruling!(
        "Urban Daggertooth",
        "If Urban Daggertooth has a +1/+1 counter on it and is dealt lethal damage, it will die before its triggered ability resolves and you proliferate."
    );
    supported("Park Heights Maverick");
    supported("Urban Daggertooth");
    // Park Heights Maverick ("Whenever this creature deals combat damage to a player or
    // dies, proliferate."), a 3/3 with its counter, is dealt 3 damage.
    let mut t = TestGame::new(2);
    let maverick = t.battlefield(P0, "Park Heights Maverick");
    add(&mut t, maverick, counters::PLUS1, 1);
    let other = giant_with(&mut t, P0, counters::PLUS1, 1);
    let shock = t.battlefield(P1, "Prodigal Pyromancer");
    let from = t.asked().len();
    damage(&mut t, shock, 3, maverick);
    // It's already gone when the trigger is put on the stack.
    assert!(t.in_graveyard(P0, "Park Heights Maverick"));
    assert_eq!(triggers_on_stack(&t, "proliferate"), 1);
    t.answer_choose(P0, &[Entity::Object(other)]);
    t.resolve_all();
    assert_eq!(proliferations(&t, from), vec![vec![Entity::Object(other)]]);
    // Urban Daggertooth ("Enrage — Whenever this creature is dealt damage, proliferate."),
    // a 5/4 with its counter, is dealt 4 damage.
    let mut t = TestGame::new(2);
    let dagger = t.battlefield(P0, "Urban Daggertooth");
    add(&mut t, dagger, counters::PLUS1, 1);
    assert_eq!(t.pt(dagger), (5, 4));
    let other = giant_with(&mut t, P0, counters::PLUS1, 1);
    let src = t.battlefield(P1, "Prodigal Pyromancer");
    let from = t.asked().len();
    damage(&mut t, src, 4, dagger);
    assert!(t.in_graveyard(P0, "Urban Daggertooth"));
    t.answer_choose(P0, &[Entity::Object(other)]);
    t.resolve_all();
    assert_eq!(proliferations(&t, from), vec![vec![Entity::Object(other)]]);
    assert_eq!(t.counters(other, counters::PLUS1), 2);
}

#[test]
fn martyr_for_the_cause_cant_save_a_creature_dealt_lethal_damage_with_it() {
    cr!("704.3", "704.5g", "603.3", "701.34a");
    ruling!(
        "Martyr for the Cause",
        "If another creature with a +1/+1 counter on it is dealt lethal damage at the same time as Martyr for the Cause, the triggered ability can't proliferate a +1/+1 counter on the other creature in time to save it."
    );
    supported("Martyr for the Cause");
    supported("Pyroclasm");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Martyr for the Cause");
    // A 2/2 Llanowar Elves (with a +1/+1 counter) and the 2/2 Martyr; Pyroclasm deals 2
    // damage to each creature. A Hill Giant with a counter survives.
    let elves = t.battlefield(P0, "Llanowar Elves");
    add(&mut t, elves, counters::PLUS1, 1);
    let survivor = giant_with(&mut t, P0, counters::PLUS1, 1);
    let pyro = t.hand(P1, "Pyroclasm");
    give_mana_for(&mut t, P1, "Pyroclasm");
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, pyro).go();
    t.resolve();
    // Both died at once; the trigger hasn't resolved yet.
    assert!(t.in_graveyard(P0, "Martyr for the Cause"));
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    let _ = elves;
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Object(survivor)]);
    t.resolve_all();
    assert_eq!(proliferations(&t, from), vec![vec![Entity::Object(survivor)]]);
    assert_eq!(t.counters(survivor, counters::PLUS1), 2);
}

#[test]
fn a_proliferate_spell_whose_targets_are_all_illegal_doesnt_proliferate() {
    cr!("608.2b", "701.34a");
    ruling!(
        "Spread the Sickness",
        "If the creature is an illegal target when Spread the Sickness tries to resolve, the spell doesn't resolve. You won't proliferate."
    );
    ruling!(
        "Volt Charge",
        "If the permanent or player is an illegal target when Volt Charge tries to resolve, it won't resolve and none of its effects will happen. You won't proliferate."
    );
    ruling!(
        "Smell Fear",
        "If all targets are illegal as Smell Fear tries to resolve, it doesn't resolve and you will not proliferate."
    );
    ruling!(
        "Recon Craft Theta",
        "Some spells and abilities that cause you to proliferate may require targets. If each target chosen is an illegal target as that spell or ability tries to resolve, it won't resolve. You won't proliferate."
    );
    for name in ["Spread the Sickness", "Volt Charge", "Smell Fear"] {
        supported(name);
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let counted = giant_with(&mut t, P0, counters::PLUS1, 1);
        let mine = t.battlefield(P0, "Hill Giant");
        let theirs = t.battlefield(P1, "Hill Giant");
        let spell = t.hand(P0, name);
        give_mana_for(&mut t, P0, name);
        let targets = if name == "Smell Fear" {
            vec![Entity::Object(mine), Entity::Object(theirs)]
        } else {
            vec![Entity::Object(theirs)]
        };
        let mut b = t.cast(P0, spell);
        for e in &targets {
            b = b.target(*e);
        }
        b.go();
        // Every target leaves before it resolves.
        for e in &targets {
            if let Entity::Object(o) = e {
                destroy(&mut t, *o);
            }
        }
        let from = t.asked().len();
        t.answer_choose(P0, &[Entity::Object(counted)]);
        t.resolve_all();
        assert!(t.in_graveyard(P0, name), "{name}");
        assert!(proliferations(&t, from).is_empty(), "{name}");
        assert_eq!(t.counters(counted, counters::PLUS1), 1, "{name}");
    }
    // Smell Fear with one target still legal does resolve and proliferate.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let counted = giant_with(&mut t, P0, counters::PLUS1, 1);
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Hill Giant");
    let spell = t.hand(P0, "Smell Fear");
    give_mana_for(&mut t, P0, "Smell Fear");
    t.cast(P0, spell).target(mine).target(theirs).go();
    destroy(&mut t, theirs);
    t.answer_choose(P0, &[Entity::Object(counted)]);
    t.resolve_all();
    assert_eq!(t.counters(counted, counters::PLUS1), 2);
}

#[test]
fn spread_the_sickness_still_proliferates_if_the_creature_survives() {
    cr!("608.2b", "701.19a", "702.12b", "701.34a");
    ruling!(
        "Spread the Sickness",
        "If the creature regenerates or has indestructible when Spread the Sickness resolves, you'll still proliferate."
    );
    // Indestructible: Darksteel Myr.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let counted = giant_with(&mut t, P0, counters::PLUS1, 1);
    let myr = t.battlefield(P1, "Darksteel Myr");
    let spell = t.hand(P0, "Spread the Sickness");
    give_mana_for(&mut t, P0, "Spread the Sickness");
    t.cast(P0, spell).target(myr).go();
    t.answer_choose(P0, &[Entity::Object(counted)]);
    t.resolve_all();
    assert!(t.on_battlefield(myr));
    assert_eq!(t.counters(counted, counters::PLUS1), 2);
    // Regeneration: Drudge Skeletons with a regeneration shield.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let counted = giant_with(&mut t, P0, counters::PLUS1, 1);
    let skeletons = t.battlefield(P1, "Drudge Skeletons");
    t.lands(P1, "Swamp", 1);
    t.activate(P1, skeletons, 0, &[]).expect("regenerate");
    t.resolve_all();
    let spell = t.hand(P0, "Spread the Sickness");
    give_mana_for(&mut t, P0, "Spread the Sickness");
    t.cast(P0, spell).target(skeletons).go();
    t.answer_choose(P0, &[Entity::Object(counted)]);
    t.resolve_all();
    assert!(t.on_battlefield(skeletons));
    assert!(t.obj_now(skeletons).tapped);
    assert_eq!(t.counters(counted, counters::PLUS1), 2);
}

#[test]
fn fuel_for_the_cause_needs_another_spell_and_doesnt_proliferate_if_it_fizzles() {
    cr!("115.5", "601.2c", "608.2b", "701.34a");
    ruling!(
        "Fuel for the Cause",
        "You must be able to target another spell in order to cast Fuel for the Cause. You can't just cast it in order to proliferate, and it can't target itself."
    );
    ruling!(
        "Fuel for the Cause",
        "If the spell is an illegal target when Fuel for the Cause tries to resolve, it won't resolve. You won't proliferate."
    );
    supported("Fuel for the Cause");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let counted = giant_with(&mut t, P0, counters::PLUS1, 1);
    let fuel = t.hand(P0, "Fuel for the Cause");
    give_mana_for(&mut t, P0, "Fuel for the Cause");
    // No other spell: it can't be cast.
    assert!(!castable(&mut t, P0, fuel));
    assert!(t.cast(P0, fuel).try_go().is_err());
    assert!(t.in_hand(P0, "Fuel for the Cause"));
    // The opponent's Lightning Bolt on the stack: it can target only that.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(P0).go();
    let from = t.asked().len();
    let fuel = t.cast(P0, fuel).target(bolt).go();
    assert_eq!(
        target_candidates(&t, P0, from),
        vec![vec![Entity::Object(bolt)]]
    );
    // A Counterspell resolves first and counters the Bolt: Fuel for the Cause fizzles.
    let cs = t.hand(P0, "Counterspell");
    give_mana_for(&mut t, P0, "Counterspell");
    t.cast(P0, cs).target(bolt).go();
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Object(counted)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Fuel for the Cause"));
    assert_eq!(t.zone(fuel), Zone::Graveyard(P0));
    assert!(proliferations(&t, from).is_empty());
    assert_eq!(t.counters(counted, counters::PLUS1), 1);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn brokers_confluence_with_all_targets_illegal_doesnt_proliferate() {
    cr!("608.2b", "700.2d", "701.34a");
    ruling!(
        "Brokers Confluence",
        "If you choose the first mode and also the second or third mode as you cast Brokers Confluence and all targets are illegal as it attempts to resolve, you won't get to proliferate."
    );
    supported("Brokers Confluence");
    // Proliferate twice and phase out a creature that leaves before it resolves.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let counted = giant_with(&mut t, P0, counters::PLUS1, 1);
    let theirs = t.battlefield(P1, "Hill Giant");
    let conf = t.hand(P0, "Brokers Confluence");
    give_mana_for(&mut t, P0, "Brokers Confluence");
    t.cast(P0, conf).modes(&[0, 0, 1]).target(theirs).go();
    destroy(&mut t, theirs);
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Object(counted)]);
    t.answer_choose(P0, &[Entity::Object(counted)]);
    t.resolve_all();
    assert!(proliferations(&t, from).is_empty());
    assert_eq!(t.counters(counted, counters::PLUS1), 1);
    // With its target still legal, it proliferates twice.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let counted = giant_with(&mut t, P0, counters::PLUS1, 1);
    let theirs = t.battlefield(P1, "Hill Giant");
    let conf = t.hand(P0, "Brokers Confluence");
    give_mana_for(&mut t, P0, "Brokers Confluence");
    t.cast(P0, conf).modes(&[0, 0, 1]).target(theirs).go();
    t.answer_choose(P0, &[Entity::Object(counted)]);
    t.answer_choose(P0, &[Entity::Object(counted)]);
    t.resolve_all();
    assert_eq!(t.counters(counted, counters::PLUS1), 3);
    assert!(t.obj_now(theirs).phased_out);
}

#[test]
fn recon_craft_theta_proliferates_when_it_attacks() {
    cr!("701.34a", "603.2", "702.122a");
    supported("Recon Craft Theta");
    // Its own proliferate trigger has no targets, so nothing can make it fizzle.
    let mut t = TestGame::new(2);
    let craft = t.battlefield(P0, "Recon Craft Theta");
    let counted = giant_with(&mut t, P0, counters::PLUS1, 1);
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(crew(&mut t, P0, craft, &[giant]));
    t.resolve_all();
    t.answer_choose(P0, &[Entity::Object(counted)]);
    attack_with(&mut t, &[(craft, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.counters(counted, counters::PLUS1), 2);
}

#[test]
fn inexorable_tides_trigger_resolves_before_the_spell() {
    cr!("603.3", "405.5", "701.34a");
    ruling!(
        "Inexorable Tide",
        "Whenever you cast a spell, Inexorable Tide's ability triggers and goes on the stack on top of it. It will resolve (and you'll proliferate) before the spell resolves."
    );
    supported("Inexorable Tide");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Inexorable Tide");
    // A 1/1 Bears (with a -1/-1 counter) that Lightning Bolt would kill anyway, and a
    // Hill Giant with a +1/+1 counter that survives the Bolt only thanks to the trigger.
    let giant = t.battlefield(P0, "Hill Giant");
    add(&mut t, giant, counters::PLUS1, 1);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(giant).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.resolve();
    // The trigger resolved first: the Bolt is still on the stack.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.counters(giant, counters::PLUS1), 2);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    assert_eq!(t.pt(giant), (5, 5));
}

#[test]
fn cacophony_scamp_sacrifices_and_proliferates_with_no_window_between() {
    cr!("608.2c", "117.3b", "603.3", "701.34a");
    ruling!(
        "Cacophony Scamp",
        "You choose whether to sacrifice Cacophony Scamp as its first ability resolves. No player may respond between the time you sacrifice it and the time you proliferate."
    );
    supported("Cacophony Scamp");
    let mut t = TestGame::new(2);
    let scamp = t.battlefield(P0, "Cacophony Scamp");
    let counted = giant_with(&mut t, P0, counters::PLUS1, 1);
    attack_with(&mut t, &[(scamp, Entity::Player(P1))]);
    let from = t.asked().len();
    // Seen whenever P0 is asked for priority: is the Scamp gone, and the counters.
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::Priority { .. }),
        |g: &mtg_engine::game::Game| {
            (
                g.find_in_zone(Zone::Battlefield, "Cacophony Scamp").is_empty(),
                g.permanents().map(|o| o.counter(counters::PLUS1)).sum::<u32>(),
            )
        },
    );
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(counted)]);
    // The dies trigger: 1 damage to the opponent.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    block_and_finish(&mut t, P1, &[]);
    assert!(t.in_graveyard(P0, "Cacophony Scamp"));
    assert_eq!(t.counters(counted, counters::PLUS1), 2);
    assert_eq!(t.life(P1), 18);
    assert_eq!(proliferations(&t, from).len(), 1);
    // No one ever had priority with the Scamp sacrificed but the counter not yet added.
    assert!(!seen.lock().unwrap().contains(&(true, 1)));
    // Choosing not to sacrifice it: no proliferation.
    let mut t = TestGame::new(2);
    let scamp = t.battlefield(P0, "Cacophony Scamp");
    let counted = giant_with(&mut t, P0, counters::PLUS1, 1);
    attack_with(&mut t, &[(scamp, Entity::Player(P1))]);
    let from = t.asked().len();
    t.answer_yes(P0, false);
    block_and_finish(&mut t, P1, &[]);
    assert!(t.on_battlefield(scamp));
    assert!(proliferations(&t, from).is_empty());
    assert_eq!(t.counters(counted, counters::PLUS1), 1);
}

#[test]
fn proliferate_can_choose_opponents_things_but_not_cards_off_the_battlefield() {
    cr!("701.34a", "702.62a");
    ruling!(
        "Yawgmoth, Thran Physician",
        "To proliferate, you can choose any permanent that has a counter, including ones controlled by opponents. You can't choose cards in any zone other than the battlefield, even if they have counters on them."
    );
    supported("Yawgmoth, Thran Physician");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let yawg = t.battlefield(P0, "Yawgmoth, Thran Physician");
    let theirs = giant_with(&mut t, P1, counters::MINUS1, 1);
    add(&mut t, P1, counters::POISON, 1);
    // A suspended card in exile with time counters.
    let suspended = t.exile(P1, "Rift Bolt");
    add(&mut t, suspended, counters::TIME, 2);
    let card = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    t.answer_choose(P0, &[Entity::Object(card)]);
    t.answer_choose(P0, &[Entity::Object(theirs), Entity::Player(P1)]);
    let from = t.asked().len();
    activate_containing(&mut t, P0, yawg, "Proliferate").expect("Yawgmoth");
    t.resolve_all();
    let c = proliferations(&t, from);
    assert_eq!(c.len(), 1);
    assert!(c[0].contains(&Entity::Object(theirs)));
    assert!(c[0].contains(&Entity::Player(P1)));
    assert!(!c[0].contains(&Entity::Object(suspended)));
    assert_eq!(t.counters(theirs, counters::MINUS1), 2);
    assert_eq!(t.g.player(P1).counter(counters::POISON), 2);
    assert_eq!(t.counters(suspended, counters::TIME), 2);
}
