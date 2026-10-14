//! Rulings batch P117 — abilities that compare powers among creatures ("the creature with
//! the greatest power", coven, "power 3 or greater enters", evolve-like comparisons):
//! conditions checked as triggers trigger and again as they resolve (CR 603.4), values
//! determined on resolution (CR 608.2h), base power (CR 613.4b), and last known
//! information (CR 608.2h).

use crate::r_p117_common::*;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::decision::Decision;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// --- Primal Empathy ---------------------------------------------------------------------

/// Advances to P0's next upkeep with Primal Empathy's trigger on the stack.
fn empathy_upkeep(t: &mut TestGame) {
    t.battlefield(P0, "Primal Empathy");
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(stack(t), 1);
}

#[test]
fn primal_empathy_with_no_creatures_does_nothing() {
    cr!("608.2c");
    ruling!(
        "Primal Empathy",
        "If you control no creatures, you do nothing as Primal Empathy's ability resolves. If there are no creatures on the battlefield at all, you don't draw a card."
    );
    supported("Primal Empathy");
    // No creatures at all.
    let mut t = TestGame::new(2);
    empathy_upkeep(&mut t);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // Only an opponent controls a creature.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    empathy_upkeep(&mut t);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(plus1(&t, bears), 0);
}

#[test]
fn primal_empathy_you_choose_the_creature_as_it_resolves() {
    cr!("608.2c", "608.2d");
    ruling!(
        "Primal Empathy",
        "If you don't control a creature with the greatest power, you choose which creature gets a +1/+1 counter while the ability is resolving."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P1, "Hill Giant");
    empathy_upkeep(&mut t);
    t.answer_choose(P0, &[obj(elves)]);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(plus1(&t, elves), 1);
}

#[test]
fn primal_empathy_decides_only_as_it_resolves() {
    cr!("608.2c", "603.3");
    ruling!(
        "Primal Empathy",
        "You determine whether you draw a card or put a +1/+1 counter only as Primal Empathy's triggered ability begins resolving. Players may respond to the ability to try to change which happens."
    );
    // P0's Bears are the greatest as it triggers; P1 pumps the Elves in response.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    empathy_upkeep(&mut t);
    cast_new(&mut t, P1, "Giant Growth", &[obj(elves)]);
    t.resolve();
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand, "no draw");
    assert_eq!(plus1(&t, bears), 1);
}

// --- Duel for Dominance -------------------------------------------------------------------

#[test]
fn duel_for_dominance_still_puts_the_counter_if_the_other_target_is_illegal() {
    cr!("608.2b", "701.14b");
    ruling!(
        "Duel for Dominance",
        "If you control three or more creatures with different powers and the target creature you control is still a legal target as Duel for Dominance resolves but the target creature you don't control isn't, you will still put a +1/+1 counter on the creature you control."
    );
    supported("Duel for Dominance");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P0, "Hill Giant");
    let air = t.battlefield(P1, "Air Elemental");
    cast_new(&mut t, P0, "Duel for Dominance", &[obj(bears), obj(air)]);
    exile(&mut t, air);
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 1);
    assert_eq!(damage(&t, bears), 0);
}

// --- Parish-Blade Trainee -------------------------------------------------------------------

/// Parish-Blade Trainee with a +1/+1 counter and a flying counter dies, targeting P0's
/// Grizzly Bears; with The Ozolith if `ozolith`. Returns (Bears, Ozolith) counters as
/// (+1/+1, flying) pairs.
fn trainee_dies(ozolith: bool) -> ((u32, u32), (u32, u32)) {
    let mut t = TestGame::new(2);
    let oz = ozolith.then(|| t.battlefield(P0, "The Ozolith"));
    let trainee = t.battlefield(P0, "Parish-Blade Trainee");
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_counters(&mut t, trainee, "+1/+1", 1);
    put_counters(&mut t, trainee, "flying", 1);
    t.answer_targets(P0, &[obj(bears)]);
    destroy(&mut t, trainee);
    t.resolve_all();
    let c = |id: ObjectId| (t.counters(id, "+1/+1"), t.counters(id, "flying"));
    (c(bears), oz.map_or((0, 0), c))
}

#[test]
fn parish_blade_trainee_puts_all_kinds_of_counters() {
    cr!("603.10a", "122.1");
    ruling!(
        "Parish-Blade Trainee",
        "Parish-Blade Trainee's last ability puts all of its counters onto the target creature, not just its +1/+1 counters."
    );
    supported("Parish-Blade Trainee");
    assert_eq!(trainee_dies(false).0, (1, 1));
}

#[test]
fn parish_blade_trainee_puts_the_same_counters_rather_than_moving_them() {
    cr!("603.10a", "122.1");
    ruling!(
        "Parish-Blade Trainee",
        "Parish-Blade Trainee's last ability doesn't cause you to move counters from itself to the target creature. Rather, you put the same number of each kind of counter Parish-Blade Trainee had when it died onto that creature."
    );
    ruling!(
        "Parish-Blade Trainee",
        "In some unusual cases, you may end up putting the appropriate counters on more than one permanent. For example, if you control The Ozolith when Parish-Blade Trainee dies, you'll put the appropriate number of each kind of counter onto both The Ozolith and the target creature."
    );
    supported("The Ozolith");
    assert_eq!(trainee_dies(true), ((1, 1), (1, 1)));
}

// --- Drop of Honey and Porphyry Nodes ----------------------------------------------------

#[test]
fn drop_of_honey_and_porphyry_nodes_destroy_the_chosen_tied_creature() {
    cr!("608.2c", "117.2e");
    ruling!(
        "Drop of Honey",
        "No player can take actions between the time you choose which creature to destroy and the time it's destroyed."
    );
    ruling!(
        "Porphyry Nodes",
        "No player can take actions between the time you choose which creature to destroy and the time it's destroyed."
    );
    for name in ["Drop of Honey", "Porphyry Nodes"] {
        supported(name);
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        let a = t.battlefield(P0, "Llanowar Elves");
        let b = t.battlefield(P1, "Llanowar Elves");
        t.battlefield(P1, "Grizzly Bears");
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        t.answer_choose(P0, &[obj(b)]);
        let from = n_asked(&t);
        t.g.resolve_top();
        // The choice was made and the creature destroyed within the resolution.
        assert!(!t.g.is_live(b), "{name}");
        assert!(t.g.is_live(a), "{name}");
        assert!(!crate::r_p116_common::priority_asked_since(&t, from));
    }
}

// --- Baird, Argivian Recruiter --------------------------------------------------------------

#[test]
fn baird_compares_with_base_power() {
    cr!("613.4b", "613.4c");
    ruling!(
        "Baird, Argivian Recruiter",
        "If another effect sets a creature’s power and toughness to specific numbers or values, those become its base power and toughness. If an effect modifies a creature’s power and/or toughness without setting them, that is not included when determining its base power and toughness."
    );
    supported("Baird, Argivian Recruiter");
    // (setup, whether a Soldier is created)
    let cases: [(fn(&mut TestGame, ObjectId), bool); 4] = [
        (|_, _| {}, false),
        (|t, b| give_plus1(t, b, 1), true),
        (|t, b| giant_growth(t, b), true),
        (
            |t, b| {
                modify(
                    t,
                    b,
                    vec![Modification::SetPT(Some(Value::c(5)), Some(Value::c(5)))],
                )
            },
            false,
        ),
    ];
    for (i, (setup, token)) in cases.into_iter().enumerate() {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Baird, Argivian Recruiter");
        let bears = t.battlefield(P0, "Grizzly Bears");
        setup(&mut t, bears);
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert_eq!(tokens_named(&t, P0, "Soldier") == 1, token, "case {i}");
    }
}

// --- O-Naginata ------------------------------------------------------------------------------

#[test]
fn o_naginata_attaches_only_to_power_three_or_greater() {
    cr!("301.5c", "702.6a");
    ruling!(
        "O-Naginata",
        "O-Naginata can become attached to a creature only if that creature's power is 3 or greater before O-Naginata is attached to it."
    );
    supported("O-Naginata");
    let mut t = TestGame::new(2);
    let nag = t.battlefield(P0, "O-Naginata");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 4);
    let _ = t.activate(P0, nag, 0, &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.obj_now(nag).attached_to, None);
    // Once its power is 5, it can be equipped (and it becomes 8/5).
    giant_growth(&mut t, bears);
    t.activate(P0, nag, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(nag).attached_to, Some(obj(bears)));
    assert_eq!(t.pt(bears), (8, 5));
}

// --- Might Makes Right ------------------------------------------------------------------------

#[test]
fn might_makes_right_can_target_any_opposing_creature() {
    cr!("603.4", "115.1");
    ruling!(
        "Might Makes Right",
        "The ability can target any creature an opponent controls, not just one that’s tapped."
    );
    ruling!(
        "Might Makes Right",
        "“Each creature on the battlefield with the greatest power” may be a single creature if each other creature has less power."
    );
    supported("Might Makes Right");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Might Makes Right");
    t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(!t.obj_now(bears).tapped);
    t.answer_targets(P0, &[obj(bears)]);
    beginning_of_combat(&mut t);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
}

#[test]
fn might_makes_right_rechecks_its_condition_on_resolution() {
    cr!("603.4");
    ruling!(
        "Might Makes Right",
        "The ability will check its condition again as it tries to resolve. If, at that time, another player controls a creature with the greatest power or tied for the greatest power, the ability won’t do anything."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Might Makes Right");
    t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    beginning_of_combat(&mut t);
    assert_eq!(stack(&t), 1);
    // The Bears become 3/3 in response: tied for the greatest power.
    modify(
        &mut t,
        bears,
        vec![Modification::ModifyPT(Value::c(1), Value::c(1))],
    );
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P1);
}

// --- creatures with power 3 or greater entering -------------------------------------------------

/// With `watcher` on the battlefield, `enter` puts a creature onto the battlefield under
/// P0's control; `respond` runs with any trigger on the stack. Returns (triggers on the
/// stack after it entered, cards drawn).
fn enters_watched(
    watcher: &str,
    enter: impl FnOnce(&mut TestGame) -> ObjectId,
    respond: impl FnOnce(&mut TestGame, ObjectId),
) -> (usize, usize) {
    let mut t = TestGame::new(2);
    t.battlefield(P0, watcher);
    t.answer_yes(P0, true);
    let c = enter(&mut t);
    t.settle();
    let n = stack(&t);
    respond(&mut t, c);
    let hand = t.hand_size(P0);
    t.resolve_all();
    (n, t.hand_size(P0) - hand)
}

fn power_three_entering(watcher: &str) {
    // Enters with counters: power 3.
    let (n, drawn) = enters_watched(
        watcher,
        |t| enter_with(t, P0, "Grizzly Bears", 1, false),
        |_, _| {},
    );
    assert_eq!((n, drawn), (1, 1), "{watcher}: counters");
    // A static ability: Flinthoof Boar with a Mountain is 3/3.
    let (n, _) = enters_watched(
        watcher,
        |t| {
            t.battlefield(P0, "Mountain");
            t.enter(P0, "Flinthoof Boar")
        },
        |_, _| {},
    );
    assert_eq!(n, 1, "{watcher}: static");
    // Raising its power after it entered doesn't trigger it retroactively.
    let (n, drawn) = enters_watched(
        watcher,
        |t| t.enter(P0, "Grizzly Bears"),
        |t, c| {
            cast_new(t, P0, "Titanic Growth", &[obj(c)]);
            t.resolve_all();
        },
    );
    assert_eq!((n, drawn), (0, 0), "{watcher}: pumped later");
}

#[test]
fn garruks_packleader_checks_power_as_the_creature_enters() {
    cr!("603.2", "603.6a", "614.1c");
    ruling!(
        "Garruk's Packleader",
        "The ability checks the power of each creature as it enters. It will take into account counters the permanent enters with and static abilities that affect its power (such as Flinthoof Boar's ability). After the creature is on the battlefield, raising its power with a spell (such as Titanic Growth), an activated ability, or a triggered ability won't cause this ability to retroactively trigger."
    );
    supported("Garruk's Packleader");
    power_three_entering("Garruk's Packleader");
}

#[test]
fn elemental_bond_checks_power_as_the_creature_enters() {
    cr!("603.2", "603.6a", "614.1c");
    ruling!(
        "Elemental Bond",
        "The creature must have power 3 or greater as it enters, or Elemental Bond's ability won't trigger. Static abilities that raise (or lower) a creature's power are taken into account. However, you can't have a creature with power 2 or less enter and try to raise its power with a spell, an activated ability, or a triggered ability."
    );
    supported("Elemental Bond");
    power_three_entering("Elemental Bond");
}

#[test]
fn garruks_packleader_doesnt_recheck_power_on_resolution() {
    cr!("603.2", "603.4");
    ruling!(
        "Garruk's Packleader",
        "The creature's power is checked only to see if the ability triggers. It doesn't matter what the creature's power is when the ability resolves."
    );
    let (n, drawn) = enters_watched(
        "Garruk's Packleader",
        |t| t.enter(P0, "Hill Giant"),
        |t, c| minus_power(t, c, 3),
    );
    assert_eq!((n, drawn), (1, 1));
}

// --- Might of the Old Ways --------------------------------------------------------------------

#[test]
fn might_of_the_old_ways_pumps_before_checking_coven() {
    cr!("608.2c");
    ruling!(
        "Might of the Old Ways",
        "The effects of Might of the Old Ways happen in the listed order, so the +2/+2 bonus happens before you check the powers of creatures you control."
    );
    supported("Might of the Old Ways");
    // (creatures, index of the target, draws)
    let cases: [(&[&str], usize, bool); 4] = [
        (&["Grizzly Bears", "Grizzly Bears", "Thragtusk"], 0, true),
        (&["Llanowar Elves", "Hill Giant", "Thragtusk"], 0, false),
        (&["Llanowar Elves", "Hill Giant", "Thragtusk"], 1, false),
        (&["Llanowar Elves", "Hill Giant", "Thragtusk"], 2, true),
    ];
    for (names, target, draws) in cases {
        let mut t = TestGame::new(2);
        let ids: Vec<ObjectId> = names.iter().map(|n| t.battlefield(P0, n)).collect();
        cast_new(&mut t, P0, "Might of the Old Ways", &[obj(ids[target])]);
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.hand_size(P0) == hand + 1, draws, "{names:?} {target}");
    }
}

// --- Doran, Besieged by Time -------------------------------------------------------------------

#[test]
fn doran_uses_the_difference_between_power_and_toughness() {
    cr!("608.2h");
    ruling!(
        "Doran, Besieged by Time",
        "To find the difference between a creature's power and its toughness, subtract the smaller of those two numbers from the larger one."
    );
    supported("Doran, Besieged by Time");
    let mut t = TestGame::new(2);
    let doran = t.battlefield(P0, "Doran, Besieged by Time");
    let tusk = t.battlefield(P0, "Thragtusk");
    attack_with(
        &mut t,
        &[(doran, Entity::Player(P1)), (tusk, Entity::Player(P1))],
    );
    t.resolve_all();
    assert_eq!(t.pt(doran), (5, 10), "0/5: +5/+5");
    assert_eq!(t.pt(tusk), (7, 5), "5/3: +2/+2");
}

// --- Triumph of Cruelty -------------------------------------------------------------------

#[test]
fn triumph_of_cruelty_always_triggers_and_checks_on_resolution() {
    cr!("603.4", "608.2c");
    ruling!(
        "Triumph of Cruelty",
        "Triumph of Cruelty’s ability will trigger at the beginning of each of your upkeeps regardless of who controls the creature with the greatest power. The ability checks whether the opponent will discard a card when it resolves."
    );
    supported("Triumph of Cruelty");
    for respond in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Triumph of Cruelty");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.battlefield(P1, "Hill Giant");
        t.hand(P1, "Forest");
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        assert_eq!(stack(&t), 1, "it triggers although P1 has the greatest power");
        if respond {
            giant_growth(&mut t, bears);
        }
        let hand = t.hand_size(P1);
        t.resolve_all();
        assert_eq!(t.hand_size(P1) + usize::from(respond), hand);
    }
}

// --- Driftgloom Coyote, Fling ----------------------------------------------------------------

#[test]
fn driftgloom_coyote_uses_the_exiled_creatures_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Driftgloom Coyote",
        "Use the power of the exiled creature as it last existed on the battlefield to determine whether or not to put a +1/+1 counter on Driftgloom Coyote."
    );
    supported("Driftgloom Coyote");
    for (counters, gets) in [(0, 1), (2, 0)] {
        let mut t = TestGame::new(2);
        let bears = with_counters(&mut t, P1, "Grizzly Bears", counters);
        t.answer_targets(P0, &[obj(bears)]);
        let coyote = t.enter(P0, "Driftgloom Coyote");
        t.resolve_all();
        assert!(t.in_exile("Grizzly Bears"));
        assert_eq!(plus1(&t, coyote), gets, "{counters}");
    }
}

#[test]
fn fling_uses_the_sacrificed_creatures_last_known_power() {
    cr!("608.2h", "113.7a", "601.2h");
    ruling!(
        "Fling",
        "Use the sacrificed creature's power as it last existed on the battlefield to determine how much damage Fling deals."
    );
    supported("Fling");
    let mut t = TestGame::new(2);
    let bears = with_counters(&mut t, P0, "Grizzly Bears", 2);
    t.lands(P0, "Mountain", 2);
    let fling = t.hand(P0, "Fling");
    t.answer_choose(P0, &[obj(bears)]);
    t.cast(P0, fling).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

// --- Evolving Adaptive and evolve ----------------------------------------------------------------

/// Evolving Adaptive with `oil` oil counters (and `mods`) is on the battlefield; `name`
/// enters under P0's control. Returns (triggers, Adaptive) after `respond` and
/// resolution.
fn adaptive(
    oil: u32,
    mods: Vec<Modification>,
    name: &str,
    respond: impl FnOnce(&mut TestGame, ObjectId),
) -> (usize, u32) {
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Evolving Adaptive");
    put_counters(&mut t, a, "oil", oil);
    if !mods.is_empty() {
        modify(&mut t, a, mods);
    }
    let c = t.enter(P0, name);
    t.settle();
    let n = stack(&t);
    respond(&mut t, c);
    t.resolve_all();
    (n, t.counters(a, "oil"))
}

#[test]
fn evolving_adaptive_doesnt_trigger_if_neither_stat_is_greater() {
    cr!("603.4");
    ruling!(
        "Evolving Adaptive",
        "Whenever another creature enters the battlefield under your control, check its power and toughness against the power and toughness of Evolving Adaptive. If neither stat of the new creature is greater, the last ability won't trigger at all."
    );
    supported("Evolving Adaptive");
    assert_eq!(adaptive(2, vec![], "Grizzly Bears", |_, _| {}), (0, 2));
    assert_eq!(adaptive(2, vec![], "Hill Giant", |_, _| {}), (1, 3));
}

#[test]
fn evolving_adaptive_compares_power_to_power_and_toughness_to_toughness() {
    cr!("603.4");
    ruling!(
        "Evolving Adaptive",
        "When comparing the stats of the two creatures for Evolving Adaptive's last ability, you always compare power to power and toughness to toughness."
    );
    // A 1/4 Adaptive and a 0/3 Wall of Wood: neither is greater (though 3 > 1).
    let mods = vec![Modification::ModifyPT(Value::c(0), Value::c(3))];
    assert_eq!(adaptive(1, mods, "Wall of Wood", |_, _| {}), (0, 1));
}

fn plus2_minus2(t: &mut TestGame, c: ObjectId) {
    modify(
        t,
        c,
        vec![Modification::ModifyPT(Value::c(2), Value::c(-2))],
    );
}

#[test]
fn evolving_adaptive_greater_stat_may_change_before_resolution() {
    cr!("603.4");
    ruling!(
        "Evolving Adaptive",
        "When comparing the stats as the ability resolves, it's possible that the stat that's greater changes from power to toughness or vice versa. If this happens, the ability will still resolve and you'll put an oil counter on Evolving Adaptive."
    );
    // A 2/2 Adaptive; a 1/3 enters and becomes 3/1 in response.
    assert_eq!(
        adaptive(2, vec![], "Dreadhorde Arcanist", plus2_minus2),
        (1, 3)
    );
}

#[test]
fn propagator_drone_evolve_greater_stat_may_change_before_resolution() {
    cr!("702.100b", "603.4");
    ruling!(
        "Propagator Drone",
        "When comparing the stats as the evolve ability resolves, it's possible that the stat that's greater changes from power to toughness or vice versa. If this happens, the ability will still resolve and you'll put a +1/+1 counter on the creature with evolve."
    );
    supported("Propagator Drone");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Propagator Drone");
    let token = crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    give_plus1(&mut t, token, 1);
    assert_eq!(t.pt(token), (2, 2));
    let c = t.enter(P0, "Dreadhorde Arcanist");
    t.settle();
    assert_eq!(stack(&t), 1);
    plus2_minus2(&mut t, c);
    t.resolve_all();
    assert_eq!(plus1(&t, token), 2);
}

// --- Augur of Autumn -------------------------------------------------------------------------

#[test]
fn augur_of_autumn_casts_creatures_from_the_top_with_normal_timing() {
    cr!("601.3", "702.8a", "307.1");
    ruling!(
        "Augur of Autumn",
        "While you have three creatures with different powers, Augur of Autumn allows you to cast the top card of your library if it's a creature card, it's your main phase, and the stack is empty. If that creature card has flash, you'll be able to cast it any time you could cast an instant, even on an opponent's turn."
    );
    supported("Augur of Autumn");
    let setup = |t: &mut TestGame, coven: bool, top: &str| {
        t.battlefield(P0, "Augur of Autumn");
        t.battlefield(P0, "Llanowar Elves");
        if coven {
            t.battlefield(P0, "Hill Giant");
        }
        t.lands(P0, "Forest", 3);
        t.library_top(P0, top)
    };
    let mut t = TestGame::new(2);
    let bears = setup(&mut t, true, "Grizzly Bears");
    assert!(can_cast(&mut t, P0, bears, CastMethod::Normal));
    // Not without coven.
    let mut t = TestGame::new(2);
    let bears = setup(&mut t, false, "Grizzly Bears");
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
    // Not a sorcery-speed creature on an opponent's turn; a flash creature can be.
    let mut t = TestGame::new(2);
    let bears = setup(&mut t, true, "Grizzly Bears");
    t.set_step(P1, Step::Upkeep);
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
    let mut t = TestGame::new(2);
    let viper = setup(&mut t, true, "Ambush Viper");
    t.set_step(P1, Step::Upkeep);
    assert!(can_cast(&mut t, P0, viper, CastMethod::Normal));
}

#[test]
fn augur_of_autumn_pays_all_costs() {
    cr!("601.2b", "601.2f");
    ruling!(
        "Augur of Autumn",
        "You still pay all costs for that spell, including additional costs. You may also pay alternative costs."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Augur of Autumn");
    t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Forest", 5);
    let titan = t.library_top(P0, "Kavu Titan");
    t.cast(P0, titan).kicked(true).go();
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 5);
    t.resolve_all();
    let titan = t.g.current(titan);
    assert_eq!(t.pt(titan), (5, 5));
}

// --- Wall of Mourning ---------------------------------------------------------------------

#[test]
fn wall_of_mourning_returns_a_random_card_without_looking() {
    cr!("406.3", "406.4");
    ruling!(
        "Wall of Mourning",
        "You don't get to look at the cards exiled with Wall of Mourning. If there are more than one, you choose which one to put back into your hand at random without looking at them."
    );
    supported("Wall of Mourning");
    let mut t = TestGame::new(3);
    t.enter(P0, "Wall of Mourning");
    t.resolve_all();
    let exiled: Vec<ObjectId> = t.g.exile.iter().copied().collect();
    assert_eq!(exiled.len(), 2, "one per opponent");
    assert!(exiled.iter().all(|e| t.g.obj(*e).face_down));
    t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    let from = n_asked(&t);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.g.exile.len(), 1);
    // The two hidden cards are one indistinguishable pile: a single choice is offered,
    // and which card of it is returned is random (CR 406.4).
    let offered: Vec<usize> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *p == P0 => Some(candidates.len()),
            _ => None,
        })
        .collect();
    assert_eq!(offered, vec![1]);
}
