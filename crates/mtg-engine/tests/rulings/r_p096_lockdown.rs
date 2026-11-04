//! Rulings batch P096 — "doesn't untap during its controller's untap step" (CR 502.3):
//! for as long as you control the source (CR 611.2b: a duration that ended before the
//! effect began does nothing), for as long as the source remains tapped, Auras that need a
//! legal target to enter (CR 608.3b, 303.4f) and paying to untap.

use crate::r_p096_common::*;
use crate::r_s01_common::supported;
use crate::r_s25_common::cast_new;
use crate::r_s29_common::cast_and_resolve;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// From P0's turn, advances to P1's next upkeep (through P1's untap step).
fn to_p1_upkeep(t: &mut TestGame) {
    if t.g.turn.active == P1 {
        t.advance_to(P0, Step::Upkeep);
    }
    t.advance_to(P1, Step::Upkeep);
}

fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

// ---------------------------------------------------------------------------------------
// "... for as long as you control this creature"
// ---------------------------------------------------------------------------------------

/// The four "for as long as you control this creature" sources: puts `name` onto P0's
/// battlefield and makes its ability trigger targeting `target` (P1's creature); the
/// trigger is left on the stack. Returns the source.
fn lock_with(t: &mut TestGame, name: &str, target: ObjectId) -> ObjectId {
    lock_all_with(t, name, &[target])
}

/// [`lock_with`] with several targets (Shipbreaker Kraken's "up to four").
fn lock_all_with(t: &mut TestGame, name: &str, targets: &[ObjectId]) -> ObjectId {
    supported(name);
    let targets: Vec<Entity> = targets.iter().map(|o| Entity::Object(*o)).collect();
    t.answer_targets(P0, &targets);
    if name == "Shipbreaker Kraken" {
        // "{6}{U}{U}: Monstrosity 4. ... When this creature becomes monstrous, tap up to
        // four target creatures. Those creatures don't untap during their controllers'
        // untap steps for as long as you control this creature."
        let kraken = t.battlefield(P0, name);
        t.lands(P0, "Island", 2);
        t.lands(P0, "Wastes", 6);
        t.activate(P0, kraken, 0, &[]).unwrap();
        t.resolve();
        t.settle();
        assert_eq!(triggers_from(t, kraken), 1);
        return kraken;
    }
    // Dungeon Geists, Icefall Regent: "When this creature enters, tap target creature an
    // opponent controls. That creature doesn't untap during its controller's untap step
    // for as long as you control this creature." Merchant Raiders: "Whenever this
    // creature or another Pirate you control enters, tap up to one target creature. ..."
    let c = crate::r_s05_common::enter(t, P0, name);
    assert_eq!(triggers_from(t, c), 1, "{name}");
    c
}

const AS_LONG_AS_YOU_CONTROL: [&str; 4] = [
    "Dungeon Geists",
    "Icefall Regent",
    "Merchant Raiders",
    "Shipbreaker Kraken",
];

#[test]
fn lock_ends_when_another_player_gains_control_even_if_you_regain_it() {
    cr!("611.2b", "502.3", "613.1b");
    ruling!(
        "Dungeon Geists",
        "If another player gains control of Dungeon Geists, its effect expires. It won’t keep the creature from untapping anymore, even if you later regain control of Dungeon Geists."
    );
    ruling!(
        "Icefall Regent",
        "If another player gains control of Icefall Regent, the creature will no longer be affected by the ability stopping it from untapping, even if you later regain control of Icefall Regent."
    );
    ruling!(
        "Merchant Raiders",
        "If another player gains control of Merchant Raiders, the creature will no longer be affected by the ability stopping it from untapping, even if you later regain control of Merchant Raiders."
    );
    ruling!(
        "Shipbreaker Kraken",
        "If another player gains control of Shipbreaker Kraken, the creatures will no longer be affected by the effect preventing them from untapping, even if you later regain control of Shipbreaker Kraken."
    );
    for name in AS_LONG_AS_YOU_CONTROL {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let source = lock_with(&mut t, name, bears);
        t.resolve_all();
        assert!(tapped(&t, bears), "{name}");
        // P1's untap step: still tapped.
        to_p1_upkeep(&mut t);
        assert!(tapped(&t, bears), "{name}");
        // P1 Threatens the source ("Untap target creature and gain control of it until end
        // of turn."); it goes back to P0 at end of turn.
        t.advance_to(P1, Step::PrecombatMain);
        crate::r_s25_common::lands_for_cost(&mut t, P1, "Threaten");
        // (Icefall Regent: "Spells your opponents cast that target this creature cost {2}
        // more to cast.")
        t.lands(P1, "Wastes", 2);
        let threaten = t.hand(P1, "Threaten");
        t.cast_with(P1, threaten, &[Entity::Object(source)])
            .unwrap_or_else(|e| panic!("{name}: {e:?}\n{}", t.dump_log()));
        t.resolve_all();
        assert_eq!(t.obj_now(source).controller, P1, "{name}");
        t.advance_to(P0, Step::Upkeep);
        assert_eq!(t.obj_now(source).controller, P0, "{name}");
        // P1's next untap step: the Bears untap.
        to_p1_upkeep(&mut t);
        assert!(!tapped(&t, bears), "{name}");
    }
}

#[test]
fn lock_can_target_a_tapped_creature() {
    cr!("502.3", "701.26a", "115.1d");
    ruling!(
        "Dungeon Geists",
        "Dungeon Geists’s triggered ability can target a tapped creature. If the targeted creature is already tapped when it resolves, that creature just remains tapped and won’t untap during its controller’s untap step."
    );
    ruling!(
        "Icefall Regent",
        "Icefall Regent’s triggered ability can target any creature controlled by an opponent, including one that is already tapped. In that case, the creature stays tapped and won’t untap during its controller’s untap step."
    );
    for name in ["Dungeon Geists", "Icefall Regent"] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.g.tap(bears);
        t.g.flush_events();
        let from = t.g.turn_events.len();
        lock_with(&mut t, name, bears);
        t.resolve_all();
        // (It wasn't tapped again: it stayed tapped.)
        assert!(t.g.turn_events[from..]
            .iter()
            .all(|e| !matches!(e, Event::Tapped { .. })));
        to_p1_upkeep(&mut t);
        assert!(tapped(&t, bears), "{name}");
    }
}

#[test]
fn dungeon_geists_leaving_before_its_trigger_resolves() {
    cr!("611.2b", "608.2b");
    ruling!(
        "Dungeon Geists",
        "If Dungeon Geists leaves the battlefield before its triggered ability has resolved, the target creature will be tapped, but it will be able to untap as normal."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let geists = lock_with(&mut t, "Dungeon Geists", bears);
    cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Object(geists)]);
    t.resolve();
    assert!(!t.on_battlefield(geists));
    t.resolve_all();
    assert!(tapped(&t, bears));
    to_p1_upkeep(&mut t);
    assert!(!tapped(&t, bears));
}

#[test]
fn shipbreaker_kraken_losing_control_before_its_trigger_resolves() {
    cr!("611.2b", "608.2b");
    ruling!(
        "Shipbreaker Kraken",
        "If you lose control of Shipbreaker Kraken between when its last ability triggers and when that ability resolves, the target creatures will become tapped. However, they won't be affected by the other effect and will untap as normal during their controllers' untap steps."
    );
    supported("Ray of Command");
    // P1 responds with Ray of Command ("Untap target creature an opponent controls and gain
    // control of it until end of turn. ...").
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let kraken = lock_all_with(&mut t, "Shipbreaker Kraken", &[bears, giant]);
    cast_new(&mut t, P1, "Ray of Command", &[Entity::Object(kraken)]);
    t.resolve();
    assert_eq!(t.obj_now(kraken).controller, P1);
    t.resolve_all();
    assert!(tapped(&t, bears) && tapped(&t, giant));
    to_p1_upkeep(&mut t);
    assert_eq!(t.obj_now(kraken).controller, P0);
    assert!(!tapped(&t, bears) && !tapped(&t, giant));
}

#[test]
fn shipbreaker_kraken_a_locked_creature_you_gain_control_of_stays_locked() {
    cr!("502.3", "613.1b");
    ruling!(
        "Shipbreaker Kraken",
        "If you gain control of a creature that's being prevented from untapping, that creature won't untap during your untap step for as long as you control Shipbreaker Kraken."
    );
    supported("Mind Control");
    // P0 locks P1's Bears, then takes them with Mind Control ("You control enchanted
    // creature."): P0's untap step doesn't untap them while P0 controls the Kraken.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let kraken = lock_with(&mut t, "Shipbreaker Kraken", bears);
    t.resolve_all();
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 3);
    cast_and_resolve(&mut t, P0, "Mind Control", &[Entity::Object(bears)]);
    assert_eq!(t.obj_now(bears).controller, P0);
    t.advance_to(P0, Step::Upkeep);
    assert!(tapped(&t, bears));
    // Once P0 no longer controls the Kraken (Unsummon), the Bears untap in P0's next untap
    // step.
    cast_and_resolve(&mut t, P0, "Unsummon", &[Entity::Object(kraken)]);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(!tapped(&t, bears));
}

// ---------------------------------------------------------------------------------------
// "... for as long as this artifact remains tapped"
// ---------------------------------------------------------------------------------------

/// P0's Entrancing Lyre ("{X}, {T}: Tap target creature with power X or less. It doesn't
/// untap during its controller's untap step for as long as this artifact remains tapped.")
/// activated with X = 2 targeting `target`; the ability is left on the stack.
fn lyre(t: &mut TestGame, target: ObjectId) -> ObjectId {
    supported("Entrancing Lyre");
    let lyre = t.battlefield(P0, "Entrancing Lyre");
    t.lands(P0, "Wastes", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.activate(P0, lyre, 0, &[Entity::Object(target)]).unwrap();
    lyre
}

#[test]
fn entrancing_lyre_lock_lasts_while_it_stays_tapped() {
    cr!("611.2b", "502.3", "115.1b");
    ruling!(
        "Entrancing Lyre",
        "Entrancing Lyre’s activated ability can target a creature that’s already tapped. That creature won’t untap for as long as Entrancing Lyre remains tapped."
    );
    ruling!(
        "Entrancing Lyre",
        "If the target creature’s power is increased after Entrancing Lyre’s ability has resolved, it remains affected and won’t untap as usual."
    );
    ruling!(
        "Entrancing Lyre",
        "Once Entrancing Lyre is no longer tapped, the creature won’t immediately become untapped."
    );
    supported("Voltaic Key");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    let l = lyre(&mut t, bears);
    t.resolve_all();
    assert!(tapped(&t, bears));
    // Giant Growth makes the Bears 5/5: no longer a legal target, but still affected.
    cast_and_resolve(&mut t, P1, "Giant Growth", &[Entity::Object(bears)]);
    to_p1_upkeep(&mut t);
    assert!(tapped(&t, bears));
    // P1 untaps the Lyre with Voltaic Key ("{1}, {T}: Untap target artifact."): the Bears
    // stay tapped until P1's next untap step.
    let key = t.battlefield(P1, "Voltaic Key");
    t.lands(P1, "Wastes", 1);
    t.activate(P1, key, 0, &[Entity::Object(l)]).unwrap();
    t.resolve_all();
    assert!(!tapped(&t, l));
    assert!(tapped(&t, bears));
    t.advance_to(P0, Step::Upkeep);
    assert!(tapped(&t, bears));
    to_p1_upkeep(&mut t);
    assert!(!tapped(&t, bears));
}

#[test]
fn entrancing_lyre_untapped_before_its_ability_resolves() {
    cr!("611.2b", "608.2b");
    ruling!(
        "Entrancing Lyre",
        "If Entrancing Lyre becomes untapped before its activated ability resolves, the target creature is tapped, but it can untap as usual. This is true even if Entrancing Lyre becomes tapped again before the activated ability resolves."
    );
    // In response, P0 untaps the Lyre with Voltaic Key and activates it again targeting the
    // Hill Giant (X = 3), tapping it again. The second ability locks the Giant; the first
    // only taps the Bears.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let l = lyre(&mut t, bears);
    let key = t.battlefield(P0, "Voltaic Key");
    t.lands(P0, "Wastes", 4);
    t.activate(P0, key, 0, &[Entity::Object(l)]).unwrap();
    t.resolve();
    assert!(!tapped(&t, l));
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.activate(P0, l, 0, &[Entity::Object(giant)]).unwrap();
    assert!(tapped(&t, l));
    t.resolve_all();
    assert!(tapped(&t, bears) && tapped(&t, giant));
    to_p1_upkeep(&mut t);
    assert!(!tapped(&t, bears));
    assert!(tapped(&t, giant));
}

#[test]
fn entrancing_lyre_leaving_the_battlefield_ends_the_lock() {
    cr!("611.2b", "502.3");
    ruling!(
        "Entrancing Lyre",
        "If Entrancing Lyre leaves the battlefield, it’s no longer tapped, so the creature is free to untap as usual."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let l = lyre(&mut t, bears);
    t.resolve_all();
    cast_and_resolve(&mut t, P1, "Disenchant", &[Entity::Object(l)]);
    assert!(tapped(&t, bears));
    to_p1_upkeep(&mut t);
    assert!(!tapped(&t, bears));
}

#[test]
fn entrancing_lyre_illegal_target_doesnt_tap() {
    cr!("608.2b", "115.1b");
    ruling!(
        "Entrancing Lyre",
        "If the target creature is an illegal target by the time Entrancing Lyre’s ability tries to resolve, the ability won’t resolve. The creature isn’t tapped and can untap as usual."
    );
    // In response, Giant Growth makes the Bears 5/5: power more than X = 2.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    lyre(&mut t, bears);
    cast_new(&mut t, P1, "Giant Growth", &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(!tapped(&t, bears));
    // Tapped some other way later (Pressure Point: "Tap target creature."), it untaps as
    // usual.
    cast_and_resolve(&mut t, P0, "Pressure Point", &[Entity::Object(bears)]);
    to_p1_upkeep(&mut t);
    assert!(!tapped(&t, bears));
}

#[test]
fn deserters_quarters_lock_survives_the_creature_untapping_another_way() {
    cr!("611.2b", "502.3");
    ruling!(
        "Deserter's Quarters",
        "As long as Deserter's Quarters remains tapped, the effect stopping the creature from untapping during its controller's untap step remains even if that creature untaps some other way (for example, with Colossal Heroics)."
    );
    supported("Deserter's Quarters");
    supported("Colossal Heroics");
    // "{6}, {T}: Tap target creature. It doesn't untap during its controller's untap step
    // for as long as this artifact remains tapped." P1 untaps the Bears with Colossal
    // Heroics ("Any number of target creatures each get +2/+2 until end of turn. Untap
    // those creatures."); P0 taps them again with Pressure Point.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let quarters = t.battlefield(P0, "Deserter's Quarters");
    t.lands(P0, "Wastes", 6);
    t.activate(P0, quarters, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert!(tapped(&t, bears));
    cast_and_resolve(&mut t, P1, "Colossal Heroics", &[Entity::Object(bears)]);
    assert!(!tapped(&t, bears));
    cast_and_resolve(&mut t, P0, "Pressure Point", &[Entity::Object(bears)]);
    assert!(tapped(&t, bears));
    to_p1_upkeep(&mut t);
    assert!(tapped(&t, bears));
}

// ---------------------------------------------------------------------------------------
// Auras
// ---------------------------------------------------------------------------------------

#[test]
fn lockdown_auras_can_enchant_tapped_or_untapped_creatures() {
    cr!("502.3", "303.4a");
    ruling!(
        "Castaway's Despair",
        "Castaway's Despair can target a creature that's already tapped."
    );
    ruling!(
        "Paralyze",
        "Paralyze can be cast targeting a creature which is already tapped."
    );
    ruling!(
        "Containment Membrane",
        "Containment Membrane can target and be attached to an untapped creature. It just won't have any effect until that creature becomes tapped."
    );
    // Castaway's Despair, Paralyze: "When this Aura enters, tap enchanted creature.
    // Enchanted creature doesn't untap during its controller's untap step."
    for name in ["Castaway's Despair", "Paralyze"] {
        supported(name);
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.g.tap(bears);
        let aura = cast_new(&mut t, P0, name, &[Entity::Object(bears)]);
        t.resolve_all();
        assert_eq!(t.obj_now(aura).attached_to, Some(Entity::Object(bears)), "{name}");
        to_p1_upkeep(&mut t);
        assert!(tapped(&t, bears), "{name}");
    }
    // Containment Membrane: "Enchanted creature doesn't untap during its controller's
    // untap step." On untapped Bears, nothing happens until they're tapped: P1 attacks
    // with them, and they stay tapped in P1's next untap step.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = cast_new(&mut t, P0, "Containment Membrane", &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.obj_now(aura).attached_to, Some(Entity::Object(bears)));
    to_p1_upkeep(&mut t);
    assert!(!tapped(&t, bears));
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P0))]),
    );
    t.advance_to(P1, Step::EndOfCombat);
    assert!(tapped(&t, bears));
    to_p1_upkeep(&mut t);
    assert!(tapped(&t, bears));
}

#[test]
fn lockdown_auras_with_an_illegal_target_dont_enter() {
    cr!("608.3b", "608.2b", "303.4f");
    ruling!(
        "Sleep with the Fishes",
        "If the creature targeted by Sleep with the Fishes isn't a legal target at the time it would resolve, Sleep with the Fishes is put into its owner's graveyard from the stack. It's not put onto the battlefield and then put into their graveyard. Notably, this means that its controller will not create a Fish token."
    );
    ruling!(
        "Fall from Favor",
        "If the target creature is an illegal target by the time Fall from Favor tries to resolve, it doesn't resolve. It won't enter the battlefield, so its enters-the-battlefield ability won't trigger."
    );
    ruling!(
        "Sleep Paralysis",
        "If the target creature is an illegal target by the time Sleep Paralysis tries to resolve, the spell doesn't resolve. It won't enter the battlefield, so its ability won't trigger."
    );
    // In response, P1 returns the Bears to their hand with Unsummon.
    for name in ["Sleep with the Fishes", "Fall from Favor", "Sleep Paralysis"] {
        supported(name);
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        cast_new(&mut t, P0, name, &[Entity::Object(bears)]);
        let from = t.g.turn_events.len();
        cast_new(&mut t, P1, "Unsummon", &[Entity::Object(bears)]);
        t.resolve_all();
        assert!(t.in_graveyard(P0, name), "{name}");
        assert!(t.g.turn_events[from..].iter().all(|e| !matches!(
            e,
            Event::ZoneChange { to: mtg_engine::object::Zone::Battlefield, .. }
                | Event::AbilityTriggeredOnStack { .. }
        )));
        assert_eq!(crate::r_s25_common::creature_tokens(&t, P0), 0, "{name}");
        assert_eq!(t.g.monarch, None, "{name}");
    }
}

#[test]
fn paralyze_paying_to_untap_is_optional() {
    cr!("603.5", "502.3");
    ruling!("Paralyze", "Paying the cost is optional.");
    // "At the beginning of the upkeep of enchanted creature's controller, that player may
    // pay {4}. If the player does, untap the creature." P1 is asked (not P0): declining
    // leaves the Bears tapped; paying untaps them.
    for pay in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.lands(P1, "Wastes", 4);
        cast_new(&mut t, P0, "Paralyze", &[Entity::Object(bears)]);
        t.resolve_all();
        assert!(tapped(&t, bears));
        to_p1_upkeep(&mut t);
        t.settle();
        let from = t.asked().len();
        t.answer_yes(P1, pay);
        t.resolve_all();
        let asked: Vec<PlayerId> = t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
            .map(|(p, _)| *p)
            .collect();
        assert_eq!(asked, vec![P1]);
        assert_eq!(tapped(&t, bears), !pay);
        assert_eq!(crate::r_s01_common::tapped_lands(&t, P1), if pay { 4 } else { 0 });
    }
}

// ---------------------------------------------------------------------------------------
// Mudslide, Intruder Alarm
// ---------------------------------------------------------------------------------------

#[test]
fn mudslide_chooses_all_creatures_then_pays_all_at_once() {
    cr!("601.2h", "118.3", "608.2c");
    ruling!(
        "Mudslide",
        "First the player chooses all the tapped creatures they would like to untap this way. Then the entire payment is made at once, then all of those creatures untap at the same time. If the entire payment can’t be paid (because the player chose too many creatures, for example), none of it is paid and none of those creatures untap."
    );
    supported("Mudslide");
    // "Creatures without flying don't untap during their controllers' untap steps. At the
    // beginning of each player's upkeep, that player may choose any number of tapped
    // creatures without flying they control and pay {2} for each creature chosen this
    // way. If the player does, untap those creatures." P1 has three tapped Bears and four
    // lands.
    for chosen in [2usize, 3] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Mudslide");
        let bears: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P1, "Grizzly Bears")).collect();
        for b in &bears {
            t.g.tap(*b);
        }
        t.lands(P1, "Wastes", 4);
        let seen: crate::r_s01_common::Seen<(usize, usize)> = Default::default();
        let s = seen.clone();
        let ids = bears.clone();
        t.g.observer = Some(mtg_engine::game::EventObserver::new(move |g, ev| {
            if matches!(ev, Event::Untapped { obj } if ids.contains(obj)) {
                let tapped_bears = ids.iter().filter(|b| g.obj(**b).tapped).count();
                let tapped_lands = g
                    .permanents()
                    .filter(|o| o.controller == P1 && o.is(mtg_engine::types::CardType::Land) && o.tapped)
                    .count();
                s.lock().unwrap().push((tapped_bears, tapped_lands));
            }
        }));
        let picks: Vec<Entity> = bears[..chosen].iter().map(|b| Entity::Object(*b)).collect();
        t.answer_choose(P1, &picks);
        t.answer_yes(P1, true);
        to_p1_upkeep(&mut t);
        t.settle();
        t.resolve_all();
        let still_tapped = bears.iter().filter(|b| tapped(&t, **b)).count();
        if chosen == 2 {
            // Both untapped at once, after all {4} was paid.
            assert_eq!(still_tapped, 1);
            assert_eq!(*seen.lock().unwrap(), vec![(1, 4), (1, 4)]);
        } else {
            // {6} can't be paid: nothing paid, nothing untapped.
            assert_eq!(still_tapped, 3);
            assert_eq!(crate::r_s01_common::tapped_lands(&t, P1), 0);
            assert!(seen.lock().unwrap().is_empty());
        }
    }
}

#[test]
fn intruder_alarm_triggers_for_each_creature_entering_at_once() {
    cr!("603.2c", "603.2");
    ruling!(
        "Intruder Alarm",
        "If multiple creatures enter the battlefield at one time, this ability triggers once for each of those creatures."
    );
    // "Whenever a creature enters, untap all creatures." Raise the Alarm: "Create two 1/1
    // white Soldier creature tokens."
    let mut t = TestGame::new(2);
    let alarm = t.battlefield(P0, "Intruder Alarm");
    cast_new(&mut t, P0, "Raise the Alarm", &[]);
    t.resolve();
    t.settle();
    assert_eq!(triggers_from(&t, alarm), 2);
}
