//! Rulings batch S03 — cohort (an ability word): "Cohort — {T}, Tap an untapped Ally you
//! control: ..."

use crate::r_s01_common::*;
use crate::r_s02_common::can_activate;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn the_cohort_creature_cant_be_summoning_sick_but_the_other_ally_can() {
    cr!("302.6", "602.5a", "118.3", "207.2c");
    ruling!(
        "Ondu War Cleric",
        "To activate a cohort ability, the Ally with that ability must have been under your control continuously since the beginning of your most recent turn. Informally, it can’t have “summoning sickness.” However, the other Ally you tap can be one that just came under your control. (Note that tapping the second Ally doesn’t use {T} [the tap symbol].)"
    );
    supported("Ondu War Cleric");
    // Ondu War Cleric: "Cohort — {T}, Tap an untapped Ally you control: You gain 2 life."
    let mut t = TestGame::new(2);
    let old = t.battlefield(P0, "Ondu War Cleric");
    let new = t.battlefield_sick(P0, "Ondu War Cleric");
    // The one that just came under P0's control can't activate its cohort ability ...
    assert!(!can_activate(&mut t, P0, new));
    assert!(can_activate(&mut t, P0, old));
    // ... but it can be tapped to pay the cost of the other one's.
    t.answer_choose(P0, &[Entity::Object(new)]);
    t.activate(P0, old, 0, &[]).unwrap();
    assert!(t.obj(old).tapped && t.obj(new).tapped);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);

    // The Ally with the ability can't be the other Ally tapped for its cost.
    let mut t = TestGame::new(2);
    let alone = t.battlefield(P0, "Ondu War Cleric");
    t.battlefield(P0, "Grizzly Bears");
    assert!(!can_activate(&mut t, P0, alone));
    // Nor can a summoning-sick creature pay a {T} cost of its own when another Ally is
    // untapped.
    let mut t = TestGame::new(2);
    let sick = t.battlefield_sick(P0, "Ondu War Cleric");
    t.battlefield(P0, "Ondu War Cleric");
    assert!(!can_activate(&mut t, P0, sick));
}

#[test]
fn summoning_sick_creatures_can_be_tapped_for_a_cost_without_the_tap_symbol() {
    cr!("302.6", "118.3");
    ruling!(
        "Heritage Druid",
        "Since Heritage Druid’s activated ability doesn’t have a tap symbol in its cost, you can tap creatures that haven’t been under your control since your most recent turn began (including Heritage Druid itself) to pay the cost."
    );
    supported("Heritage Druid");
    // Heritage Druid: "Tap three untapped Elves you control: Add {G}{G}{G}." All three
    // Elves just came under P0's control, including the Druid itself.
    let mut t = TestGame::new(2);
    let druid = t.battlefield_sick(P0, "Heritage Druid");
    let a = t.battlefield_sick(P0, "Llanowar Elves");
    let b = t.battlefield_sick(P0, "Llanowar Elves");
    assert!(can_pay_druid(&mut t, druid));
    t.answer_choose(
        P0,
        &[Entity::Object(druid), Entity::Object(a), Entity::Object(b)],
    );
    t.activate(P0, druid, 0, &[]).unwrap();
    assert!([druid, a, b].iter().all(|x| t.obj(*x).tapped));
    assert_eq!(
        t.g.player(P0)
            .mana_pool
            .count(mtg_engine::mana::ManaType::G),
        3
    );
}

/// Whether the cost of the Druid's (only) activated ability could be paid now.
fn can_pay_druid(t: &mut TestGame, druid: ObjectId) -> bool {
    use mtg_engine::ability::AbilityKind;
    t.g.recompute();
    let cost = t
        .obj(druid)
        .chars
        .abilities
        .iter()
        .find_map(|a| match &a.kind {
            AbilityKind::Activated(x) => Some(x.cost.clone()),
            _ => None,
        })
        .unwrap();
    let ctx = mtg_engine::eval::Ctx::new(Some(druid), P0);
    t.g.can_pay_cost(P0, &cost, Some(druid), &ctx)
}
