//! Rulings batch P043 — Summon: Magus Sisters's random modes (CR 700.2, 714), Invert //
//! Invent, "whenever day becomes night or night becomes day" (CR 731.1a), and Colfenor's
//! Plans (cards exiled face down that may be played, CR 406.3, 601.3).

use crate::r_p007_common::*;
use crate::r_s01_common::{stack_library, supported};
use crate::r_s02_common::{can_cast, destroy};
use crate::r_s06_common::attach_new;
use crate::r_s25_common::cast_new;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Number of triggered abilities from `source` on the stack.
fn triggers_from(t: &TestGame, source: ObjectId) -> usize {
    t.g.stack
        .iter()
        .filter(|id| {
            let o = t.g.obj(**id);
            o.stack.as_ref().is_some_and(|si| {
                matches!(si.kind, mtg_engine::object::StackKind::Triggered { source: s, .. }
                    if t.g.current(s) == t.g.current(source))
            })
        })
        .count()
}

#[test]
fn magus_sisters_random_mode_needs_a_legal_target_and_comes_before_targets() {
    cr!("700.2b", "700.2c", "714.2b");
    ruling!(
        "Summon: Magus Sisters",
        "In the incredibly rare case where there is no legal target for the first or second mode, those modes can't be chosen at random. Since the third mode doesn't require a target, it is always eligible to be chosen."
    );
    ruling!(
        "Summon: Magus Sisters",
        "The random choice of mode for Summon: Magus Sisters's chapter abilities is made before targets are chosen."
    );
    supported("Summon: Magus Sisters");
    // With shroud (Lightning Greaves) and no other creature, only "Fight!" can be chosen.
    for seed in 0..8u64 {
        let mut t = TestGame::with_config(
            2,
            mtg_engine::game::GameConfig {
                seed,
                ..Default::default()
            },
        );
        let saga = t.battlefield(P0, "Summon: Magus Sisters");
        attach_new(&mut t, P0, "Lightning Greaves", saga);
        t.set_step(P0, Step::Upkeep);
        t.advance_to(P0, Step::PrecombatMain);
        t.settle();
        assert_eq!(t.counters(saga, "lore"), 1);
        assert_eq!(triggers_from(&t, saga), 1, "chapter I");
        t.resolve_all();
        assert_eq!(t.life(P0), 20, "seed {seed}: not Defense!");
        assert_eq!(
            t.counters(saga, "+1/+1"),
            0,
            "seed {seed}: not Combine Powers!"
        );
        assert_eq!(t.counters(saga, "shield"), 0);
    }
    // The mode is chosen first: one target choice, for that mode only.
    for seed in 0..8u64 {
        let mut t = TestGame::with_config(
            2,
            mtg_engine::game::GameConfig {
                seed,
                ..Default::default()
            },
        );
        let saga = t.battlefield(P0, "Summon: Magus Sisters");
        t.battlefield(P1, "Grizzly Bears");
        t.set_step(P0, Step::Upkeep);
        let from = t.asked().len();
        t.advance_to(P0, Step::PrecombatMain);
        t.settle();
        assert_eq!(triggers_from(&t, saga), 1, "chapter I");
        let asked = &t.asked()[from..];
        assert!(!asked
            .iter()
            .any(|(_, d)| matches!(d, Decision::ChooseModes { .. })));
        let target_choices = asked
            .iter()
            .filter(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
            .count();
        assert_eq!(target_choices, 1, "seed {seed}");
    }
}

#[test]
fn invert_switches_each_creature_separately() {
    cr!("613.4d");
    ruling!(
        "Invert // Invent",
        "If you chose two targets, Invert switches the power and toughness of one creature and it switches the power and toughness of another creature. It doesn't switch one creature's power with another creature's power."
    );
    supported("Invert // Invent");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let hawk = t.battlefield(P1, "Vampire Nighthawk");
    t.lands(P0, "Island", 1);
    let card = t.hand(P0, "Invert // Invent");
    t.cast(P0, card)
        .method(CastMethod::Half(0))
        .targets(&[obj(wurm), obj(hawk)])
        .go();
    t.resolve_all();
    assert_eq!(t.pt(wurm), (4, 6));
    assert_eq!(t.pt(hawk), (3, 2));
}

#[test]
fn invent_finds_an_instant_and_or_a_sorcery() {
    cr!("701.23a", "701.23b");
    ruling!(
        "Invert // Invent",
        "While resolving Invent, you could put no cards, an instant card, a sorcery card, or an instant card and a sorcery card into your hand."
    );
    for pick in [
        vec![],
        vec!["Shock"],
        vec!["Divination"],
        vec!["Shock", "Divination"],
    ] {
        let mut t = TestGame::new(2);
        let lib = stack_library(&mut t, P0, &["Shock", "Divination", "Grizzly Bears"]);
        t.lands(P0, "Island", 1);
        t.lands(P0, "Mountain", 1);
        t.lands(P0, "Wastes", 4);
        let card = t.hand(P0, "Invert // Invent");
        // An instant card, then a sorcery card.
        for (i, n) in ["Shock", "Divination"].iter().enumerate() {
            if pick.contains(n) {
                t.answer_choose(P0, &[obj(lib[i])]);
            } else {
                t.answer_choose(P0, &[]);
            }
        }
        t.cast(P0, card).method(CastMethod::Half(1)).go();
        t.resolve_all();
        for n in ["Shock", "Divination"] {
            assert_eq!(t.in_hand(P0, n), pick.contains(&n), "{pick:?}: {n}");
        }
        assert!(!t.in_hand(P0, "Grizzly Bears"));
    }
}

#[test]
fn day_night_triggers_dont_trigger_when_it_first_becomes_day() {
    cr!("731.1", "731.1a", "603.2");
    ruling!(
        "Brimstone Vandal",
        "It does not trigger when the game becomes day or night for the first time."
    );
    ruling!(
        "Celestus Sanctifier",
        "It does not trigger when the game becomes day or night for the first time."
    );
    ruling!(
        "Component Collector",
        "It does not trigger when the game becomes day or night for the first time."
    );
    ruling!(
        "Firmament Sage",
        "It does not trigger when the game becomes day or night for the first time."
    );
    ruling!(
        "Gavony Dawnguard",
        "It does not trigger when the game becomes day or night for the first time."
    );
    ruling!(
        "Obsessive Astronomer",
        "It does not trigger when the game becomes day or night for the first time."
    );
    ruling!(
        "Sunrise Cavalier",
        "It does not trigger when the game becomes day or night for the first time."
    );
    ruling!(
        "Sunstreak Phoenix",
        "It does not trigger when the game becomes day or night for the first time."
    );
    ruling!(
        "The Celestus",
        "It does not trigger when the game becomes day or night for the first time."
    );
    ruling!(
        "Vadrik, Astral Archmage",
        "It does not trigger when the game becomes day or night for the first time."
    );
    ruling!(
        "Firmament Sage",
        "If a triggered ability triggers whenever day becomes night or night becomes day, it only triggers if it was already either day/night and becomes the other."
    );
    for name in [
        "Brimstone Vandal",
        "Celestus Sanctifier",
        "Component Collector",
        "Firmament Sage",
        "Gavony Dawnguard",
        "Obsessive Astronomer",
        "Sunrise Cavalier",
        "Sunstreak Phoenix",
        "The Celestus",
        "Vadrik, Astral Archmage",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Grizzly Bears");
        assert_eq!(t.g.day, None);
        let id = t.enter(P0, name);
        t.settle();
        assert_eq!(t.g.day, Some(true), "{name}: it became day");
        assert_eq!(triggers_from(&t, id), 0, "{name}: no trigger");
        let id = if name == "Sunstreak Phoenix" {
            crate::r_s05_common::move_to(&mut t, id, Zone::Graveyard(P0)).unwrap()
        } else {
            id
        };
        t.g.set_day(false);
        t.settle();
        assert_eq!(triggers_from(&t, id), 1, "{name}: day became night");
    }
}

/// P0 casts Colfenor's Plans with `top` on top of its library (seven cards); its trigger
/// resolves. Returns the exiled cards (followed).
fn plans(t: &mut TestGame, top: &[&str]) -> (ObjectId, Vec<ObjectId>) {
    supported("Colfenor's Plans");
    let lib = stack_library(t, P0, top);
    t.set_step(P0, Step::PrecombatMain);
    cast_new(t, P0, "Colfenor's Plans", &[]);
    t.resolve_all();
    let p = t.named_on_battlefield("Colfenor's Plans")[0];
    let exiled: Vec<ObjectId> = lib.iter().map(|c| t.g.current(*c)).collect();
    for c in &exiled {
        assert_eq!(t.zone(*c), Zone::Exile);
        assert!(t.obj_now(*c).face_down);
    }
    (p, exiled)
}

const SEVEN: [&str; 7] = [
    "Grizzly Bears",
    "Exalted Angel",
    "Shock",
    "Divination",
    "Island",
    "Island",
    "Crowd-Control Warden",
];

#[test]
fn colfenors_plans_no_more_spells_the_turn_it_was_cast() {
    cr!("101.2", "601.3");
    ruling!(
        "Colfenor's Plans",
        "The turn you cast Colfenor's Plans, you have cast (at least) one spell that turn. After it enters and its last ability goes into effect, you can't cast any more spells that turn."
    );
    let mut t = TestGame::new(2);
    let (_, exiled) = plans(&mut t, &SEVEN);
    t.lands(P0, "Mountain", 1);
    assert!(!can_cast(&mut t, P0, exiled[2], CastMethod::Normal));
    let shock = t.hand(P0, "Shock");
    assert!(!can_cast(&mut t, P0, shock, CastMethod::Normal));
    // Next turn: one spell.
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(can_cast(&mut t, P0, exiled[2], CastMethod::Normal));
}

#[test]
fn colfenors_plans_cards_are_played_normally_and_only_while_it_remains() {
    cr!("601.3", "702.37c", "702.168a", "406.3a", "406.3", "307.1");
    ruling!(
        "Colfenor's Plans",
        "Playing a card exiled with Colfenor's Plans follows all the normal rules for playing that card. You must pay its costs, and you must follow all timing restrictions, for example."
    );
    ruling!(
        "Colfenor's Plans",
        "If an exiled card has morph, you may cast it face down."
    );
    ruling!(
        "Colfenor's Plans",
        "If Colfenor's Plans leaves the battlefield, you can continue to look at cards exiled by it, but you can no longer play them."
    );
    supported("Crowd-Control Warden");
    let mut t = TestGame::new(2);
    let (p, exiled) = plans(&mut t, &SEVEN);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    // Divination (a sorcery) in the upkeep: no. Shock without mana: no.
    t.lands(P0, "Island", 3);
    assert!(
        !can_cast(&mut t, P0, exiled[3], CastMethod::Normal),
        "timing"
    );
    t.advance_to(P0, Step::PrecombatMain);
    assert!(can_cast(&mut t, P0, exiled[3], CastMethod::Normal));
    assert!(
        !can_cast(&mut t, P0, exiled[2], CastMethod::Normal),
        "{{R}} unpaid"
    );
    // Exalted Angel face down for {3} (or Crowd-Control Warden, with disguise).
    let morph = CastMethod::FaceDown(KeywordKind::Morph);
    let disguise = CastMethod::FaceDown(KeywordKind::Disguise);
    assert!(can_cast(&mut t, P0, exiled[6], disguise.clone()));
    assert!(can_cast(&mut t, P0, exiled[1], morph.clone()));
    t.cast(P0, exiled[1]).method(morph).go();
    t.resolve_all();
    let angel = t.g.current(exiled[1]);
    assert!(t.on_battlefield(angel) && t.obj_now(angel).face_down);
    // A land can be played from among them.
    let island = t.g.current(exiled[4]);
    t.play_land(P0, island).expect("play an exiled land");
    // Plans leaves: P0 may still look at them, but can't play them.
    destroy(&mut t, p);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    for c in [exiled[0], exiled[2], exiled[3]] {
        assert!(mtg_engine::facedown::can_look_at(&t.g, P0, t.g.current(c)));
        assert!(!mtg_engine::facedown::can_look_at(&t.g, P1, t.g.current(c)));
        assert!(!can_cast(&mut t, P0, c, CastMethod::Normal));
    }
    assert!(!can_cast(&mut t, P0, exiled[6], disguise));
    let island = t.g.current(exiled[5]);
    assert!(t.play_land(P0, island).is_err());
}
