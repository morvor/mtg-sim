//! Rulings batch P080 — spells and abilities for attacking creatures whose target became
//! illegal: none of their other effects happen (CR 608.2b), and a legal target that
//! survives still lets the rest happen.

use crate::r_p079_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s02_common::destroy;
use crate::r_s06_common::activate_containing;
use crate::r_s13_common::add as add_counters;
use mtg_engine::combat::remove_from_combat;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0's Grizzly Bears attack P1 (declare attackers step, P1 to act).
fn bears_attack(t: &mut TestGame) -> ObjectId {
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    bears
}

#[test]
fn spells_for_attackers_do_nothing_else_with_an_illegal_target() {
    cr!("608.2b", "506.4");
    ruling!(
        "Bright Reprisal",
        "If the target creature is an illegal target by the time Bright Reprisal resolves, the entire spell doesn’t resolve. You won’t draw a card."
    );
    ruling!(
        "Sonar Strike",
        "If the target creature is an illegal target by the time Sonar Strike tries to resolve, the spell doesn’t resolve."
    );
    ruling!(
        "Joust Through",
        "If the target creature is an illegal target when Joust Through tries to resolve, it won't resolve and none of its effects will happen. You won't gain life."
    );
    ruling!(
        "Nemesis Trap",
        "If the targeted creature is an illegal target by the time Nemesis Trap resolves, the entire spell doesn't resolve. You won't get a token."
    );
    ruling!(
        "Second Thoughts",
        "If the targeted creature is an illegal target when Second Thoughts tries to resolve, the entire spell doesn’t resolve. You won’t draw a card."
    );
    // (card, P1's hand size gain, P1's life gain, P1's tokens) when it resolves.
    let cases: [(&str, usize, i32, usize); 5] = [
        ("Bright Reprisal", 1, 0, 0),
        ("Sonar Strike", 0, 3, 0),
        ("Joust Through", 0, 1, 0),
        ("Nemesis Trap", 0, 0, 1),
        ("Second Thoughts", 1, 0, 0),
    ];
    for (name, draw, gain, tokens) in cases {
        supported(name);
        // Removed from combat (no longer attacking) or gone altogether.
        for illegal in [None, Some(false), Some(true)] {
            // Sonar Strike can also target a tapped creature: one removed from combat
            // is still legal.
            if name == "Sonar Strike" && illegal == Some(false) {
                continue;
            }
            let mut t = TestGame::new(2);
            // Sonar Strike's life gain needs a Bat.
            t.battlefield(P1, "Vampire Bats");
            let bears = bears_attack(&mut t);
            cast_card(&mut t, P1, name, &[obj(bears)]).unwrap();
            let hand = t.hand_size(P1);
            match illegal {
                Some(false) => remove_from_combat(&mut t.g, bears),
                Some(true) => destroy(&mut t, bears),
                None => {}
            }
            t.resolve_all();
            let label = format!("{name} illegal: {illegal:?}");
            let ok = illegal.is_none();
            assert_eq!(t.hand_size(P1), hand + if ok { draw } else { 0 }, "{label}");
            assert_eq!(t.life(P1), 20 + if ok { gain } else { 0 }, "{label}");
            assert_eq!(
                crate::r_s01_common::tokens(&t, P1).len(),
                if ok { tokens } else { 0 },
                "{label}"
            );
            if illegal == Some(false) {
                assert!(t.on_battlefield(bears), "{label}: untouched");
            }
        }
    }
}

#[test]
fn bright_reprisal_draws_when_the_legal_target_survives() {
    cr!("608.2b", "702.12b");
    ruling!(
        "Bright Reprisal",
        "If, on the other hand, the target is a legal target but isn’t destroyed (most likely because it has indestructible), you’ll draw a card."
    );
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P0, "Darksteel Myr");
    attack_with(&mut t, &[(myr, Entity::Player(P1))]);
    t.resolve_all();
    cast_card(&mut t, P1, "Bright Reprisal", &[obj(myr)]).unwrap();
    let hand = t.hand_size(P1);
    t.resolve_all();
    assert!(t.on_battlefield(myr));
    assert_eq!(t.hand_size(P1), hand + 1);
}

#[test]
fn sonar_strike_without_its_target_gains_no_life_even_with_a_bat() {
    cr!("608.2b");
    ruling!(
        "Sonar Strike",
        "You won’t gain life even if you control a Bat."
    );
    // A tapped (non-attacking) creature is a legal target too; destroyed in response.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vampire Bats");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.objects[giant.0 as usize].tapped = true;
    cast_card(&mut t, P0, "Sonar Strike", &[obj(giant)]).unwrap();
    destroy(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn arcbound_javelineer_deals_no_damage_to_a_creature_removed_from_combat() {
    cr!("608.2b", "506.4", "107.3a");
    ruling!(
        "Arcbound Javelineer",
        "If the target is illegal as the activated ability tries to resolve, perhaps because it was removed from combat, no damage will be dealt."
    );
    supported("Arcbound Javelineer");
    for removed in [false, true] {
        let mut t = TestGame::new(2);
        let jav = t.battlefield(P1, "Arcbound Javelineer");
        add_counters(&mut t, jav, "+1/+1", 2);
        let giant = t.battlefield(P0, "Hill Giant");
        attack_with(&mut t, &[(giant, Entity::Player(P1))]);
        t.resolve_all();
        t.answer(P1, DecisionKind::X, Answer::Number(2));
        t.answer_targets(P1, &[obj(giant)]);
        activate_containing(&mut t, P1, jav, "Remove X").expect("activated");
        assert_eq!(t.counters(jav, "+1/+1"), 0, "the cost was paid");
        if removed {
            remove_from_combat(&mut t.g, giant);
        }
        t.resolve_all();
        assert_eq!(
            crate::r_s07_common::damage_on(&t, giant),
            if removed { 0 } else { 2 },
            "removed: {removed}"
        );
    }
}

#[test]
fn argent_dais_illegal_target_no_one_draws() {
    cr!("608.2b");
    ruling!(
        "Argent Dais",
        "If the target nonland permanent is illegal as Argent Dais's last ability tries to resolve, it won't resolve and none of its effects will happen. No player will draw cards."
    );
    supported("Argent Dais");
    for illegal in [false, true] {
        let mut t = TestGame::new(2);
        let dais = t.battlefield(P0, "Argent Dais");
        add_counters(&mut t, dais, "oil", 2);
        t.lands(P0, "Wastes", 2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[obj(bears)]);
        activate_containing(&mut t, P0, dais, "Exile another").expect("activated");
        if illegal {
            destroy(&mut t, bears);
        }
        let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
        t.resolve_all();
        assert_eq!(t.hand_size(P0), h0);
        assert_eq!(t.hand_size(P1), h1 + if illegal { 0 } else { 2 });
        if !illegal {
            assert!(t.in_exile("Grizzly Bears"));
        }
    }
}

#[test]
fn scriv_creates_no_contract_without_a_legal_target() {
    cr!("608.2b", "303.4f");
    ruling!(
        "Scriv, the Obligator",
        "If the target creature is an illegal target as Scriv's last ability tries to resolve, it won't resolve and none of its effects will happen. The Contract token won't be created."
    );
    supported("Scriv, the Obligator");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Scriv, the Obligator");
    t.settle();
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.named_on_battlefield("Contract").is_empty());
    assert!(t.g.exile.iter().all(|e| t.g.obj(*e).chars.name != "Contract"));
}

#[test]
fn bury_in_books_with_an_empty_library_puts_the_creature_on_top() {
    cr!("401.7");
    ruling!(
        "Bury in Books",
        "If the target creature's controller has zero cards left in their library when Bury in Books resolves, that creature becomes the top card of their library."
    );
    supported("Bury in Books");
    let mut t = TestGame::new(2);
    crate::r_s11_common::empty_library(&mut t, P1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_card(&mut t, P0, "Bury in Books", &[obj(bears)]).unwrap();
    t.resolve_all();
    let lib = &t.g.player(P1).library;
    assert_eq!(lib.len(), 1);
    assert_eq!(t.g.obj(*lib.last().unwrap()).chars.name, "Grizzly Bears");
}
