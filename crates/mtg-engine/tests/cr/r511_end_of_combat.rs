//! CR 511: the end of combat step.

use crate::r506_common::*;
use mtg_engine::ability::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn end_of_combat_has_no_turn_based_actions_and_active_player_gets_priority() {
    cr!("511.1");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, &[(a, Entity::Player(P1))]);
    go_to(&mut t, Step::CombatDamage);
    let before = t.g.turn_events.len();
    let order = priority_order_in(&mut t, Step::EndOfCombat);
    assert_eq!(order, vec![P0, P1]);
    // Nothing happened as the step began besides the step beginning itself.
    let evs: Vec<_> = t.g.turn_events[before..]
        .iter()
        .filter(|e| !matches!(e, mtg_engine::events::Event::StepBegan { .. }))
        .collect();
    assert!(evs.is_empty(), "{evs:?}");
}

#[test]
fn at_end_of_combat_triggers_and_until_end_of_combat_effects_expire() {
    cr!("511.2");
    let mut t = TestGame::new(2);
    // "At end of combat, you gain 1 life."
    bf(
        &mut t,
        P0,
        custom_card(
            "Victory Bell",
            "Artifact",
            None,
            "At end of combat, you gain 1 life.",
        ),
    );
    let a = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, &[(a, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    apply(
        &mut t,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::ModifyPT(Value::c(2), Value::c(2))],
            duration: Duration::EndOfCombat,
        },
        &[a],
    );
    // A delayed "at end of combat" trigger too.
    apply(
        &mut t,
        P0,
        Effect::AtNext {
            step: TriggerStep::EndOfCombat,
            effect: Box::new(gain(10)),
        },
        &[],
    );
    go_to(&mut t, Step::CombatDamage);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 16);
    go_to(&mut t, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P0), 31, "both triggered as the step began");
    // The +2/+2 lasts through the end of combat step...
    assert_eq!(t.pt(a), (4, 4));
    // ...and expires at the end of the combat phase.
    go_to(&mut t, Step::PostcombatMain);
    assert_eq!(t.pt(a), (2, 2));
    // "At end of combat" also triggers with no attackers.
    t.advance_to(P1, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P0), 32);
}

#[test]
fn everything_is_removed_from_combat_after_the_end_of_combat_step() {
    cr!("511.3");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Hill Giant");
    let b = t.battlefield(P0, "Grizzly Bears");
    let jace = t.battlefield(P1, "Jace Beleren");
    let x = t.battlefield(P1, "Craw Wurm");
    declare(
        &mut t,
        &[(a, Entity::Player(P1)), (b, Entity::Object(jace))],
    );
    block(&mut t, P1, &[(x, a)]);
    go_to(&mut t, Step::EndOfCombat);
    assert!(t.g.is_blocking(x));
    assert!(t.g.is_attacking(b));
    assert_eq!(attack_target(&t, b), Some(Entity::Object(jace)));
    go_to(&mut t, Step::PostcombatMain);
    assert!(t.g.combat.is_none());
    assert!(!t.g.is_attacking(b) && !t.g.is_blocking(x));
    let ctx = mtg_engine::eval::Ctx::new(None, P0);
    assert!(!t.g.matches(b, &Filter::Attacking, &ctx));
    assert!(!t.g.matches(x, &Filter::Blocking, &ctx));
    assert_eq!(t.g.turn.step, Step::PostcombatMain);
}
