//! Rulings batch P144 — repeatable impulse draw: the permission to play an exiled card
//! comes from a resolved ability, so it lasts for its duration even after the source
//! leaves the battlefield or changes control (CR 611.2a, 610.3); a card that isn't played
//! simply stays in exile once the permission ends; and a permission tied to an object
//! doesn't carry over to a new object (CR 400.7).

use crate::r_p107_common::{act, mana};
use crate::r_p117_common::gain_control;
use crate::r_p144_common::*;
use crate::r_s02_common::{can_cast, can_play_land, create_token, destroy};
use crate::r_s09_common::declare;
use mtg_engine::events::MoveCause;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Whether P0 could cast the exiled Lightning Bolt `bolt` now with {R} in the pool (the
/// pool is emptied again afterwards).
fn bolt_castable(t: &mut TestGame, bolt: ObjectId) -> bool {
    mana(t, P0, ManaType::R, 1);
    let r = can_cast(t, P0, bolt, CastMethod::Normal);
    t.g.players[P0.idx()].mana_pool = Default::default();
    r
}

/// Whether P0 could cast the exiled Lava Spike `spike` now (in P0's main phase) with {R}.
fn spike_castable(t: &mut TestGame, spike: ObjectId) -> bool {
    to_main(t);
    bolt_castable(t, spike)
}

fn attack(t: &mut TestGame, attackers: &[ObjectId]) {
    let decl: Vec<(ObjectId, Entity)> =
        attackers.iter().map(|a| (*a, Entity::Player(P1))).collect();
    declare(t, P0, &decl);
    t.resolve_all();
}

/// The setup of a "leaves or changes control" case: puts the source onto the battlefield
/// and has it exile the top card of P0's library; returns the source.
type Setup = fn(&mut TestGame) -> ObjectId;

#[test]
fn the_card_stays_playable_after_the_source_leaves_or_changes_control() {
    cr!("611.2a", "610.3");
    ruling!(
        "Hedron Detonator",
        "You may play the card exiled with Hedron Detonator even if Hedron Detonator leaves the battlefield or you lose control of it."
    );
    ruling!(
        "Ob Nixilis, Captive Kingpin",
        "You may play the card exiled with Ob Nixilis even if Ob Nixilis leaves the battlefield or you lose control of it."
    );
    ruling!(
        "Oracle's Vault",
        "You may play the exiled card this turn even if Oracle's Vault is no longer on the battlefield or under your control."
    );
    ruling!(
        "Ardent Dustspeaker",
        "You may play those cards that turn even if Ardent Dustspeaker leaves the battlefield or another player gains control of it."
    );
    ruling!(
        "Elkin Bottle",
        "If the Bottle leaves the battlefield or your control, the cards remain waiting until played or until the beginning of your next upkeep. The card is in the “Exile” zone."
    );
    let cases: [(&str, Setup); 5] = [
        ("Hedron Detonator", |t| {
            let h = t.battlefield(P0, "Hedron Detonator");
            let a = [
                t.battlefield(P0, "Ornithopter"),
                t.battlefield(P0, "Ornithopter"),
            ];
            t.answer_choose(P0, &[obj(a[0]), obj(a[1])]);
            act(t, P0, h, "Exile the top", &[]).expect("activate");
            h
        }),
        ("Ob Nixilis, Captive Kingpin", |t| {
            let o = t.battlefield(P0, "Ob Nixilis, Captive Kingpin");
            let pyro = t.battlefield(P0, "Prodigal Pyromancer");
            act(t, P0, pyro, "damage", &[Entity::Player(P1)]).expect("ping");
            o
        }),
        ("Oracle's Vault", |t| {
            let v = t.battlefield(P0, "Oracle's Vault");
            mana(t, P0, ManaType::C, 2);
            act(t, P0, v, "brick counter", &[]).expect("activate");
            v
        }),
        ("Ardent Dustspeaker", |t| {
            let d = t.battlefield(P0, "Ardent Dustspeaker");
            t.graveyard(P0, "Shock");
            t.answer_yes(P0, true);
            attack(t, &[d]);
            d
        }),
        ("Elkin Bottle", |t| {
            let b = t.battlefield(P0, "Elkin Bottle");
            mana(t, P0, ManaType::C, 3);
            act(t, P0, b, "Exile the top", &[]).expect("activate");
            b
        }),
    ];
    for (card, setup) in cases {
        supported(card);
        for steal in [false, true] {
            eprintln!("case: {card}, stolen: {steal}");
            let mut t = TestGame::new(2);
            let bolt = on_top(&mut t, P0, &["Lightning Bolt", "Wastes", "Wastes"])[0];
            let source = setup(&mut t);
            t.resolve_all();
            assert_eq!(t.zone(bolt), Zone::Exile);
            if steal {
                gain_control(&mut t, P1, source);
                assert_eq!(t.obj_now(source).controller, P1);
            } else {
                destroy(&mut t, source);
                assert!(!t.on_battlefield(source));
            }
            to_main(&mut t);
            assert!(bolt_castable(&mut t, bolt), "{card}");
        }
    }
}

#[test]
fn elkin_bottle_card_waits_until_the_next_upkeep_and_then_stays_exiled() {
    cr!("611.2a", "611.2b", "121.1", "406.3");
    supported("Elkin Bottle");
    let mut t = TestGame::new(2);
    let lib = on_top(&mut t, P0, &["Lightning Bolt", "Wastes", "Wastes"]);
    let b = t.battlefield(P0, "Elkin Bottle");
    let draws_before = t.g.player(P0).hand.len();
    mana(&mut t, P0, ManaType::C, 3);
    act(&mut t, P0, b, "Exile the top", &[]).unwrap();
    t.resolve_all();

    ruling!(
        "Elkin Bottle",
        "This is not considered to be drawing a card."
    );
    // Nothing was drawn: no card in hand, no draw event.
    assert_eq!(t.g.player(P0).hand.len(), draws_before);
    assert!(!t
        .g
        .turn_events
        .iter()
        .any(|e| matches!(e, mtg_engine::events::Event::Drew { .. })));

    ruling!(
        "Elkin Bottle",
        "The card is not part of your hand in any way. You can’t be forced to discard it due to a discard from hand effect, and you can’t discard it to pay a cost."
    );
    // Tormenting Voice's discard cost can't be paid with it: with no card in hand, P0
    // can't cast Tormenting Voice.
    let voice = t.hand(P0, "Tormenting Voice");
    mana(&mut t, P0, ManaType::R, 2);
    assert!(!can_cast(&mut t, P0, voice, CastMethod::Normal));
    t.g.players[P0.idx()].mana_pool = Default::default();

    // On P1's turn the instant is still playable. Mind Rot (P1): "Target player discards
    // two cards." P0 discards the one card in their hand; the exiled card isn't touched.
    t.advance_to(P1, Step::PrecombatMain);
    assert!(bolt_castable(&mut t, lib[0]));
    let held = t.hand(P0, "Wastes");
    let rot = t.hand(P1, "Mind Rot");
    mana(&mut t, P1, ManaType::B, 3);
    t.cast(P1, rot).target(P0).go();
    t.resolve_all();
    assert_eq!(t.zone(held), Zone::Graveyard(P0));
    assert_eq!(t.zone(lib[0]), Zone::Exile);
    assert!(bolt_castable(&mut t, lib[0]));

    ruling!(
        "Elkin Bottle",
        "If you do not play the card before your next upkeep, it remains exiled; you just lose the ability to play it."
    );
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.zone(lib[0]), Zone::Exile);
    assert!(!bolt_castable(&mut t, lib[0]));
}

#[test]
fn unplayed_cards_remain_exiled_and_played_ones_go_where_they_normally_would() {
    cr!("611.2a", "608.2n", "305.1");
    supported("Aerial Caravan");
    ruling!(
        "Aerial Caravan",
        "If you do not play the card by end of turn, it remains exiled. If you play it, it goes wherever the it would normally go when played."
    );
    // A Lava Spike played this way goes to the graveyard; an Island to the battlefield.
    let mut t = TestGame::new(2);
    let lib = on_top(
        &mut t,
        P0,
        &["Lava Spike", "Island", "Lightning Bolt", "Wastes"],
    );
    let c = t.battlefield(P0, "Aerial Caravan");
    for _ in 0..3 {
        mana(&mut t, P0, ManaType::U, 2);
        mana(&mut t, P0, ManaType::C, 1);
        act(&mut t, P0, c, "Exile the top", &[]).unwrap();
        t.resolve_all();
    }
    assert!(lib[..3].iter().all(|c| t.zone(*c) == Zone::Exile));
    mana(&mut t, P0, ManaType::R, 1);
    let spike = t.g.current(lib[0]);
    t.cast(P0, spike).target(P1).go();
    t.resolve_all();
    assert_eq!(t.zone(lib[0]), Zone::Graveyard(P0));
    assert_eq!(t.life(P1), 17);
    let island = t.g.current(lib[1]);
    t.play_land(P0, island).unwrap();
    assert!(t.on_battlefield(lib[1]));
    // The Bolt wasn't played: still exiled on P1's turn, and unplayable.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.zone(lib[2]), Zone::Exile);
    assert!(!bolt_castable(&mut t, lib[2]));

    supported("Chaos Channeler");
    ruling!(
        "Chaos Channeler",
        "If you do not play the exiled cards on the turn that they are exiled, they remain exiled indefinitely."
    );
    let mut t = TestGame::new(2);
    let lib = on_top(
        &mut t,
        P0,
        &["Lightning Bolt", "Wastes", "Wastes", "Wastes"],
    );
    let c = t.battlefield(P0, "Chaos Channeler");
    attack(&mut t, &[c]);
    assert_eq!(t.zone(lib[0]), Zone::Exile);
    assert!(bolt_castable(&mut t, lib[0]));
    t.advance_to(P1, Step::Upkeep);
    assert!(!bolt_castable(&mut t, lib[0]));
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.zone(lib[0]), Zone::Exile);
    assert!(!bolt_castable(&mut t, lib[0]));
}

#[test]
fn boros_strike_captain_permission_follows_the_attack_count_on_later_turns() {
    cr!("611.2a", "508.1");
    supported("Boros Strike-Captain");
    ruling!(
        "Boros Strike-Captain",
        "Similarly, once you attack with three or more creatures in a single turn, you can still play the exiled card that turn no matter what happens to those creatures."
    );
    let mut t = TestGame::new(2);
    let lib = on_top(&mut t, P0, &["Lava Spike", "Wastes", "Wastes", "Wastes"]);
    let cap = t.battlefield(P0, "Boros Strike-Captain");
    let b = [
        t.battlefield(P0, "Grizzly Bears"),
        t.battlefield(P0, "Grizzly Bears"),
    ];
    attack(&mut t, &[cap, b[0], b[1]]);
    assert_eq!(t.zone(lib[0]), Zone::Exile);
    for id in [cap, b[0], b[1]] {
        destroy(&mut t, id);
    }
    assert!(spike_castable(&mut t, lib[0]));

    ruling!(
        "Boros Strike-Captain",
        "If Boros Strike-Captain leaves the battlefield, the effect allowing you to play the exiled card if you attacked with three or more creatures continues to apply, even during later turns, as long as you attack with three or more creatures during those later turns."
    );
    // P0's next turn: before attacking, no; after attacking with only two, no; after a
    // second combat with three, yes.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    let n = [
        t.battlefield(P0, "Memnite"),
        t.battlefield(P0, "Memnite"),
        t.battlefield(P0, "Memnite"),
    ];
    assert!(!spike_castable(&mut t, lib[0]));
    attack(&mut t, &[n[0], n[1]]);
    assert!(!spike_castable(&mut t, lib[0]));
    for id in n {
        let id = t.g.current(id);
        t.g.objects[id.0 as usize].tapped = false;
    }
    attack(&mut t, &[n[0], n[1], n[2]]);
    assert!(spike_castable(&mut t, lib[0]));
}

#[test]
fn neyali_permission_survives_the_token_and_neyali() {
    cr!("611.2a", "506.4");
    supported("Neyali, Suns' Vanguard");
    ruling!(
        "Neyali, Suns' Vanguard",
        "You can cast the exiled card if the token that attacked is still in combat, if it's left combat, if it's left the battlefield, or even if combat is over."
    );
    let mut t = TestGame::new(2);
    let lib = on_top(
        &mut t,
        P0,
        &["Lightning Bolt", "Wastes", "Wastes", "Wastes"],
    );
    let ney = t.battlefield(P0, "Neyali, Suns' Vanguard");
    let tok = create_token(&mut t, P0, "Soldier");
    attack(&mut t, &[tok]);
    assert_eq!(t.zone(lib[0]), Zone::Exile);
    // In combat, with the token attacking: yes.
    assert!(bolt_castable(&mut t, lib[0]));
    // The token leaves the battlefield (during combat, and after combat): still yes.
    destroy(&mut t, tok);
    assert!(bolt_castable(&mut t, lib[0]));
    to_main(&mut t);
    assert!(bolt_castable(&mut t, lib[0]));

    ruling!(
        "Neyali, Suns' Vanguard",
        "The effect allowing you to cast the exiled card if you attacked with a token continues to apply even if Neyali leaves the battlefield or another player gains control of it. It also continues to apply during later turns—as long as you attacked with a token during those later turns."
    );
    gain_control(&mut t, P1, ney);
    assert!(bolt_castable(&mut t, lib[0]));
    destroy(&mut t, ney);
    assert!(bolt_castable(&mut t, lib[0]));
    // P1's turn (P0 attacked with no token): no. P0's next turn: no until a token attacks.
    t.advance_to(P1, Step::Upkeep);
    assert!(!bolt_castable(&mut t, lib[0]));
    t.advance_to(P0, Step::PrecombatMain);
    assert!(!bolt_castable(&mut t, lib[0]));
    let tok2 = create_token(&mut t, P0, "Soldier");
    attack(&mut t, &[tok2]);
    assert!(bolt_castable(&mut t, lib[0]));
}

#[test]
fn possibility_technician_permission_needs_only_a_kavu() {
    cr!("611.2a", "611.2b");
    supported("Possibility Technician");
    ruling!(
        "Possibility Technician",
        "If Possibility Technician leaves the battlefield, the effect allowing you to play the exiled cards if you control a Kavu continues to apply, even during later turns."
    );
    let mut t = TestGame::new(2);
    let lib = on_top(
        &mut t,
        P0,
        &["Lightning Bolt", "Wastes", "Wastes", "Wastes"],
    );
    let pt = t.enter(P0, "Possibility Technician");
    t.resolve_all();
    assert_eq!(t.zone(lib[0]), Zone::Exile);
    destroy(&mut t, pt);
    // No Kavu: no.
    assert!(!bolt_castable(&mut t, lib[0]));
    // Another Kavu (Flametongue Kavu put directly onto the battlefield): yes, on later
    // turns too.
    t.battlefield(P0, "Flametongue Kavu");
    assert!(bolt_castable(&mut t, lib[0]));
    t.advance_to(P1, Step::Upkeep);
    assert!(bolt_castable(&mut t, lib[0]));
}

#[test]
fn furious_rise_card_stays_playable_after_furious_rise_leaves() {
    cr!("611.2a", "611.2b");
    supported("Furious Rise");
    ruling!(
        "Furious Rise",
        "If Furious Rise leaves the battlefield before you play the most recently exiled card, you can play that card for as long as it remains exiled."
    );
    let mut t = TestGame::new(2);
    let lib = on_top(
        &mut t,
        P0,
        &["Lightning Bolt", "Wastes", "Wastes", "Wastes"],
    );
    let fr = t.battlefield(P0, "Furious Rise");
    t.battlefield(P0, "Craw Wurm");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.zone(lib[0]), Zone::Exile);
    destroy(&mut t, fr);
    t.advance_to(P1, Step::Upkeep);
    assert!(bolt_castable(&mut t, lib[0]));
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(bolt_castable(&mut t, lib[0]));
}

#[test]
fn redcap_gutter_dweller_trigger_resolves_after_it_leaves() {
    cr!("603.4", "608.2h", "113.7a");
    supported("Redcap Gutter-Dweller");
    ruling!(
        "Redcap Gutter-Dweller",
        "If Redcap Gutter-Dweller leaves the battlefield after its last ability has triggered, you can still sacrifice a creature and exile the top card of your library, even though you won't put a +1/+1 counter on Redcap Gutter-Dweller."
    );
    let mut t = TestGame::new(2);
    let lib = on_top(
        &mut t,
        P0,
        &["Lightning Bolt", "Wastes", "Wastes", "Wastes"],
    );
    let r = t.battlefield(P0, "Redcap Gutter-Dweller");
    let m = t.battlefield(P0, "Memnite");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, r);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(m)]);
    t.resolve_all();
    assert_eq!(t.zone(m), Zone::Graveyard(P0));
    assert_eq!(t.zone(lib[0]), Zone::Exile);
    assert_eq!(t.counters(t.g.current(r), counters::PLUS1), 0);
    assert!(bolt_castable(&mut t, lib[0]));
}

#[test]
fn a_new_raphael_cant_play_cards_the_old_one_exiled() {
    cr!("400.7", "607.2a");
    supported("Raphael, Most Attitude");
    ruling!(
        "Raphael, Most Attitude",
        "If Raphael, Most Attitude leaves the battlefield, and later another Raphael, Most Attitude enters the battlefield, it is a new object (even if the two were represented by the same card). You can't play cards exiled by the original Raphael, Most Attitude with the permission granted by the last ability of the new Raphael, Most Attitude."
    );
    // The same Raphael card: it exiles a card (alliance), dies, returns to the
    // battlefield, and attacks.
    let mut t = TestGame::new(2);
    let lib = on_top(&mut t, P0, &["Island", "Wastes", "Wastes", "Wastes"]);
    let r = t.battlefield(P0, "Raphael, Most Attitude");
    t.answer_yes(P0, true);
    t.enter(P0, "Memnite");
    t.resolve_all();
    assert_eq!(t.zone(lib[0]), Zone::Exile);
    destroy(&mut t, r);
    let card = t.g.current(r);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    let r2 =
        t.g.move_object(card, Zone::Battlefield, MoveCause::Effect, Some(P0))
            .unwrap();
    t.g.objects[r2.0 as usize].summoning_sick = false;
    t.settle();
    attack(&mut t, &[r2]);
    to_main(&mut t);
    assert!(!can_play_land(&mut t, P0, lib[0]));

    // Control: the original Raphael attacking lets P0 play it.
    let mut t = TestGame::new(2);
    let lib = on_top(&mut t, P0, &["Island", "Wastes", "Wastes", "Wastes"]);
    let r = t.battlefield(P0, "Raphael, Most Attitude");
    t.answer_yes(P0, true);
    t.enter(P0, "Memnite");
    t.resolve_all();
    attack(&mut t, &[r]);
    to_main(&mut t);
    assert!(can_play_land(&mut t, P0, lib[0]));
}
