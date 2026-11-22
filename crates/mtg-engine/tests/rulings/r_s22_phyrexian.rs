//! Rulings batch S22 — Phyrexian mana (CR 107.4f): how each Phyrexian symbol of a cost
//! will be paid (2 life or mana) is announced as the spell or ability is proposed, with
//! its modes and the value of X, before its targets are chosen (CR 601.2b, 602.2b), and
//! it's paid that way (CR 601.2g–h).

use crate::r_s01_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::*;

/// The kinds of the decisions P0 was asked since `from`, in order: "pay" for how to pay
/// a Phyrexian symbol, "targets" for a target choice.
fn order_of(t: &TestGame, from: usize) -> Vec<&'static str> {
    t.asked()[from..]
        .iter()
        .filter(|(p, _)| *p == P0)
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { prompt, .. } if prompt.starts_with("How will you pay") => {
                Some("pay")
            }
            Decision::ChooseTargets { .. } => Some("targets"),
            _ => None,
        })
        .collect()
}

#[test]
fn how_to_pay_a_phyrexian_symbol_is_chosen_before_targets() {
    cr!("601.2b", "601.2c", "602.2b", "107.4f", "118.13a");
    ruling!(
        "Gut Shot",
        "As you cast a spell or activate an activated ability with one or more Phyrexian mana symbols in its cost, you choose how to pay for each Phyrexian mana symbol at the same time you would choose modes or choose a value for X."
    );
    supported("Gut Shot");
    supported("Spellskite");
    // Gut Shot {R/P}: "Gut Shot deals 1 damage to any target." P0 announces paying 2 life
    // (options: either way, {R}, 2 life), then chooses the target; the Mountain stays
    // untapped.
    let mut t = TestGame::new(2);
    let mountain = t.lands(P0, "Mountain", 1)[0];
    let shot = t.hand(P0, "Gut Shot");
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    let from = t.asked().len();
    t.cast(P0, shot).target(Entity::Player(P1)).go();
    assert_eq!(order_of(&t, from), vec!["pay", "targets"]);
    assert_eq!(t.life(P0), 18);
    assert!(!t.obj(mountain).tapped);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // Announcing {R}: the Mountain pays it.
    let mut t = TestGame::new(2);
    let mountain = t.lands(P0, "Mountain", 1)[0];
    let shot = t.hand(P0, "Gut Shot");
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, shot).target(Entity::Player(P1)).go();
    assert_eq!(t.life(P0), 20);
    assert!(t.obj(mountain).tapped);

    // An activated ability: Spellskite's "{U/P}: Change the target of target spell or
    // ability to Spellskite." P0 announces 2 life, then targets P1's Shock.
    let mut t = TestGame::new(2);
    let skite = t.battlefield(P0, "Spellskite");
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    let spell = t.cast(P1, shock).target(Entity::Player(P0)).go();
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    let from = t.asked().len();
    t.activate(P0, skite, 0, &[Entity::Object(spell)]).unwrap();
    assert_eq!(order_of(&t, from), vec!["pay", "targets"]);
    assert_eq!(t.life(P0), 18);
    t.resolve_all();
    // Shock was redirected to Spellskite (0/4).
    assert_eq!(t.life(P0), 18);
    assert_eq!(crate::r_s07_common::damage_on(&t, skite), 2);
}
