//! CR 602.2b with 601.2f, 118.7 and 118.9: an activated ability's total cost — cost
//! increases before reductions, colored reductions, reductions that keep one mana, and
//! alternative activation costs.

use super::r600_common::*;
use mtg_engine::ability::*;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::Color;
use mtg_engine::*;

/// A permanent with "{cost}: You gain 1 life."
fn pinger(cost: &str) -> CardDef {
    CB::new("Test Pinger")
        .artifact()
        .cost("{1}")
        .ability(act(mana_cost(cost), Body::effect(gain(1))))
        .build()
}

/// A permanent with a static cost change for every activated ability.
fn changer(change: CostChange) -> CardDef {
    CB::new("Test Changer")
        .enchantment()
        .cost("{1}")
        .ability(stat(StaticEffect::CostModifier(CostModifier {
            applies_to: CostTarget::ActivatedAbilities(Box::new(AbilityScope::new(
                Filter::Any,
                AbilityClass::Any,
            ))),
            who: PlayerRel::Any,
            change,
        })))
        .build()
}

fn total(t: &mut TestGame, src: ObjectId) -> String {
    t.g.recompute();
    let a = t
        .g
        .obj(src)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .cloned()
        .unwrap();
    let AbilityKind::Activated(act) = &a.kind else {
        unreachable!()
    };
    let c = t.g.ability_total_cost(P0, src, &a, act);
    c.mana.map(|m| m.to_string()).unwrap_or_default()
}

#[test]
fn reductions_apply_after_increases_whatever_the_order_of_their_sources() {
    cr!("601.2f", "602.2b");
    let mut t = TestGame::new(2);
    // The reduction comes first in timestamp order.
    t.custom(P0, changer(CostChange::ReduceGeneric(Value::c(2))), Zone::Battlefield);
    t.custom(P0, changer(CostChange::IncreaseGeneric(Value::c(2))), Zone::Battlefield);
    let p = t.custom(P0, pinger("{R}"), Zone::Battlefield);
    assert_eq!(total(&mut t, p), "{R}");
}

#[test]
fn a_colored_reduction_reduces_that_color_then_generic_mana() {
    cr!("118.7b", "118.7c", "602.2b");
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        changer(CostChange::ReduceColored(Color::Red, Value::c(2))),
        Zone::Battlefield,
    );
    let p = t.custom(P0, pinger("{2}{R}"), Zone::Battlefield);
    assert_eq!(total(&mut t, p), "{1}");
    let q = t.custom(P0, pinger("{U}{U}"), Zone::Battlefield);
    assert_eq!(total(&mut t, q), "{U}{U}");
}

#[test]
fn a_reduction_that_keeps_one_mana_never_adds_mana() {
    cr!("601.2f", "118.7a");
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        changer(CostChange::ReduceGenericMinOne(Value::c(3))),
        Zone::Battlefield,
    );
    for (cost, want) in [("{4}", "{1}"), ("{1}", "{1}"), ("{2}{G}", "{G}"), ("{G}{G}", "{G}{G}")] {
        let p = t.custom(P0, pinger(cost), Zone::Battlefield);
        assert_eq!(total(&mut t, p), want, "{cost}");
    }
    let tap = t.custom(
        P0,
        CB::new("Tapper")
            .artifact()
            .ability(act(Cost::default().with(CostPart::Tap), Body::effect(gain(1))))
            .build(),
        Zone::Battlefield,
    );
    assert_eq!(total(&mut t, tap), "");
}
