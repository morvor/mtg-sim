//! Item `rulings-residue-shapechange-typal` — interaction rulings about effects that set
//! power and toughness (layer 7b) against characteristic-defining abilities (7a),
//! modifications and counters (7c), each other (timestamps, CR 613.7) and their durations
//! ("until your next turn", CR 611.2b); and "activate only once" on a new object
//! (CR 400.7, 602.5b).

use crate::r_residue_shapechange_typal_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Amplifire on P0's battlefield with Craw Wurm (6/4) on top of P0's library; P0's next
/// upkeep has begun (its trigger not yet on the stack).
fn amplifire_upkeep(t: &mut TestGame) -> ObjectId {
    let fire = t.battlefield(P0, "Amplifire");
    t.library_top(P0, "Craw Wurm");
    t.advance_to(P0, Step::Upkeep);
    fire
}

#[test]
fn amplifire_is_1_1_from_the_untap_step_until_its_trigger_resolves() {
    cr!("611.2b", "613.4b", "503.1a");
    ruling!(
        "Amplifire",
        "Amplifire will be a 1/1 creature from the time your untap step begins until the time its triggered ability resolves during your upkeep."
    );
    supported("Amplifire");
    let mut t = TestGame::new(2);
    let fire = amplifire_upkeep(&mut t);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "reveal"), 1);
    assert_eq!(pt_now(&mut t, fire), (1, 1));
    t.resolve_all();
    assert_eq!(pt_now(&mut t, fire), (12, 8));
    // It stays 12/8 through the opponent's turn ...
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(pt_now(&mut t, fire), (12, 8));
    t.advance_to(P1, Step::End);
    assert_eq!(pt_now(&mut t, fire), (12, 8));
    // ... and is 1/1 again once P0's next turn begins, until the trigger resolves.
    t.library_top(P0, "Grizzly Bears");
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(pt_now(&mut t, fire), (1, 1));
    t.settle();
    assert_eq!(pt_now(&mut t, fire), (1, 1));
    t.resolve_all();
    assert_eq!(pt_now(&mut t, fire), (4, 4));
}

#[test]
fn amplifire_base_pt_under_earlier_pumps_and_counters() {
    cr!("613.4b", "613.4c", "613.7");
    ruling!(
        "Amplifire",
        "Any effects that modify Amplifire’s power and/or toughness without setting them to a specific number or value will apply after its base power and toughness are set, regardless of the order in which those effects were created. The same is true of counters that modify its power and toughness."
    );
    let mut t = TestGame::new(2);
    let fire = amplifire_upkeep(&mut t);
    t.settle();
    // With the trigger on the stack: a +1/+1 counter and Giant Growth (+3/+3) first.
    give_plus1(&mut t, fire, 1);
    t.lands(P0, "Forest", 1);
    let gg = t.hand(P0, "Giant Growth");
    t.cast_with(P0, gg, &[obj(fire)]).unwrap();
    t.resolve();
    assert_eq!(pt_now(&mut t, fire), (5, 5));
    t.resolve_all();
    // Base 12/8, then +1/+1 and +3/+3.
    assert_eq!(pt_now(&mut t, fire), (16, 12));
}

#[test]
fn amplifire_with_no_creature_card_in_the_library_is_0_0() {
    cr!("613.4b", "704.5f", "701.20a");
    ruling!(
        "Amplifire",
        "If you reveal your entire library and don’t reveal a creature card, Amplifire becomes a 0/0 creature and you put your library back in a random order."
    );
    let mut t = TestGame::new(2);
    let fire = t.battlefield(P0, "Amplifire");
    t.advance_to(P0, Step::Upkeep);
    let before = t.library_size(P0);
    t.settle();
    // With a +1/+1 counter it survives as a 1/1, showing its base is 0/0.
    give_plus1(&mut t, fire, 1);
    t.resolve_all();
    assert_eq!(pt_now(&mut t, fire), (1, 1));
    assert_eq!(t.library_size(P0), before);
    // Without the counter, it dies to the state-based action.
    crate::r_p121_common::remove_plus1(&mut t, fire, 1);
    assert!(t.in_graveyard(P0, "Amplifire"));
}

#[test]
fn okos_hospitality_overwrites_earlier_setting_effects_and_later_ones_overwrite_it() {
    cr!("613.4b", "613.7", "611.2c");
    ruling!(
        "Oko's Hospitality",
        "Oko's Hospitality overwrites all previous effects that set the creatures' base power and toughness to specific values. Any power- or toughness-setting effects that start to apply after Oko's Hospitality resolves will overwrite this effect."
    );
    supported("Oko's Hospitality");
    supported("Diminish");
    let mut t = main_phase(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let bears = t.battlefield(P0, "Grizzly Bears");
    diminish(&mut t, wurm);
    assert_eq!(pt_now(&mut t, wurm), (1, 1));
    give_plus1(&mut t, bears, 1);
    cast_resolve(&mut t, P0, "Oko's Hospitality", &[]);
    assert_eq!(pt_now(&mut t, wurm), (3, 3));
    // The counter still applies on top.
    assert_eq!(pt_now(&mut t, bears), (4, 4));
    diminish(&mut t, wurm);
    assert_eq!(pt_now(&mut t, wurm), (1, 1));
    // A creature that enters afterwards isn't affected (CR 611.2c).
    let late = t.battlefield(P0, "Craw Wurm");
    assert_eq!(pt_now(&mut t, late), (6, 4));
}

/// The artifact Shapeshifter ({6}, "* / 7-*") entering with the number `n` chosen.
fn shapeshifter_with(t: &mut TestGame, n: i64) -> ObjectId {
    supported("Shapeshifter");
    t.answer(P0, DecisionKind::Number, Answer::Number(n));
    let s = t.enter(P0, "Shapeshifter");
    t.settle();
    s
}

#[test]
fn shapeshifter_cda_applies_before_setting_and_modifying_effects() {
    cr!("604.3", "613.4a", "613.4b", "613.4c");
    ruling!(
        "Shapeshifter",
        "The ability that sets its power and toughness is a characteristic-defining ability. This means it applies before any other effects that modify power and/or toughness."
    );
    let mut t = main_phase(2);
    let s = shapeshifter_with(&mut t, 2);
    assert_eq!(pt_now(&mut t, s), (2, 5));
    giant_growth(&mut t, s);
    assert_eq!(pt_now(&mut t, s), (5, 8));
    // Diminish (layer 7b) overrides the characteristic-defining ability (7a) ...
    diminish(&mut t, s);
    assert_eq!(pt_now(&mut t, s), (4, 4));
    // ... even when a new number is chosen after it (in the next upkeep, with Diminish
    // cast while the trigger waits on the stack).
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(pt_now(&mut t, s), (2, 5));
    diminish(&mut t, s);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Number, Answer::Number(6));
    t.resolve_all();
    assert_eq!(pt_now(&mut t, s), (1, 1));
    // Diminish ends: the new number shows.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(pt_now(&mut t, s), (6, 1));
}

#[test]
fn shapeshifter_chooses_again_once_each_upkeep() {
    cr!("603.2", "604.3", "503.1");
    ruling!(
        "Shapeshifter",
        "The ability that lets you choose a new number is a triggered ability that triggers only once each upkeep."
    );
    let mut t = main_phase(2);
    let s = shapeshifter_with(&mut t, 2);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "number"), 1);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Number, Answer::Number(6));
    t.resolve_all();
    assert_eq!(pt_now(&mut t, s), (6, 1));
    // The opponent's upkeep doesn't trigger it.
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "number"), 0);
    assert_eq!(pt_now(&mut t, s), (6, 1));
}

#[test]
fn atomic_microsizer_overwrites_earlier_setting_effects_only() {
    cr!("613.4b", "613.4c", "613.7", "301.5c");
    ruling!(
        "Atomic Microsizer",
        "The effect of Atomic Microsizer’s second ability will overwrite any previous effects that set the creature’s power and toughness to specific values. Effects that otherwise modify the target creature’s power and toughness will still apply no matter when they took effect. The same is true for +1/+1 counters."
    );
    supported("Atomic Microsizer");
    let mut t = main_phase(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let m = t.battlefield(P0, "Atomic Microsizer");
    t.lands(P0, "Wastes", 2);
    t.activate(P0, m, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    // Oko's Hospitality (base 3/3) and a +1/+1 counter, both before the trigger.
    cast_resolve(&mut t, P0, "Oko's Hospitality", &[]);
    give_plus1(&mut t, bears, 1);
    assert_eq!(pt_now(&mut t, bears), (5, 4));
    assert_eq!(pt_now(&mut t, wurm), (3, 3));
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    // Base 1/1 overwrites the 3/3; the Equipment's +1/+0 and the counter still apply.
    assert_eq!(pt_now(&mut t, bears), (3, 2));
    assert_eq!(pt_now(&mut t, wurm), (3, 3));
}

#[test]
fn surge_engine_activates_its_last_ability_once_per_object() {
    cr!("602.5b", "400.7");
    ruling!(
        "Surge Engine",
        "Surge Engine's last ability can be activated only once per game, not once per turn."
    );
    ruling!(
        "Surge Engine",
        "If a Surge Engine whose last ability has been activated leaves the battlefield and comes back, it's a new object on the battlefield and its last ability can be activated again (once it's blue, of course)."
    );
    supported("Surge Engine");
    supported("Cloudshift");
    let mut t = main_phase(2);
    let e = t.battlefield(P0, "Surge Engine");
    t.lands(P0, "Island", 40);
    // Not blue yet: the last ability can't be activated.
    assert!(t.activate(P0, e, 2, &[]).is_err());
    t.activate(P0, e, 0, &[]).unwrap();
    t.resolve_all();
    t.activate(P0, e, 1, &[]).unwrap();
    t.resolve_all();
    assert!(has_color(&mut t, e, Color::Blue));
    assert_eq!(pt_now(&mut t, e), (5, 4));
    let hand = t.hand_size(P0);
    t.activate(P0, e, 2, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
    assert!(t.activate(P0, e, 2, &[]).is_err());
    // Not even in a later turn.
    t.advance_to(P0, Step::PrecombatMain);
    for l in t.g.permanents().map(|o| o.id).collect::<Vec<_>>() {
        t.g.objects[l.0 as usize].tapped = false;
    }
    assert!(t.activate(P0, e, 2, &[]).is_err());
    // Flickered, it's a new object: colorless 3/2 with defender again ...
    cast_resolve(&mut t, P0, "Cloudshift", &[obj(e)]);
    let e2 = t.g.current(e);
    assert_ne!(e2, e);
    assert!(!has_color(&mut t, e2, Color::Blue));
    assert!(t.activate(P0, e2, 2, &[]).is_err());
    // ... whose last ability can be activated again once it's blue.
    t.activate(P0, e2, 0, &[]).unwrap();
    t.resolve_all();
    t.activate(P0, e2, 1, &[]).unwrap();
    t.resolve_all();
    let hand = t.hand_size(P0);
    t.activate(P0, e2, 2, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
}
