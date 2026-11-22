//! CR 701.64: harness.

use crate::a701_028_071_common::*;
use mtg_engine::ability::*;
use mtg_engine::events::MoveCause;
use mtg_engine::kwa::harness::HARNESSED;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn harnessed(t: &TestGame, id: ObjectId) -> bool {
    mtg_engine::kwa::harness::is_harnessed(&t.g, t.g.current(id))
}

fn has_end_step_trigger(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).chars.abilities.iter().any(|a| {
        matches!(&a.kind, AbilityKind::Triggered(tr)
            if matches!(tr.trigger, TriggerCond::BeginningOf { step: TriggerStep::End, .. }))
    })
}

#[test]
fn harness_makes_a_permanent_harnessed_once() {
    cr!("701.64a");
    supported("The Mind Stone");
    // "{5}{W}, {T}: Harness The Mind Stone." and "∞ — At the beginning of your end step,
    // exile up to one other target nonland permanent you control, then return that card
    // to the battlefield under its owner's control."
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "The Mind Stone");
    assert!(!harnessed(&t, stone));
    assert!(!has_end_step_trigger(&t, stone));
    t.lands(P0, "Plains", 6);
    // Index 1: the harness ability (index 0 is the mana ability).
    t.activate(P0, stone, 1, &[]).unwrap();
    t.resolve_all();
    assert!(harnessed(&t, stone));
    assert_eq!(custom_events(&t, HARNESSED).len(), 1);
    // Its ∞ ability is active: at the beginning of the end step, it flickers a permanent.
    assert!(has_end_step_trigger(&t, stone));
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.g.is_live(bears));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Harnessing a harnessed permanent does nothing.
    run(
        &mut t,
        P0,
        Some(stone),
        ka(KeywordAction::Harness, Sel::This, 1),
        &[],
    );
    assert_eq!(custom_events(&t, HARNESSED).len(), 1);
}

#[test]
fn harnessed_is_a_designation_of_the_permanent_until_it_leaves() {
    cr!("701.64b");
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "The Mind Stone");
    run(
        &mut t,
        P0,
        Some(stone),
        ka(KeywordAction::Harness, Sel::This, 1),
        &[],
    );
    assert!(harnessed(&t, stone));
    // Not an ability: losing all abilities doesn't change it (though the ∞ ability, which
    // the permanent has because it's harnessed, is lost with the others).
    run(
        &mut t,
        P1,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveAllAbilities],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(stone)],
    );
    assert!(harnessed(&t, stone));
    assert!(!has_end_step_trigger(&t, stone));
    // Not a copiable value: a copy of it isn't harnessed.
    let copy = run(
        &mut t,
        P0,
        None,
        Effect::CreateTokenCopy {
            of: Sel::Target(0),
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
            mods: vec![],
        },
        &[Entity::Object(stone)],
    )
    .var_objects(vars::CREATED);
    assert_eq!(copy.len(), 1);
    assert!(!harnessed(&t, copy[0]));
    assert!(!has_end_step_trigger(&t, copy[0]));
    // A card that isn't a permanent can't be harnessed; one that leaves the battlefield
    // stops being harnessed.
    let gy = t
        .g
        .move_object(stone, Zone::Graveyard(P0), MoveCause::Effect, None)
        .unwrap();
    assert!(!harnessed(&t, gy));
    run(
        &mut t,
        P0,
        Some(gy),
        ka(KeywordAction::Harness, Sel::This, 1),
        &[],
    );
    assert!(!harnessed(&t, gy));
    let back = t
        .g
        .move_object(gy, Zone::Battlefield, MoveCause::Effect, Some(P0))
        .unwrap();
    t.g.recompute();
    assert!(!harnessed(&t, back));
}
