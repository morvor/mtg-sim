//! Rulings batch S21 — "Whenever this creature becomes blocked" (CR 509.1h, 509.3c,
//! 603.2c, 603.2e): it triggers once however many creatures block it, and also when an
//! effect makes it blocked; the Laccolith's damage can target any creature.

use crate::r_s01_common::*;
use crate::r_s02_common::target_candidates;
use crate::r_s03_common::to_blockers;
use crate::r_s20_common::to_beginning_of_combat;
use crate::r_s21_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn the_laccolith_damage_can_target_a_creature_that_isnt_blocking_it() {
    cr!("509.3c", "115.1");
    ruling!(
        "Laccolith Grunt",
        "It can target any creature on the battlefield, not just one of the ones blocking it."
    );
    supported("Laccolith Grunt");
    let mut t = TestGame::new(2);
    // "Whenever this creature becomes blocked, you may have it deal damage equal to its
    // power to target creature. If you do, this creature assigns no combat damage this
    // turn."
    let grunt = t.battlefield(P0, "Laccolith Grunt");
    let mine = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    to_beginning_of_combat(&mut t, P0);
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(elves)]);
    t.answer_yes(P0, true);
    to_blockers(&mut t, &[(grunt, Entity::Player(P1))], &[(bears, grunt)]);
    // Every creature on the battlefield could be targeted, blocking or not.
    let cands = target_candidates(&t, P0, from);
    assert_eq!(cands.len(), 1);
    for c in [grunt, mine, bears, elves] {
        assert!(cands[0].contains(&Entity::Object(c)));
    }
    assert_eq!(triggers_on_stack(&t, "becomes blocked"), 1);
    t.resolve_all();
    // The Elves, which weren't blocking, took 2 damage.
    assert!(!t.on_battlefield(elves));
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    // The Grunt assigns no combat damage: the blocking Bears survive combat.
    t.advance_to(P0, Step::EndOfCombat);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn the_laccolith_ability_triggers_when_an_effect_makes_it_blocked() {
    cr!("509.1h", "509.3c", "603.2e");
    ruling!(
        "Laccolith Whelp",
        "The ability triggers even if a spell or ability makes it blocked, instead of being blocked by a creature."
    );
    supported("Laccolith Whelp");
    supported("Curtain of Light");
    let mut t = TestGame::new(2);
    let whelp = t.battlefield(P0, "Laccolith Whelp");
    let elves = t.battlefield(P1, "Llanowar Elves");
    to_beginning_of_combat(&mut t, P0);
    // No creature blocks it.
    to_blockers(&mut t, &[(whelp, Entity::Player(P1))], &[]);
    assert!(t.g.combat.as_ref().unwrap().is_unblocked(whelp));
    assert_eq!(triggers_on_stack(&t, "becomes blocked"), 0);
    // P1 casts Curtain of Light: "Target unblocked attacking creature becomes blocked."
    let curtain = t.hand(P1, "Curtain of Light");
    t.lands(P1, "Plains", 2);
    t.cast(P1, curtain).target(whelp).go();
    t.answer_targets(P0, &[Entity::Object(elves)]);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(t.g.combat.as_ref().unwrap().is_blocked(whelp));
    assert!(t.g.combat.as_ref().unwrap().blockers_of(whelp).is_empty());
    assert_eq!(triggers_on_stack(&t, "becomes blocked"), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(elves), "the Whelp dealt 1 damage to the Elves");
    // Blocked by no creature, it deals no combat damage to P1.
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn alley_grifters_triggers_once_when_blocked_by_two_creatures() {
    cr!("509.3c", "603.2c");
    ruling!(
        "Alley Grifters",
        "It triggers only once even if blocked by more than one creature."
    );
    supported("Alley Grifters");
    let mut t = TestGame::new(2);
    // "Whenever this creature becomes blocked, defending player discards a card."
    let grifters = t.battlefield(P0, "Alley Grifters");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Llanowar Elves");
    for _ in 0..3 {
        t.hand(P1, "Island");
    }
    to_beginning_of_combat(&mut t, P0);
    to_blockers(
        &mut t,
        &[(grifters, Entity::Player(P1))],
        &[(a, grifters), (b, grifters)],
    );
    assert_eq!(t.g.combat.as_ref().unwrap().blockers_of(grifters).len(), 2);
    assert_eq!(triggers_on_stack(&t, "becomes blocked"), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 2, "P1 discarded one card");
}

#[test]
fn plague_wight_gives_each_of_its_blockers_minus_1_minus_1_once() {
    cr!("509.3c", "603.2c");
    ruling!(
        "Plague Wight",
        "An ability that triggers when a creature becomes blocked triggers only once if two or more creatures block it."
    );
    supported("Plague Wight");
    let mut t = TestGame::new(2);
    // "Whenever this creature becomes blocked, each creature blocking it gets -1/-1 until
    // end of turn."
    let wight = t.battlefield(P0, "Plague Wight");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    to_beginning_of_combat(&mut t, P0);
    to_blockers(
        &mut t,
        &[(wight, Entity::Player(P1))],
        &[(a, wight), (b, wight)],
    );
    assert_eq!(triggers_on_stack(&t, "becomes blocked"), 1);
    t.resolve_all();
    // Each Bears got -1/-1 once (not twice: they'd be 0/0).
    assert_eq!(t.pt(a), (1, 1));
    assert_eq!(t.pt(b), (1, 1));
    assert!(t.on_battlefield(a) && t.on_battlefield(b));
}
