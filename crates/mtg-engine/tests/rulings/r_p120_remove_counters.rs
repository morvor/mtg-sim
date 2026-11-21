//! Rulings batch P120 — "remove a [kind] counter from target ..." (CR 122.1, 115.1):
//! Bloodcrazed Hoplite's per-counter trigger (CR 603.2c) and the other cards whose text
//! compiles with the targeted removal: Chainbreaker, Woeleecher ("if you do", CR 608.2c),
//! Decimator Beetle and Clash of the Eikons (lore counters on a Saga, CR 714.2b).

use crate::r_p120_common::*;
use crate::r_s29_common::damage_marked;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn bloodcrazed_hoplite_triggers_once_for_each_counter_put() {
    cr!("603.2c", "122.6");
    ruling!(
        "Bloodcrazed Hoplite",
        "If multiple +1/+1 counters are placed on Bloodcrazed Hoplite simultaneously, it last ability will trigger once for each of those counters."
    );
    supported("Bloodcrazed Hoplite");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Bloodcrazed Hoplite");
    let theirs = t.battlefield(P1, "Hill Giant");
    give_plus1(&mut t, theirs, 3);
    let mine = t.battlefield(P0, "Grizzly Bears");
    give_plus1(&mut t, mine, 1);
    t.answer_targets(P0, &[obj(theirs)]);
    t.answer_targets(P0, &[obj(theirs)]);
    give_plus1(&mut t, h, 2);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(plus1(&t, theirs), 1);
    // Only a creature an opponent controls can be the target.
    let from = t.asked().len();
    give_plus1(&mut t, h, 1);
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(cands[0].contains(&obj(theirs)));
    assert!(!cands[0].contains(&obj(mine)));
    assert!(!cands[0].contains(&obj(h)));
}

#[test]
fn bloodcrazed_hoplite_heroic_then_removes_a_counter() {
    cr!("603.2");
    supported("Giant Growth");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Bloodcrazed Hoplite");
    let theirs = t.battlefield(P1, "Hill Giant");
    give_plus1(&mut t, theirs, 1);
    cast_new(&mut t, P0, "Giant Growth", &[obj(h)]);
    t.answer_targets(P0, &[obj(theirs)]);
    t.resolve_all();
    assert_eq!(plus1(&t, h), 1);
    assert_eq!(plus1(&t, theirs), 0);
}

#[test]
fn chainbreaker_removes_a_minus_counter_from_target_creature() {
    cr!("122.1");
    supported("Chainbreaker");
    let mut t = TestGame::new(2);
    let cb = t.enter(P0, "Chainbreaker");
    assert_eq!(t.counters(cb, MINUS1), 2);
    assert_eq!(t.pt(cb), (1, 1));
    // Next turn it can tap: remove one of its own -1/-1 counters.
    t.g.objects[cb.0 as usize].summoning_sick = false;
    t.lands(P0, "Wastes", 3);
    t.activate(P0, cb, 0, &[obj(cb)]).expect("activate");
    t.resolve_all();
    assert_eq!(t.counters(cb, MINUS1), 1);
    assert_eq!(t.pt(cb), (2, 2));
}

#[test]
fn woeleecher_gains_life_only_if_a_counter_was_removed() {
    cr!("608.2c");
    supported("Woeleecher");
    for has_counter in [true, false] {
        let mut t = TestGame::new(2);
        let w = t.battlefield(P0, "Woeleecher");
        let target = t.battlefield(P1, "Hill Giant");
        if has_counter {
            give_minus1(&mut t, target, 1);
        }
        t.lands(P0, "Plains", 1);
        t.activate(P0, w, 0, &[obj(target)]).expect("activate");
        t.resolve_all();
        assert_eq!(t.counters(target, MINUS1), 0);
        assert_eq!(
            t.life(P0),
            if has_counter { 22 } else { 20 },
            "has_counter = {has_counter}"
        );
    }
}

#[test]
fn decimator_beetle_moves_a_minus_counter_when_it_attacks() {
    cr!("508.1m", "115.1");
    supported("Decimator Beetle");
    let mut t = TestGame::new(2);
    let beetle = t.battlefield(P0, "Decimator Beetle");
    let mine = t.battlefield(P0, "Hill Giant");
    give_minus1(&mut t, mine, 1);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(mine)]);
    t.answer_targets(P0, &[obj(theirs)]);
    attack_with(&mut t, &[(beetle, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.counters(mine, MINUS1), 0);
    assert_eq!(t.counters(theirs, MINUS1), 1);
    assert_eq!(t.pt(theirs), (1, 1));
}

/// P0 casts Clash of the Eikons choosing `modes`, with the given targets.
fn clash(t: &mut TestGame, modes: &[usize], targets: &[Entity]) {
    lands_for_cost(t, P0, "Clash of the Eikons");
    let card = t.hand(P0, "Clash of the Eikons");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(modes.to_vec()));
    for e in targets {
        t.answer_targets(P0, &[*e]);
    }
    t.cast(P0, card).go();
    t.resolve_all();
}

#[test]
fn clash_of_the_eikons_each_mode() {
    cr!("700.2", "714.2b", "701.14a");
    supported("Clash of the Eikons");
    supported("History of Benalia");
    // Fight.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    clash(&mut t, &[0], &[obj(mine), obj(theirs)]);
    assert_eq!(damage_marked(&t, mine), 2);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // Remove a lore counter: no chapter ability triggers.
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "History of Benalia");
    put_counters(&mut t, saga, "lore", 2);
    t.resolve_all();
    let knights = tokens_named(&t, P0, "Knight");
    clash(&mut t, &[1], &[obj(saga)]);
    assert_eq!(t.counters(saga, "lore"), 1);
    assert_eq!(tokens_named(&t, P0, "Knight"), knights);
    // Put a lore counter: chapter II triggers.
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "History of Benalia");
    put_counters(&mut t, saga, "lore", 1);
    t.resolve_all();
    let knights = tokens_named(&t, P0, "Knight");
    clash(&mut t, &[2], &[obj(saga)]);
    assert_eq!(t.counters(saga, "lore"), 2);
    assert_eq!(tokens_named(&t, P0, "Knight"), knights + 1);
}

/// Heroic's "put a +1/+1 counter on it": "it" is the creature the spell targets, not the
/// spell (a regression test: the counter used to go on the spell).
#[test]
fn heroic_it_is_the_creature() {
    cr!("603.2", "122.6");
    for name in ["Fabled Hero", "Lagonna-Band Trailblazer", "Favored Hoplite"] {
        supported(name);
        let mut t = TestGame::new(2);
        let hero = t.battlefield(P0, name);
        cast_new(&mut t, P0, "Giant Growth", &[obj(hero)]);
        t.resolve();
        assert_eq!(plus1(&t, hero), 1, "{name}");
        t.resolve_all();
    }
    // Favored Hoplite: "and prevent all damage that would be dealt to it this turn".
    let mut t = TestGame::new(2);
    let hoplite = t.battlefield(P0, "Favored Hoplite");
    cast_new(&mut t, P0, "Giant Growth", &[obj(hoplite)]);
    t.resolve_all();
    let src = t.battlefield(P1, "Hill Giant");
    crate::r_s06_common::damage(&mut t, src, 10, hoplite);
    assert!(t.on_battlefield(hoplite));
    assert_eq!(damage_marked(&t, hoplite), 0);
}
