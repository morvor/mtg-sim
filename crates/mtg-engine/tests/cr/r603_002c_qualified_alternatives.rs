//! CR 603.2, 603.2c: a trigger condition with alternatives, one of them a "one or more"
//! (batched) event, qualified as a whole ("[A] or [B] during your turn"): the qualifier
//! applies to each alternative, and the batched one triggers once for its batch.

use crate::r600_common::{abilities, compile_def, CB};
use crate::r603_audit_common::triggers_of;
use mtg_engine::ability::*;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// "Whenever this creature becomes tapped or is dealt damage during your turn, you gain 1
/// life." on a 1/4. (No card in the pool qualifies alternatives with a batched one;
/// Cryoshatter's "becomes tapped or is dealt damage" has no qualifier.)
fn sentry() -> CardDef {
    let compiled = compile_def(
        "Watchful Sentry",
        "Creature — Human Soldier",
        "{1}{W}",
        "Whenever this creature becomes tapped or is dealt damage during your turn, you gain 1 life.",
    );
    assert!(
        compiled.is_fully_supported(),
        "{:?}",
        compiled.unsupported_text()
    );
    let abs = abilities(&compiled);
    assert_eq!(abs.len(), 1, "{abs:?}");
    CB::new("Watchful Sentry")
        .creature(1, 4)
        .ability(abs[0].clone())
        .build()
}

#[test]
fn a_qualified_trigger_with_a_one_or_more_alternative_triggers_once_for_its_batch() {
    cr!("603.2", "603.2c");
    let mut t = TestGame::new(2);
    let s = t.custom(P0, sentry(), Zone::Battlefield);
    // Your turn, tapped: the per-event alternative (the control).
    t.g.tap(s);
    t.settle();
    assert_eq!(triggers_of(&t, "Watchful Sentry"), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    // Your turn, dealt damage by two sources at once: the batched alternative, once.
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage_batch(
        vec![(a, Entity::Object(s), 1), (b, Entity::Object(s), 1)],
        false,
    );
    t.settle();
    assert_eq!(
        triggers_of(&t, "Watchful Sentry"),
        1,
        "dealt damage during your turn"
    );
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // An opponent's turn: the qualifier still applies to the batched alternative.
    t.set_step(P1, Step::PrecombatMain);
    t.g.deal_damage(a, Entity::Object(s), 1, false);
    t.settle();
    assert_eq!(triggers_of(&t, "Watchful Sentry"), 0);
}

#[test]
fn a_qualified_alternative_with_a_condition_on_its_batch_keeps_it() {
    cr!("603.2", "603.2c");
    // "Whenever this creature becomes tapped or is dealt 3 or more damage during your turn,
    // you gain 1 life.": the second alternative is the damage dealt to it at once, in total
    // (a condition on its whole batch), qualified like the first. (The compiler doesn't read
    // that text; the trigger is the qualified one above with the batch condition "is dealt
    // damage" gets in "is dealt 3 or more damage".)
    let abs = abilities(&sentry());
    let AbilityKind::Triggered(tr) = &abs[0].kind else {
        panic!("{abs:?}");
    };
    let TriggerCond::AnyOf(alts) = &tr.trigger else {
        panic!("{:?}", tr.trigger);
    };
    assert!(
        matches!(&alts[1], TriggerCond::Batched { trigger, .. }
            if matches!(**trigger, TriggerCond::Where { cond: Condition::YourTurn, .. })),
        "{alts:?}"
    );
    let three_or_more = TriggerCond::Where {
        trigger: Box::new(alts[1].clone()),
        cond: Condition::Compare(Value::EventAmount, Cmp::Ge, Value::Const(3)),
    };
    let mut ta = tr.clone();
    ta.trigger = TriggerCond::AnyOf(vec![alts[0].clone(), three_or_more]);
    let def = CB::new("Stalwart Sentry")
        .creature(1, 10)
        .ability(AbilityDef::new(AbilityKind::Triggered(ta), "triggered"))
        .build();
    let mut t = TestGame::new(2);
    let s = t.custom(P0, def, Zone::Battlefield);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    // 2 damage at once: not 3 or more.
    t.g.deal_damage_batch(
        vec![(a, Entity::Object(s), 1), (b, Entity::Object(s), 1)],
        false,
    );
    t.settle();
    assert_eq!(triggers_of(&t, "Stalwart Sentry"), 0);
    // 3 damage at once, from two sources: once.
    t.g.deal_damage_batch(
        vec![(a, Entity::Object(s), 2), (b, Entity::Object(s), 1)],
        false,
    );
    t.settle();
    assert_eq!(triggers_of(&t, "Stalwart Sentry"), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    // An opponent's turn: not during your turn.
    t.set_step(P1, Step::PrecombatMain);
    t.g.deal_damage(a, Entity::Object(s), 3, false);
    t.settle();
    assert_eq!(triggers_of(&t, "Stalwart Sentry"), 0);
}
