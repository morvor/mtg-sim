//! CR 606: loyalty abilities.

use super::r600_common::*;
use mtg_engine::ability::*;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// A planeswalker with "+1: You gain 1 life." and "−4: Draw a card."
fn walker(loyalty: i32) -> CardDef {
    CB::new("Test Walker")
        .planeswalker(loyalty)
        .cost("{3}")
        .ability(act_from({
            let mut a = ActivatedAbility::new(
                Cost::default().with(CostPart::Loyalty(1)),
                Body::effect(gain(1)),
            );
            a.is_loyalty = true;
            a
        }))
        .ability(act_from({
            let mut a = ActivatedAbility::new(
                Cost::default().with(CostPart::Loyalty(-4)),
                Body::effect(draw(1)),
            );
            a.is_loyalty = true;
            a
        }))
        .build()
}

fn loyalty(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, "loyalty")
}

fn put_walker(t: &mut TestGame, p: PlayerId, n: i32) -> ObjectId {
    let w = t.custom(p, walker(n), Zone::Battlefield);
    t.g.objects[w.0 as usize]
        .counters
        .insert("loyalty".into(), n as u32);
    t.g.recompute();
    w
}

#[test]
fn an_activated_ability_with_a_loyalty_symbol_in_its_cost_is_a_loyalty_ability() {
    cr!("606.1", "606.2");
    let ajani = card("Ajani Goldmane");
    let abs = abilities(&ajani);
    for a in abs
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
    {
        let AbilityKind::Activated(x) = &a.kind else {
            unreachable!()
        };
        assert!(x.is_loyalty, "{}", a.text);
        assert!(x.cost.loyalty().is_some());
    }
    // Abilities without a loyalty symbol aren't loyalty abilities, even on planeswalkers.
    let elves = card("Llanowar Elves");
    let AbilityKind::Activated(x) = &abilities(&elves)[0].kind else {
        panic!()
    };
    assert!(!x.is_loyalty);
}

#[test]
fn loyalty_abilities_have_sorcery_timing_and_one_activation_per_turn() {
    cr!("606.3");
    let mut t = TestGame::new(2);
    let w = put_walker(&mut t, P0, 5);
    // Not while the stack isn't empty.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    assert!(t.activate(P0, w, 0, &[]).is_err());
    t.resolve();
    // Main phase, own turn, empty stack: OK — once.
    t.activate(P0, w, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.life(P0), 21);
    // No other loyalty ability of that permanent this turn.
    assert!(t.activate(P0, w, 0, &[]).is_err());
    assert!(t.activate(P0, w, 1, &[]).is_err());
    // A different planeswalker's is fine.
    let w2 = put_walker(&mut t, P0, 5);
    t.activate(P0, w2, 0, &[]).unwrap();
    t.resolve();
    // Not during combat or on another player's turn.
    t.advance_to(P0, Step::BeginningOfCombat);
    let w3 = put_walker(&mut t, P0, 5);
    assert!(t.activate(P0, w3, 0, &[]).is_err());
    t.advance_to(P1, Step::PrecombatMain);
    t.g.turn.priority = Some(P0);
    assert!(t.activate(P0, w3, 0, &[]).is_err());
    // Its next turn, it can be activated again.
    t.advance_to(P0, Step::PrecombatMain);
    t.activate(P0, w, 0, &[]).unwrap();
}

#[test]
fn the_cost_is_adding_or_removing_loyalty_counters_and_may_be_modified() {
    cr!("606.4");
    let mut t = TestGame::new(2);
    let w = put_walker(&mut t, P0, 5);
    t.activate(P0, w, 0, &[]).unwrap();
    assert_eq!(loyalty(&t, w), 6);
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    t.activate(P0, w, 1, &[]).unwrap();
    assert_eq!(loyalty(&t, w), 2);
    // Eidolon of Obstruction: "Loyalty abilities of planeswalkers your opponents control
    // cost {1} more to activate."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Eidolon of Obstruction");
    let w = put_walker(&mut t, P0, 5);
    assert!(t.activate(P0, w, 0, &[]).is_err());
    let m = t.battlefield(P0, "Mountain");
    t.activate(P0, w, 0, &[]).unwrap();
    assert!(t.obj(m).tapped);
    assert_eq!(loyalty(&t, w), 6);
    // Its controller's own planeswalkers aren't affected.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Eidolon of Obstruction");
    let w = put_walker(&mut t, P0, 5);
    t.activate(P0, w, 0, &[]).unwrap();
}

#[test]
fn multiple_loyalty_costs_combine_into_one() {
    cr!("606.5");
    // Carth the Lion: "Planeswalkers' loyalty abilities you activate cost an additional
    // [+1] to activate."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Carth the Lion");
    let w = put_walker(&mut t, P0, 3);
    let pw = t.obj(w).chars.abilities[0].clone();
    let AbilityKind::Activated(act) = &pw.kind else {
        panic!()
    };
    let total = t.g.ability_total_cost(P0, w, &pw, act);
    assert_eq!(
        total
            .parts
            .iter()
            .filter(|c| matches!(c, CostPart::Loyalty(_)))
            .count(),
        1
    );
    assert_eq!(total.loyalty(), Some(2));
    // [+1] puts two loyalty counters on it.
    t.activate(P0, w, 0, &[]).unwrap();
    assert_eq!(loyalty(&t, w), 5);
    t.resolve();
    // [−4] removes three.
    let w2 = put_walker(&mut t, P0, 3);
    t.activate(P0, w2, 1, &[]).unwrap();
    assert_eq!(loyalty(&t, w2), 0);
}

#[test]
fn a_negative_loyalty_cost_needs_that_many_counters() {
    cr!("606.6");
    let mut t = TestGame::new(2);
    let w = put_walker(&mut t, P0, 3);
    // −4 with three loyalty counters: can't be activated.
    assert!(t.activate(P0, w, 1, &[]).is_err());
    assert_eq!(loyalty(&t, w), 3);
    // Taking additional costs into account: with Carth the Lion it's −3, which it can pay.
    t.battlefield(P0, "Carth the Lion");
    t.activate(P0, w, 1, &[]).unwrap();
    assert_eq!(loyalty(&t, w), 0);
}
