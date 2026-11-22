//! Rulings batch S12 — overload (CR 702.96): an alternative cost; if it's paid, "target"
//! in the spell's text becomes "each".

use crate::r_s01_common::*;
use crate::r_s02_common::can_cast;
use crate::r_s04_common::untapped_lands;
use crate::r_s12_common::attack_target;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const OVERLOAD: CastMethod = CastMethod::Keyword(KeywordKind::Overload);

/// Whether `id` has protection from each color: from a card of each color P1 holds.
fn protected_from_all_colors(t: &mut TestGame, id: ObjectId) -> bool {
    let id = t.g.current(id);
    ["Savannah Lions", "Counterspell", "Doom Blade", "Lightning Bolt", "Giant Growth"]
        .iter()
        .map(|n| t.hand(P1, n))
        .collect::<Vec<_>>()
        .into_iter()
        .all(|src| t.g.protected_from(id, src))
}

#[test]
fn cost_increases_and_reductions_apply_to_the_overload_cost_too() {
    cr!("702.96a", "601.2f");
    ruling!(
        "Eldritch Immunity",
        "Effects that cause you to pay more or less for a spell will cause you to pay that much more or less while casting it for its overload cost, too."
    );
    supported("Eldritch Immunity");
    supported("Thalia, Guardian of Thraben");
    // Wizards of Thay: "Instant and sorcery spells you cast cost {1} less to cast."
    // Overload {4}{C} costs {3}{C}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wizards of Thay");
    t.lands(P0, "Wastes", 4);
    let ei = t.hand(P0, "Eldritch Immunity");
    assert!(can_cast(&mut t, P0, ei, OVERLOAD));
    t.cast(P0, ei).method(OVERLOAD).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    // Thalia, Guardian of Thraben: "Noncreature spells cost {1} more to cast." Overload
    // {4}{C} costs {5}{C}.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Wastes", 5);
    let ei = t.hand(P0, "Eldritch Immunity");
    assert!(!can_cast(&mut t, P0, ei, OVERLOAD));
    t.lands(P0, "Wastes", 1);
    assert!(can_cast(&mut t, P0, ei, OVERLOAD));
    t.cast(P0, ei).method(OVERLOAD).go();
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn without_the_overload_cost_it_has_one_target_with_it_none() {
    cr!("702.96a", "702.96b");
    ruling!(
        "Eldritch Immunity",
        "If you don't pay the overload cost of a spell, that spell will have a single target. If you pay the overload cost, the spell won't have any targets."
    );
    ruling!(
        "Corporeal Projection",
        "If you don't pay the overload cost of a spell, that spell will have a single target. If you pay the overload cost, the spell won't have any targets."
    );
    // Eldritch Immunity: "Target creature you control gains protection from each color
    // until end of turn."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Wastes", 1);
    let ei = t.hand(P0, "Eldritch Immunity");
    t.cast(P0, ei).target(a).go();
    t.resolve_all();
    assert!(protected_from_all_colors(&mut t, a));
    assert!(!protected_from_all_colors(&mut t, b));
    // Overloaded: each creature you control, with no target chosen.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 5);
    let ei = t.hand(P0, "Eldritch Immunity");
    let from = t.asked().len();
    t.cast(P0, ei).method(OVERLOAD).go();
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
    t.resolve_all();
    assert!(protected_from_all_colors(&mut t, a) && protected_from_all_colors(&mut t, b));
    assert!(!protected_from_all_colors(&mut t, theirs));
    // Corporeal Projection ("Target creature you control gains myriad until end of
    // turn.") overloaded: each creature you control gains myriad; no target chosen.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 3);
    let cp = t.hand(P0, "Corporeal Projection");
    let from = t.asked().len();
    t.cast(P0, cp).method(OVERLOAD).go();
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
    t.resolve_all();
    assert!(t.obj(a).has_keyword(KeywordKind::Myriad));
    assert!(t.obj(b).has_keyword(KeywordKind::Myriad));
}

#[test]
fn the_mana_value_comes_from_the_mana_cost_whatever_was_paid() {
    cr!("702.96a", "601.2f", "202.3");
    ruling!(
        "Corporeal Projection",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying (such as an overload cost), add any cost increases, then apply any cost reductions. The mana value of the spell is determined by only its mana cost, no matter what the total cost to cast that spell was."
    );
    supported("Corporeal Projection");
    // Overload {3}{U}{U}{R}{R}, plus {1} for Thalia, minus {1} for Wizards of Thay.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.battlefield(P0, "Wizards of Thay");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 2);
    let cp = t.hand(P0, "Corporeal Projection");
    assert!(!can_cast(&mut t, P0, cp, OVERLOAD));
    t.lands(P0, "Wastes", 1);
    let spell = t.cast(P0, cp).method(OVERLOAD).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    // Its mana value is that of {U}{R}.
    assert_eq!(t.g.mana_value_of(spell), 2);
}

#[test]
fn overload_doesnt_change_when_the_spell_can_be_cast() {
    cr!("702.96a", "117.1a", "307.1");
    ruling!(
        "Eldritch Immunity",
        "Overload doesn't change when you can cast the spell."
    );
    ruling!(
        "Corporeal Projection",
        "Overload doesn't change when you can cast the spell."
    );
    // Eldritch Immunity, an instant, overloaded during the opponent's combat.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let attacker = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Wastes", 5);
    let ei = t.hand(P0, "Eldritch Immunity");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(attacker, Entity::Player(P0))]);
    assert_eq!(attack_target(&t, attacker), Some(Entity::Player(P0)));
    assert!(can_cast(&mut t, P0, ei, OVERLOAD));
    t.g.turn.priority = Some(P0);
    t.cast(P0, ei).method(OVERLOAD).go();
    t.resolve_all();
    assert!(protected_from_all_colors(&mut t, bears));
    // Corporeal Projection, a sorcery, overloaded or not: only in its controller's main
    // phase with an empty stack.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    t.lands(P0, "Mountain", 4);
    let cp = t.hand(P0, "Corporeal Projection");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_cast(&mut t, P0, cp, OVERLOAD));
    assert!(!can_cast(&mut t, P0, cp, CastMethod::Normal));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, cp, OVERLOAD));
    t.set_step(P0, Step::PostcombatMain);
    assert!(can_cast(&mut t, P0, cp, OVERLOAD));
}
