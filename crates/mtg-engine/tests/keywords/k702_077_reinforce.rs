//! CR 702.77 Reinforce.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_027_037::activate_named;
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

const REINFORCE: &str = "Reinforce";

#[test]
fn reinforce_discards_the_card_to_put_counters_on_target_creature() {
    cr!("702.77", "702.77a");
    assert_supported("Burrenton Bombardier");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    let bombardier = t.hand(P0, "Burrenton Bombardier");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_named(&mut t, P0, bombardier, REINFORCE, 0).unwrap();
    // Discarding it is part of the cost.
    assert!(t.in_graveyard(P0, "Burrenton Bombardier"));
    t.resolve();
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn reinforce_can_be_activated_any_time_but_only_from_the_hand() {
    cr!("702.77a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 6);
    // At instant speed, during an opponent's turn.
    t.set_step(P1, Step::DeclareAttackers);
    let bombardier = t.hand(P0, "Burrenton Bombardier");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_named(&mut t, P0, bombardier, REINFORCE, 0).unwrap();
    t.resolve();
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    // Not from the battlefield.
    t.set_step(P0, Step::PrecombatMain);
    let on_bf = t.battlefield(P0, "Burrenton Bombardier");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    assert!(activate_named(&mut t, P0, on_bf, REINFORCE, 0).is_err());
    assert!(t.on_battlefield(on_bf));
}

#[test]
fn reinforce_x_puts_x_counters() {
    cr!("702.77a");
    assert_supported("Wren's Run Hydra");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 5);
    let hydra = t.hand(P0, "Wren's Run Hydra");
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_named(&mut t, P0, hydra, REINFORCE, 0).unwrap();
    t.resolve();
    assert_eq!(t.counters(bears, counters::PLUS1), 3);
}

#[test]
fn a_permanent_with_reinforce_has_an_activated_ability() {
    cr!("702.77b");
    assert_supported("Rustic Clachan");
    assert_supported("Tsabo's Web");
    let mut t = TestGame::new(2);
    // Rustic Clachan (a land with reinforce) on the battlefield has a nonmana activated
    // ability even though it can't be activated there.
    let clachan = t.battlefield(P0, "Rustic Clachan");
    let plains = t.battlefield(P0, "Plains");
    assert!(t.obj_now(clachan).chars.abilities.iter().any(|a| {
        matches!(&a.kind, AbilityKind::Activated(x) if !x.is_mana_ability) && a.text == REINFORCE
    }));
    // So Tsabo's Web ("Each land with an activated ability that isn't a mana ability
    // doesn't untap during its controller's untap step") affects it.
    t.battlefield(P1, "Tsabo's Web");
    t.g.objects[clachan.0 as usize].tapped = true;
    t.g.objects[plains.0 as usize].tapped = true;
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(clachan).tapped);
    assert!(!t.obj_now(plains).tapped);
}
