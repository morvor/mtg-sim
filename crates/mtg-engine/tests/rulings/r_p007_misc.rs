//! Rulings batch P007 — assorted burn rulings: restrictions on activated abilities,
//! base power and toughness (CR 613.4b–c), resolution order for several targets
//! (CR 101.4), costs (CR 118, 601.2f), copies (CR 707.10), returning spells to hand,
//! counting permanents, and named cards.

use crate::r_p007_common::*;
use crate::r_s01_common::{attack_with, stack_library, supported};
use crate::r_s02_common::destroy;
use crate::r_s05_common::run_from;
use crate::r_s06_common::{activate_containing, give_control};
use crate::r_s07_common::damage_on;
use crate::r_s11_common::empty_library;
use crate::r_s25_common::{cast_new, lands_for_cost};
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::events::Event;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Applies `mods` to the permanent until end of turn (as a resolving effect of P0's).
fn modify(t: &mut TestGame, id: ObjectId, mods: Vec<Modification>) {
    run_from(
        t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods,
            duration: Duration::EndOfTurn,
        },
        &[obj(t.g.current(id))],
    );
}

#[test]
fn mythos_of_vadrok_stops_activated_but_not_triggered_abilities() {
    cr!("602.1", "603.1", "602.5");
    ruling!(
        "Mythos of Vadrok",
        "Activated abilities contain a colon. They’re generally written “[Cost]: [Effect].” Some keyword abilities are activated abilities and will have colons in their reminder text. Triggered abilities (starting with “when,” “whenever,” or “at”) are unaffected by Mythos of Vadrok."
    );
    supported("Mythos of Vadrok");
    // Control: without Mythos, P1 can activate Vivien's +1 (with no targets) on their turn.
    let mut t = TestGame::new(2);
    let walker = pw(&mut t, P1, "Vivien, Arkbow Ranger", 4);
    t.set_step(P1, Step::PrecombatMain);
    t.answer_targets(P1, &[]);
    assert!(t.activate(P1, walker, 0, &[]).is_ok(), "control");
    let mut t = TestGame::new(2);
    let walker = pw(&mut t, P1, "Vivien, Arkbow Ranger", 4);
    let bombers = t.battlefield(P1, "Mogg Bombers");
    let maw = t.battlefield(P1, "Colossal Dreadmaw");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 2);
    let card = t.hand(P0, "Mythos of Vadrok");
    t.answer_targets(P0, &[obj(walker), obj(bombers), obj(maw)]);
    divide(&mut t, P0, &[1, 1, 3]);
    t.cast(P0, card).go();
    t.resolve_all();
    assert_eq!(t.counters(walker, "loyalty"), 3);
    // P1's turn: Vivien's loyalty abilities can't be activated...
    t.set_step(P1, Step::PrecombatMain);
    t.answer_targets(P1, &[]);
    assert!(t.activate(P1, walker, 0, &[]).is_err());
    t.clear_answers();
    // ... but Mogg Bombers' triggered ability still triggers and resolves.
    t.answer_targets(P1, &[pl(P0)]);
    t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
}

#[test]
fn bears_of_littjara_base_pt_is_modified_by_earlier_effects_and_counters() {
    cr!("613.4b", "613.4c", "714.2b");
    ruling!(
        "The Bears of Littjara",
        "Any effects that modify the Shapeshifter’s power or toughness without setting it to a specific value will apply after its base power and toughness are set, regardless of the order in which those effects were created. The same is true of counters that modify its power and toughness."
    );
    supported("The Bears of Littjara");
    let mut t = TestGame::new(2);
    let shifter = t.battlefield(P0, "Changeling Outcast");
    t.g.objects[shifter.0 as usize]
        .counters
        .insert("+1/+1".into(), 1);
    modify(
        &mut t,
        shifter,
        vec![Modification::ModifyPT(Value::Const(3), Value::Const(3))],
    );
    assert_eq!(t.pt(shifter), (5, 5));
    let saga = t.battlefield(P0, "The Bears of Littjara");
    t.g.objects[saga.0 as usize]
        .counters
        .insert("lore".into(), 1);
    t.answer_targets(P0, &[obj(shifter)]);
    t.g.add_counters(obj(saga), "lore", 1, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.pt(shifter), (8, 8), "base 4/4, +3/+3, +1/+1 counter");
}

#[test]
fn lava_spike_is_an_arcane_spell() {
    cr!("205.3k");
    ruling!(
        "Lava Spike",
        "Arcane is a spell subtype that has no inherent rules meaning. It may be referenced by effects from other cards."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soul of Magma");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let s = cast_new(&mut t, P0, "Lava Spike", &[pl(P1)]);
    assert!(t.obj_now(s).chars.has_subtype("Arcane"));
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(damage_on(&t, bears), 1, "Soul of Magma triggered");
}

#[test]
fn cavalier_of_flame_may_discard_zero_cards() {
    cr!("107.1b", "608.2d");
    ruling!(
        "Cavalier of Flame",
        "As Cavalier of Flame’s enters-the-battlefield ability resolves, you may choose to discard zero cards. You just won’t draw any cards."
    );
    supported("Cavalier of Flame");
    let mut t = TestGame::new(2);
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Hill Giant");
    let (hand, lib) = (t.hand_size(P0), t.library_size(P0));
    t.answer_choose(P0, &[]);
    t.enter(P0, "Cavalier of Flame");
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.library_size(P0)), (hand, lib));
    assert!(t.in_hand(P0, "Hill Giant"));
}

#[test]
fn soulfire_eruption_handles_targets_in_apnap_order() {
    cr!("101.4", "608.2c");
    ruling!(
        "Soulfire Eruption",
        "As Soulfire Eruption resolves, if you have chosen multiple targets (and we really hope you have), those targets will be handled one at a time in turn order. Start with targets that are either the player whose turn it is (probably you) or permanents that player controls."
    );
    // P1 is chosen first, P0's Hill Giant second; P0's group goes first: the Giant gets
    // the top card (Shock, mana value 1), P1 the next (Craw Wurm, 6).
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    lands_for_cost(&mut t, P0, "Soulfire Eruption");
    stack_library(&mut t, P0, &["Shock", "Craw Wurm"]);
    let card = t.hand(P0, "Soulfire Eruption");
    t.answer_targets(P0, &[pl(P1), obj(giant)]);
    t.cast(P0, card).go();
    t.resolve_all();
    assert_eq!(damage_on(&t, giant), 1);
    assert_eq!(t.life(P1), 14);
}

#[test]
fn boros_charm_indestructible_only_for_permanents_controlled_on_resolution() {
    cr!("611.2c");
    ruling!(
        "Boros Charm",
        "Boros Charm's second mode affects only permanents you control at the time it resolves. It won't affect permanents that come under your control later in the turn."
    );
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    cast_new(&mut t, P0, "Boros Charm", &[]);
    t.resolve_all();
    give_control(&mut t, theirs, P0);
    destroy(&mut t, mine);
    destroy(&mut t, theirs);
    assert!(t.on_battlefield(mine));
    assert!(!t.on_battlefield(theirs));
}

#[test]
fn sacrificing_the_source_itself_as_a_cost() {
    cr!("602.2b", "118.3", "608.2h");
    ruling!(
        "Breya, Etherium Shaper",
        "Breya can be one of the two artifacts sacrificed to pay the cost of its last ability."
    );
    ruling!(
        "Elite Headhunter",
        "If Elite Headhunter somehow becomes an artifact, you can sacrifice it to pay the cost of its activated ability."
    );
    supported("Breya, Etherium Shaper");
    supported("Elite Headhunter");
    // Breya and an Ornithopter: 3 damage to P1.
    let mut t = TestGame::new(2);
    let breya = t.battlefield(P0, "Breya, Etherium Shaper");
    let thopter = t.battlefield(P0, "Ornithopter");
    t.lands(P0, "Wastes", 2);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer_choose(P0, &[obj(breya), obj(thopter)]);
    t.answer_targets(P0, &[pl(P1)]);
    activate_containing(&mut t, P0, breya, "Sacrifice two artifacts").expect("Breya");
    assert!(!t.on_battlefield(breya));
    assert!(!t.on_battlefield(thopter));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Elite Headhunter made an artifact sacrifices itself.
    let mut t = TestGame::new(2);
    let hh = t.battlefield(P0, "Elite Headhunter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    modify(
        &mut t,
        hh,
        vec![Modification::AddTypes(vec![CardType::Artifact])],
    );
    t.lands(P0, "Mountain", 3);
    t.answer_choose(P0, &[obj(hh)]);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, hh, "Sacrifice").expect("Headhunter");
    assert!(!t.on_battlefield(hh));
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn hour_of_devastation_doesnt_affect_later_creatures() {
    cr!("611.2c");
    ruling!(
        "Hour of Devastation",
        "Creatures with indestructible that enter the battlefield after Hour of Devastation resolves will still have indestructible."
    );
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Hour of Devastation", &[]);
    t.resolve_all();
    let myr = t.battlefield(P0, "Darksteel Myr");
    destroy(&mut t, myr);
    assert!(t.on_battlefield(myr));
}

#[test]
fn unleash_the_inferno_excess_damage_to_a_planeswalker() {
    cr!("120.10", "603.12");
    ruling!(
        "Unleash the Inferno",
        "Excess damage has been dealt to a planeswalker if the amount of damage dealt to it is greater than the number of loyalty counters that it had remaining when the damage was dealt."
    );
    supported("Unleash the Inferno");
    // (loyalty, whether the mana value 3 artifact is destroyed)
    for (loyalty, destroyed) in [(4, true), (5, false)] {
        let mut t = TestGame::new(2);
        let walker = pw(&mut t, P1, "Vivien, Arkbow Ranger", loyalty);
        let art = t.battlefield(P1, "Scepter of Empires");
        cast_new(&mut t, P0, "Unleash the Inferno", &[obj(walker)]);
        t.answer_targets(P0, &[obj(art)]);
        t.resolve_all();
        t.clear_answers();
        assert!(!t.on_battlefield(walker));
        assert_eq!(!t.on_battlefield(art), destroyed, "loyalty {loyalty}");
    }
}

#[test]
fn tundra_fumarole_copy_adds_no_mana() {
    cr!("707.10", "601.2h", "106.3");
    ruling!(
        "Tundra Fumarole",
        "If Tundra Fumarole is copied, no snow mana was spent to cast the copy, so you won’t add any mana. Damage will still be dealt."
    );
    supported("Tundra Fumarole");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Colossal Dreadmaw");
    let b = t.battlefield(P1, "Colossal Dreadmaw");
    t.lands(P0, "Snow-Covered Mountain", 3);
    let card = t.hand(P0, "Tundra Fumarole");
    t.answer_targets(P0, &[obj(a)]);
    let fum = t.cast(P0, card).go();
    t.lands(P0, "Mountain", 2);
    let rev = t.hand(P0, "Reverberate");
    t.answer_targets(P0, &[obj(fum)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(b)]);
    t.cast(P0, rev).go();
    t.resolve();
    // The copy resolves first: damage, no mana.
    t.resolve();
    assert_eq!(damage_on(&t, b), 4);
    assert_eq!(t.g.players[0].mana_pool.count(ManaType::C), 0);
    // The original adds {C}{C}{C}.
    t.resolve();
    assert_eq!(damage_on(&t, a), 4);
    assert_eq!(t.g.players[0].mana_pool.count(ManaType::C), 3);
}

#[test]
fn wolf_of_devils_breach_x_is_zero_in_the_discarded_card() {
    cr!("202.3e", "107.3b");
    ruling!(
        "Wolf of Devil's Breach",
        "If a card with {X} in its mana cost is discarded to Wolf of Devil’s Breach’s ability, X is considered to be 0."
    );
    let mut t = TestGame::new(2);
    let wolf = t.battlefield(P0, "Wolf of Devil's Breach");
    let giant = t.battlefield(P1, "Hill Giant");
    let fireball = t.hand(P0, "Fireball");
    t.lands(P0, "Mountain", 2);
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(fireball)]);
    attack_with(&mut t, &[(wolf, pl(P1))]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Fireball"));
    assert_eq!(damage_on(&t, giant), 1, "Fireball's mana value is 1");
}

#[test]
fn road_rage_counts_a_mount_vehicle_once() {
    cr!("208.3", "608.2h");
    ruling!(
        "Road Rage",
        "If a creature is somehow both a Mount and a Vehicle, it counts only once."
    );
    supported("Road Rage");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    modify(
        &mut t,
        bears,
        vec![Modification::AddSubtypes(vec![
            "Mount".into(),
            "Vehicle".into(),
        ])],
    );
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_new(&mut t, P0, "Road Rage", &[obj(wurm)]);
    t.resolve_all();
    assert_eq!(damage_on(&t, wurm), 3);
}

#[test]
fn volcanic_salvo_negative_power_reduces_the_total() {
    cr!("601.2f", "107.1b");
    ruling!(
        "Volcanic Salvo",
        "If a creature's power is somehow less than 0, it subtracts from the total power of your other creatures. If the total power of your creatures is 0 or less, the cost remains {10}{R}{R}."
    );
    // (P0's creature powers, lands needed)
    for (with_giant, needed) in [(true, 10usize), (false, 12)] {
        for lands in [needed - 1, needed] {
            let mut t = TestGame::new(2);
            let bears = t.battlefield(P0, "Grizzly Bears");
            modify(
                &mut t,
                bears,
                vec![Modification::ModifyPT(Value::Const(-3), Value::Const(0))],
            );
            if with_giant {
                t.battlefield(P0, "Hill Giant");
            }
            let target = t.battlefield(P1, "Craw Wurm");
            t.lands(P0, "Mountain", 2);
            t.lands(P0, "Wastes", lands - 2);
            let card = t.hand(P0, "Volcanic Salvo");
            t.answer_targets(P0, &[obj(target)]);
            let ok = t.cast(P0, card).try_go().is_ok();
            t.clear_answers();
            assert_eq!(ok, lands == needed, "giant {with_giant}, {lands} lands");
        }
    }
}

#[test]
fn brutal_expulsion_returns_a_spell_without_countering_it() {
    cr!("701.6a", "707.10a", "704.5d");
    ruling!(
        "Brutal Expulsion",
        "If a spell is returned to its owner’s hand, it’s removed from the stack. The spell isn’t countered, but it won’t resolve. If a copy of a spell is returned to a hand this way, the copy will cease to exist the next time state-based actions are performed."
    );
    // A spell.
    let mut t = TestGame::new(2);
    let shock = cast_new(&mut t, P1, "Shock", &[pl(P0)]);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    cast_new(&mut t, P0, "Brutal Expulsion", &[obj(shock)]);
    t.resolve_all();
    assert!(t.in_hand(P1, "Shock"));
    assert_eq!(t.life(P0), 20);
    assert!(!t
        .g
        .turn_events
        .iter()
        .any(|e| matches!(e, Event::Countered { .. })));
    // A copy of a spell ceases to exist.
    let mut t = TestGame::new(2);
    let shock = cast_new(&mut t, P1, "Shock", &[pl(P0)]);
    t.answer_targets(P1, &[obj(shock)]);
    t.answer_yes(P1, false);
    cast_new(&mut t, P1, "Reverberate", &[obj(shock)]);
    t.resolve();
    let copy = crate::r_s25_common::spell_copies(&t)[0];
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    cast_new(&mut t, P0, "Brutal Expulsion", &[obj(copy)]);
    t.resolve();
    t.settle();
    assert!(!t.g.stack.contains(&copy));
    assert_eq!(t.hand_size(P1), 0, "the copy ceased to exist");
    t.resolve_all();
    assert_eq!(t.life(P0), 18, "only the original Shock resolved");
}

#[test]
fn scepter_of_empires_needs_the_named_cards_to_be_artifacts() {
    cr!("201.2", "608.2h");
    ruling!(
        "Scepter of Empires",
        "If any of the named cards stops being an artifact, it won't be considered by these abilities."
    );
    supported("Scepter of Empires");
    for remove in [false, true] {
        let mut t = TestGame::new(2);
        let scepter = t.battlefield(P0, "Scepter of Empires");
        t.battlefield(P0, "Crown of Empires");
        let throne = t.battlefield(P0, "Throne of Empires");
        if remove {
            modify(
                &mut t,
                throne,
                vec![Modification::RemoveTypes(vec![CardType::Artifact])],
            );
            assert!(!t.obj_now(throne).is(CardType::Artifact));
        }
        t.activate(P0, scepter, 0, &[pl(P1)]).expect("Scepter");
        t.resolve_all();
        assert_eq!(t.life(P1), if remove { 19 } else { 17 }, "remove {remove}");
    }
}

#[test]
fn kaboom_deals_zero_damage_with_an_all_land_library() {
    cr!("701.20a", "608.2c");
    ruling!(
        "Kaboom!",
        "If all the cards in your library are lands, this card deals zero damage."
    );
    supported("Kaboom!");
    let mut t = TestGame::new(2);
    empty_library(&mut t, P0);
    stack_library(&mut t, P0, &["Mountain", "Forest", "Island"]);
    t.answer_targets(P0, &[pl(P1)]);
    lands_for_cost(&mut t, P0, "Kaboom!");
    let card = t.hand(P0, "Kaboom!");
    t.cast(P0, card).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.library_size(P0), 3);
    // Control: a nonland card under the lands.
    let mut t = TestGame::new(2);
    empty_library(&mut t, P0);
    stack_library(&mut t, P0, &["Mountain", "Craw Wurm"]);
    t.answer_targets(P0, &[pl(P1)]);
    lands_for_cost(&mut t, P0, "Kaboom!");
    let card = t.hand(P0, "Kaboom!");
    t.cast(P0, card).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn the_fallen_remembers_through_phasing() {
    cr!("702.26d", "702.26a");
    ruling!(
        "The Fallen",
        "If The Fallen phases out and back in, it remembers which players it has dealt damage to this game."
    );
    supported("The Fallen");
    let mut t = TestGame::new(2);
    let fallen = t.battlefield(P0, "The Fallen");
    t.g.deal_damage(fallen, pl(P1), 2, false);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.life(P1), 18);
    mtg_engine::keyword_impls::phase_out(&mut t.g, vec![fallen]);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(fallen), "phased in");
    assert_eq!(t.life(P1), 17);
}
