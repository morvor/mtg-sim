//! Rulings batch P050 (second round) — timing of drain triggers and abilities: triggers
//! that resolve before the spell that caused them (CR 603.3, 405.5), a reflexive trigger
//! for each time its event happens (CR 603.12a), a permanent put onto the battlefield
//! attacking (CR 506.3, 508.3a), phasing and double-faced cards (CR 702.26, 712), life
//! gained this turn (CR 603.4, 119.3), and activation costs locked in before mana
//! abilities (CR 602.2b, 601.2f–h).

use crate::r_s01_common::{attack_with, supported};
use crate::r_s25_common::cast_new;
use crate::r_s29_common::put_counters;
use mtg_engine::decision::Decision;
use mtg_engine::object::{StackKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Triggered abilities on the stack whose source is named `name` (followed by name).
fn triggers_named(t: &TestGame, name: &str) -> usize {
    t.g.stack
        .iter()
        .filter(|id| {
            t.g.obj(**id).stack.as_deref().is_some_and(|si| {
                matches!(&si.kind, StackKind::Triggered { source, .. }
                    if t.g.obj(*source).chars.name.as_str() == name)
            })
        })
        .count()
}

// ---------------------------------------------------------------------------------------
// Papalymo Totolymo and Rusko, Clockmaker
// ---------------------------------------------------------------------------------------

/// P0 controls `name` (put onto the battlefield with `put`) and casts Divination; with
/// `countered`, P1 casts Counterspell targeting it. Checks the trigger resolves before
/// the spell, and even if it's countered: P1 loses 1, P0 gains 1.
fn resolves_before_the_spell(name: &str, put: fn(&mut TestGame) -> ObjectId, countered: bool) {
    supported(name);
    let mut t = TestGame::new(2);
    let source = put(&mut t);
    let div = cast_new(&mut t, P0, "Divination", &[]);
    t.settle();
    assert_eq!(triggers_named(&t, name), 1, "{name}");
    assert_eq!(t.stack_len(), 2);
    if countered {
        cast_new(&mut t, P1, "Counterspell", &[obj(div)]);
        t.resolve(); // Counterspell
        assert!(t.in_graveyard(P0, "Divination"), "{name}: countered");
        assert_eq!(t.stack_len(), 1, "{name}: the trigger is still there");
    }
    let hand = t.hand_size(P0);
    t.resolve(); // the trigger
    assert_eq!((t.life(P0), t.life(P1)), (21, 19), "{name}");
    if !countered {
        assert_eq!(
            t.zone(div),
            Zone::Stack,
            "{name}: the spell hasn't resolved yet"
        );
        assert_eq!(t.hand_size(P0), hand);
        t.resolve();
        assert_eq!(t.hand_size(P0), hand + 2);
    }
    if name == "Rusko, Clockmaker" {
        let clock = t.named_on_battlefield("Midnight Clock");
        assert_eq!(clock.len(), 1);
        assert_eq!(t.counters(clock[0], "hour"), 1);
    }
    let _ = source;
}

#[test]
fn papalymo_totolymo_trigger_resolves_before_the_spell() {
    cr!("603.3", "405.5", "701.6a");
    ruling!(
        "Papalymo Totolymo",
        "Papalymo Totolymo's first ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    for countered in [false, true] {
        resolves_before_the_spell(
            "Papalymo Totolymo",
            |t| t.battlefield(P0, "Papalymo Totolymo"),
            countered,
        );
    }
}

#[test]
fn rusko_trigger_resolves_before_the_spell() {
    cr!("603.3", "405.5", "701.6a");
    ruling!(
        "Rusko, Clockmaker",
        "Rusko’s last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    for countered in [false, true] {
        resolves_before_the_spell(
            "Rusko, Clockmaker",
            |t| {
                // Rusko enters and conjures a Midnight Clock onto the battlefield.
                let r = crate::r_s05_common::enter(t, P0, "Rusko, Clockmaker");
                t.resolve_all();
                assert_eq!(t.named_on_battlefield("Midnight Clock").len(), 1);
                r
            },
            countered,
        );
    }
}

// ---------------------------------------------------------------------------------------
// Craig Boone, Novac Guard
// ---------------------------------------------------------------------------------------

#[test]
fn craig_boone_reflexive_trigger_for_every_two_quest_counters() {
    cr!("603.12", "603.12a", "614.1a");
    interp!("reflexive-per-counters-put");
    ruling!(
        "Craig Boone, Novac Guard",
        "In the rare case where more than two quest counters are placed on Craig Boone because of the effect of its last ability (for example, because of the effect of Doubling Season), the reflexive triggered ability will trigger once for every two quest counters placed on Craig Boone this way."
    );
    supported("Craig Boone, Novac Guard");
    supported("Doubling Season");
    // P0 attacks with Craig Boone and Grizzly Bears; each reflexive trigger targets P1's
    // Colossal Dreadmaw (6/6), and P1 doesn't take the damage instead.
    for doubling in [false, true] {
        let mut t = TestGame::new(2);
        let boone = t.battlefield(P0, "Craig Boone, Novac Guard");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let dreadmaw = t.battlefield(P1, "Colossal Dreadmaw");
        if doubling {
            t.battlefield(P0, "Doubling Season");
        }
        attack_with(
            &mut t,
            &[(boone, Entity::Player(P1)), (bears, Entity::Player(P1))],
        );
        assert_eq!(triggers_named(&t, "Craig Boone, Novac Guard"), 1);
        let n = if doubling { 4 } else { 2 };
        t.answer_targets(P0, &[obj(dreadmaw)]);
        t.answer_targets(P0, &[obj(dreadmaw)]);
        t.answer_yes(P1, false);
        t.answer_yes(P1, false);
        t.resolve();
        assert_eq!(t.counters(boone, "quest"), n);
        let reflexive = n as usize / 2;
        assert_eq!(
            triggers_named(&t, "Craig Boone, Novac Guard"),
            reflexive,
            "Doubling Season: {doubling}"
        );
        // Each deals damage equal to the number of quest counters (lifelink).
        t.resolve_all();
        assert_eq!(t.life(P0), 20 + (n * reflexive as u32) as i32);
        if doubling {
            assert!(t.in_graveyard(P1, "Colossal Dreadmaw"), "4 + 4 damage");
        } else {
            assert_eq!(t.obj_now(dreadmaw).damage, 2);
        }
    }
}

#[test]
fn red_hulk_reflexive_trigger_once_for_every_counter_put() {
    cr!("603.12a", "614.1a");
    interp!("reflexive-per-counters-put");
    supported("Red Hulk");
    // "Whenever Red Hulk is dealt damage, put a +1/+1 counter on him. When you do, he deals
    // damage equal to the number of +1/+1 counters on him to any other target." P1 Shocks
    // Red Hulk; each reflexive trigger targets P1.
    for doubling in [false, true] {
        let mut t = TestGame::new(2);
        let hulk = t.battlefield(P0, "Red Hulk");
        if doubling {
            t.battlefield(P0, "Doubling Season");
        }
        cast_new(&mut t, P1, "Shock", &[obj(hulk)]);
        t.resolve();
        assert_eq!(triggers_named(&t, "Red Hulk"), 1);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.resolve();
        let n = if doubling { 2 } else { 1 };
        assert_eq!(t.counters(hulk, "+1/+1"), n);
        assert_eq!(triggers_named(&t, "Red Hulk"), n as usize);
        t.resolve_all();
        assert_eq!(
            t.life(P1),
            20 - (n * n) as i32,
            "Doubling Season: {doubling}"
        );
    }
}

// ---------------------------------------------------------------------------------------
// Mishra, Claimed by Gix
// ---------------------------------------------------------------------------------------

#[test]
fn mishra_lost_to_phyrexia_enters_attacking_without_attacking_triggers() {
    cr!("506.3", "508.3a", "701.42a");
    ruling!(
        "Mishra, Claimed by Gix",
        "Mishra, Lost to Phyrexia enters the battlefield tapped and attacking, but since it wasn't declared as an attacker, no abilities that trigger when a creature attacks will trigger."
    );
    supported("Mishra, Claimed by Gix");
    supported("Phyrexian Dragon Engine");
    // Mishra, Lost to Phyrexia: "Whenever Mishra enters or attacks, choose three —": it
    // triggers once, for entering.
    let mut t = TestGame::new(2);
    let mishra = t.battlefield(P0, "Mishra, Claimed by Gix");
    let engine = t.battlefield(P0, "Phyrexian Dragon Engine");
    attack_with(
        &mut t,
        &[(mishra, Entity::Player(P1)), (engine, Entity::Player(P1))],
    );
    assert_eq!(triggers_named(&t, "Mishra, Claimed by Gix"), 1);
    t.resolve();
    assert_eq!(t.life(P1), 18, "X = 2 attacking creatures");
    let lost = t.named_on_battlefield("Mishra, Lost to Phyrexia");
    assert_eq!(lost.len(), 1);
    let o = t.g.obj(lost[0]);
    assert!(o.tapped);
    let combat = t.g.combat.as_ref().unwrap();
    assert!(combat.attackers.iter().any(|a| a.id == lost[0]));
    assert_eq!(
        triggers_named(&t, "Mishra, Lost to Phyrexia"),
        1,
        "enters only"
    );
}

// ---------------------------------------------------------------------------------------
// Hostile Hostel // Creeping Inn
// ---------------------------------------------------------------------------------------

#[test]
fn hostile_hostel_phasing_doesnt_change_the_face() {
    cr!("702.26d", "702.26a", "712.8e");
    ruling!(
        "Hostile Hostel // Creeping Inn",
        "Phasing out or phasing in doesn't affect which face of a double-faced card is face up."
    );
    supported("Hostile Hostel // Creeping Inn");
    let mut t = TestGame::new(2);
    let hostel = t.battlefield(P0, "Hostile Hostel // Creeping Inn");
    // Two soul counters already; the third transforms it.
    put_counters(&mut t, hostel, "soul", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 5);
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, hostel, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(hostel).chars.name.as_str(), "Creeping Inn");
    // "{4}: This creature phases out."
    t.activate(P0, hostel, 0, &[]).unwrap();
    t.resolve_all();
    let inn = t.g.current(hostel);
    assert!(t.g.obj(inn).phased_out);
    assert_eq!(t.g.obj(inn).chars.name.as_str(), "Creeping Inn");
    // It phases in during P0's next untap step, still Creeping Inn.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.g.obj(inn).phased_out);
    t.advance_to(P0, Step::Upkeep);
    let inn = t.g.current(inn);
    assert!(!t.g.obj(inn).phased_out);
    assert_eq!(t.g.obj(inn).chars.name.as_str(), "Creeping Inn");
    assert!(t.g.obj(inn).is(mtg_engine::types::CardType::Creature));
}

// ---------------------------------------------------------------------------------------
// Sorin of House Markov // Sorin, Ravenous Neonate
// ---------------------------------------------------------------------------------------

const SORIN: &str = "Sorin of House Markov // Sorin, Ravenous Neonate";

/// P0 casts Sorin of House Markov ({1}{B}) from hand.
fn cast_sorin(t: &mut TestGame) {
    cast_new(t, P0, SORIN, &[]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Sorin of House Markov").len(), 1);
}

/// Moves from P0's precombat main phase to the start of its postcombat main phase (the
/// triggers there put on the stack).
fn into_postcombat_main(t: &mut TestGame) {
    t.advance_to(P0, Step::PostcombatMain);
    t.settle();
}

#[test]
fn sorin_of_house_markov_needn_t_have_been_there_when_you_gained_life() {
    cr!("603.4", "712.10");
    ruling!(
        "Sorin of House Markov // Sorin, Ravenous Neonate",
        "Sorin of House Markov doesn't need to have been on the battlefield when you gained the life. For example, if you gained 3 or more life during your upkeep and you cast Sorin of House Markov during your first main phase, its last ability will trigger at the beginning of your postcombat main phase."
    );
    supported(SORIN);
    let mut t = TestGame::new(2);
    // P0 gains 3 life (Healing Salve) during its upkeep, then casts Sorin in its first main
    // phase.
    t.set_step(P0, Step::Upkeep);
    let salve = crate::r_s25_common::lands_for_cost(&mut t, P0, "Healing Salve");
    let _ = salve;
    let c = t.hand(P0, "Healing Salve");
    t.cast(P0, c).modes(&[0]).target(P0).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    t.advance_to(P0, Step::PrecombatMain);
    cast_sorin(&mut t);
    into_postcombat_main(&mut t);
    assert_eq!(triggers_named(&t, "Sorin of House Markov"), 1);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Sorin, Ravenous Neonate").len(), 1);
}

#[test]
fn sorin_of_house_markov_triggers_once_and_only_if_life_was_already_gained() {
    cr!("603.4", "505.1");
    ruling!(
        "Sorin of House Markov // Sorin, Ravenous Neonate",
        "Sorin of House Markov's last ability will trigger only once during your postcombat main phase, no matter how much life you gained this turn. However, if you haven't gained life so far this turn as your postcombat main phase begins, the ability won't trigger at all. It's not possible to gain life during your postcombat main phase in time for the ability to trigger."
    );
    supported(SORIN);
    // Gained 12 life (two Renewed Faiths: "You gain 6 life.") before: one trigger.
    let mut t = TestGame::new(2);
    cast_sorin(&mut t);
    for _ in 0..2 {
        cast_new(&mut t, P0, "Renewed Faith", &[]);
        t.resolve_all();
    }
    assert_eq!(t.life(P0), 32);
    into_postcombat_main(&mut t);
    assert_eq!(triggers_named(&t, "Sorin of House Markov"), 1);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Sorin, Ravenous Neonate").len(), 1);
    // No life gained before the postcombat main phase: no trigger, and gaining life
    // during it is too late.
    let mut t = TestGame::new(2);
    cast_sorin(&mut t);
    into_postcombat_main(&mut t);
    assert_eq!(t.stack_len(), 0);
    cast_new(&mut t, P0, "Renewed Faith", &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Sorin of House Markov").len(), 1);
    assert!(t.named_on_battlefield("Sorin, Ravenous Neonate").is_empty());
}

#[test]
fn sorin_ravenous_neonate_counts_life_gained_ignoring_life_lost() {
    cr!("119.3", "608.2c");
    ruling!(
        "Sorin of House Markov // Sorin, Ravenous Neonate",
        "Sorin, Ravenous Neonate's second loyalty ability counts the total amount of life you gained without taking into account any life you lost during that turn. For example, if you gained 3 life and lost 3 life earlier in the turn, Sorin, Ravenous Neonate will deal 3 damage to the target."
    );
    supported(SORIN);
    let mut t = TestGame::new(2);
    cast_sorin(&mut t);
    // P0 gains 3 life (Healing Salve), then P1's Lightning Bolt makes P0 lose 3.
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Healing Salve");
    let c = t.hand(P0, "Healing Salve");
    t.cast(P0, c).modes(&[0]).target(P0).go();
    t.resolve_all();
    cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    into_postcombat_main(&mut t);
    t.resolve_all();
    let neonate = t.named_on_battlefield("Sorin, Ravenous Neonate");
    assert_eq!(neonate.len(), 1);
    // −1: "Sorin deals damage equal to the amount of life you gained this turn to any
    // target."
    let from = t.asked().len();
    t.activate(P0, neonate[0], 1, &[Entity::Player(P1)])
        .unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    let _ = from;
}

// ---------------------------------------------------------------------------------------
// Survey Mechan
// ---------------------------------------------------------------------------------------

/// Seven differently named lands for P0, five of them tapped: an untapped Island and an
/// untapped Crystal Vein ("{T}, Sacrifice this land: Add {C}{C}."). Survey Mechan's last
/// ability costs {10} − 7 = {3}, which only the Island and the sacrificed Vein can pay.
fn mechan() -> (TestGame, ObjectId, ObjectId) {
    supported("Survey Mechan");
    supported("Crystal Vein");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Survey Mechan");
    for name in ["Plains", "Swamp", "Mountain", "Forest", "Wastes"] {
        let l = t.battlefield(P0, name);
        assert!(t.g.tap(l));
    }
    t.battlefield(P0, "Island");
    let vein = t.battlefield(P0, "Crystal Vein");
    (t, m, vein)
}

#[test]
fn survey_mechan_cost_stays_after_sacrificing_a_land_for_mana() {
    cr!("602.2b", "601.2f", "601.2g", "601.2h");
    ruling!(
        "Survey Mechan",
        "Once you determine the cost to activate Survey Mechan’s last ability, you may activate mana abilities to pay that cost. If the number of differently named lands you control changes while activating mana abilities (probably because you sacrificed one or more lands), the cost to activate the ability remains what you previously determined."
    );
    let (mut t, m, vein) = mechan();
    let hand = t.hand_size(P1);
    t.activate(P0, m, 0, &[Entity::Player(P1), Entity::Player(P1)])
        .unwrap();
    assert!(!t.on_battlefield(vein), "sacrificed for mana");
    assert!(!t.on_battlefield(m));
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    t.resolve_all();
    // 3 damage, then P1 draws three and gains 3.
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.hand_size(P1), hand + 3);
}

#[test]
fn survey_mechan_nobody_acts_while_it_is_activated() {
    cr!("602.2", "602.2b");
    ruling!(
        "Survey Mechan",
        "Once you announce you’re activating an ability, no player may take actions until after the ability has been paid for. Notably, opponents can’t try to remove lands you control from the battlefield at that time."
    );
    let (mut t, m, _) = mechan();
    let from = t.asked().len();
    t.activate(P0, m, 0, &[Entity::Player(P1), Entity::Player(P1)])
        .unwrap();
    let others: Vec<(PlayerId, Decision)> = t.asked()[from..]
        .iter()
        .filter(|(p, d)| *p != P0 || matches!(d, Decision::Priority { .. }))
        .cloned()
        .collect();
    assert!(others.is_empty(), "{others:?}");
    assert_eq!(t.stack_len(), 1);
}
