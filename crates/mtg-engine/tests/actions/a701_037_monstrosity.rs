//! CR 701.37: monstrosity.

use crate::a701_028_071_common::*;
use mtg_engine::ability::*;
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::kwa::monstrosity::MONSTROUS;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn monstrous(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).monstrous
}

#[test]
fn monstrosity_puts_counters_and_the_permanent_becomes_monstrous_only_once() {
    cr!("701.37a");
    ruling!(
        "Nessian Asp",
        "Once a creature becomes monstrous, it can't become monstrous again. If the creature is already monstrous when the monstrosity ability resolves, nothing happens."
    );
    supported("Nessian Asp");
    let mut t = TestGame::new(2);
    let asp = t.battlefield(P0, "Nessian Asp");
    t.lands(P0, "Forest", 14);
    t.activate(P0, asp, 0, &[]).unwrap();
    // A second activation in response.
    t.activate(P0, asp, 0, &[]).unwrap();
    t.resolve();
    assert!(monstrous(&t, asp));
    assert_eq!(t.counters(asp, "+1/+1"), 4);
    assert_eq!(t.pt(asp), (8, 9));
    t.resolve();
    // Already monstrous: nothing happens.
    assert_eq!(t.counters(asp, "+1/+1"), 4);
    assert_eq!(custom_events(&t, MONSTROUS).len(), 1);
}

#[test]
fn monstrous_is_a_designation_not_an_ability_or_copiable_value() {
    cr!("701.37b");
    ruling!(
        "Vitality Hunter",
        "loses its +1/+1 counters, it will continue to be monstrous."
    );
    supported("Fleecemane Lion");
    let mut t = TestGame::new(2);
    let lion = t.battlefield(P0, "Fleecemane Lion");
    t.lands(P0, "Forest", 3);
    t.lands(P0, "Plains", 2);
    assert!(!t.obj(lion).has_keyword(KeywordKind::Hexproof));
    t.activate(P0, lion, 0, &[]).unwrap();
    t.resolve();
    assert!(monstrous(&t, lion));
    // "As long as this creature is monstrous, it has hexproof and indestructible."
    assert!(t.obj(lion).has_keyword(KeywordKind::Hexproof));
    assert!(t.obj(lion).has_keyword(KeywordKind::Indestructible));
    // Losing its +1/+1 counters and its abilities doesn't change that it's monstrous.
    t.g.remove_counters(Entity::Object(lion), "+1/+1", 1);
    run(
        &mut t,
        P1,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![
                Modification::RemoveAllAbilities,
                Modification::RemoveTypes(vec![CardType::Creature]),
                Modification::AddTypes(vec![CardType::Artifact]),
            ],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(lion)],
    );
    assert!(monstrous(&t, lion));
    assert!(t
        .g
        .matches(lion, &Filter::Custom(MONSTROUS.into()), &Default::default()));
    // A copy of it isn't monstrous.
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
        &[Entity::Object(lion)],
    )
    .var_objects(vars::CREATED)[0];
    assert!(!monstrous(&t, copy));
    // It stays monstrous until it leaves the battlefield: the new object isn't.
    let back = t
        .g
        .move_object(lion, Zone::Exile, MoveCause::Effect, None)
        .unwrap();
    let back = t
        .g
        .move_object(back, Zone::Battlefield, MoveCause::Effect, Some(P0))
        .unwrap();
    t.g.recompute();
    assert!(!monstrous(&t, back));
    assert!(!t.obj(back).has_keyword(KeywordKind::Hexproof));
}

#[test]
fn becomes_monstrous_triggers_see_the_x_it_became_monstrous_with() {
    cr!("701.37c");
    supported("Vitality Hunter");
    let mut t = TestGame::new(2);
    let hunter = t.battlefield(P0, "Vitality Hunter");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let c = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 4);
    // {X}{W}{W} with X = 2.
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.activate(P0, hunter, 0, &[]).unwrap();
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.resolve();
    assert_eq!(t.counters(hunter, "+1/+1"), 2);
    t.settle();
    // "Put a lifelink counter on each of up to X target creatures": X is 2.
    let maxes: Vec<u32> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { max, .. } => Some(max),
            _ => None,
        })
        .collect();
    assert_eq!(maxes, vec![2]);
    let s = t.g.stack.last().copied().expect("the trigger");
    let chosen = t.obj(s).stack.as_ref().unwrap().chosen[0].targets[0].clone();
    assert_eq!(chosen.len(), 2, "{chosen:?}");
    t.resolve();
    assert_eq!(t.counters(a, "lifelink"), 1);
    assert_eq!(t.counters(b, "lifelink"), 1);
    assert_eq!(t.counters(c, "lifelink"), 0);
}

#[test]
fn a_creature_that_left_doesnt_become_monstrous() {
    cr!("701.37a", "701.37b");
    ruling!(
        "Fleecemane Lion",
        "An ability that triggers when a creature becomes monstrous won't trigger if that creature isn't on the battlefield when its monstrosity ability resolves."
    );
    supported("Hythonia the Cruel");
    let mut t = TestGame::new(2);
    let hythonia = t.battlefield(P0, "Hythonia the Cruel");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 8);
    t.activate(P0, hythonia, 0, &[]).unwrap();
    t.g.move_object(hythonia, Zone::Hand(P0), MoveCause::Effect, None);
    t.resolve_all();
    assert!(custom_events(&t, MONSTROUS).is_empty());
    assert!(t.on_battlefield(bears));
    // On the battlefield, it becomes monstrous and "destroy all non-Gorgon creatures".
    let mut t = TestGame::new(2);
    let hythonia = t.battlefield(P0, "Hythonia the Cruel");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 8);
    t.activate(P0, hythonia, 0, &[]).unwrap();
    t.resolve_all();
    assert!(monstrous(&t, hythonia));
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(hythonia));
}
