//! Undergrowth (hand-written, `src/cards/undergrowth.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn fight(kicked: bool) -> TestGame {
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P1, "Gray Ogre");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Mountain", 3);
    let u = t.hand(P0, "Undergrowth");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(kicked));
    t.cast(P0, u).go();
    t.resolve();
    t.attack(
        &[(ogre, Entity::Player(P0)), (bear, Entity::Player(P0))],
        &[],
    );
    t
}

#[test]
fn prevents_all_combat_damage() {
    cr!("615.1a");
    let t = fight(false);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn red_creatures_deal_damage_if_its_additional_cost_was_paid() {
    cr!("615.1a", "601.2b");
    ruling!("Undergrowth", "red creatures still deal damage if you pay the additional cost");
    let t = fight(true);
    // Gray Ogre (red) deals its 2; Grizzly Bears (green) is still prevented.
    assert_eq!(t.life(P0), 18);
}
