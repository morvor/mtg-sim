//! Rulings batch S21 — "This turn, when target creature you control attacks and isn't
//! blocked, ..." (Delif's Cone): a delayed triggered ability (CR 603.7) that triggers as
//! blockers are declared (CR 509.3g); created afterward, it doesn't trigger that combat.

use crate::r_s01_common::*;
use crate::r_s20_common::to_beginning_of_combat;
use crate::r_s21_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 attacks P1 with Grizzly Bears (no blocks); P0 activates Delif's Cone targeting the
/// Bears in the declare attackers step (`before`) or in the declare blockers step, saying
/// yes to gaining life. Returns (P0's life, P1's life) after combat.
fn cone(before: bool) -> (i32, i32) {
    supported("Delif's Cone");
    let mut t = TestGame::new(2);
    // "{T}, Sacrifice this artifact: This turn, when target creature you control attacks
    // and isn't blocked, you may gain life equal to its power. If you do, it assigns no
    // combat damage this turn."
    let cone = t.battlefield(P0, "Delif's Cone");
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    let activate = |t: &mut TestGame| {
        t.activate(P0, cone, 0, &[Entity::Object(bears)])
            .expect("activate");
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Delif's Cone"));
    };
    if before {
        activate(&mut t);
    }
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.answer_yes(P0, true);
    go_to(&mut t, Step::DeclareBlockers);
    if !before {
        assert!(t.g.combat.as_ref().unwrap().is_unblocked(bears));
        activate(&mut t);
    }
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    (t.life(P0), t.life(P1))
}

#[test]
fn delifs_cone_used_before_blockers_triggers_as_the_creature_is_unblocked() {
    cr!("603.7a", "603.7b", "509.3g");
    // The Bears weren't blocked: P0 gains 2 life and the Bears deal no combat damage.
    assert_eq!(cone(true), (22, 20));
}

#[test]
fn delifs_cone_used_after_blockers_are_declared_never_triggers() {
    cr!("603.7a", "509.3g");
    ruling!(
        "Delif's Cone",
        "If you use the ability after blockers are declared, it won’t trigger at all. So you want to use it before blockers are declared."
    );
    // Too late: no life gained, and the Bears deal their combat damage.
    assert_eq!(cone(false), (20, 18));
}
