//! Rulings batch S30 — Circles of Protection (CR 609.7, 615): "{1}: The next time a red
//! source of your choice would deal damage to you this turn, prevent that damage." The
//! source is chosen as the ability resolves, whether or not any damage is about to be
//! dealt, and the shield prevents only the next damage that source deals.

use crate::r_s01_common::{attack_with, block_and_finish, supported};
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 activates the Circle, choosing `source` as it resolves; returns the sources P0 was
/// offered.
fn circle_choosing(t: &mut TestGame, circle: ObjectId, source: ObjectId) -> Vec<Entity> {
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Object(source)]);
    t.activate(P0, circle, 0, &[]).unwrap();
    t.resolve();
    t.asked()[from..]
        .iter()
        .find_map(|(p, d)| match d {
            Decision::ChooseEntities {
                prompt, candidates, ..
            } if *p == P0 && prompt.contains("source") => Some(candidates.clone()),
            _ => None,
        })
        .expect("no source choice")
}

#[test]
fn a_circle_can_be_used_with_no_damage_pending_and_prevents_only_the_next_damage() {
    cr!("609.7a", "615.8");
    ruling!(
        "Circle of Protection: Red",
        "Can be used even when there is no damage to prevent. It prevents the next damage (if any) from the source this turn."
    );
    supported("Circle of Protection: Red");
    let mut t = TestGame::new(2);
    let circle = t.battlefield(P0, "Circle of Protection: Red");
    t.lands(P0, "Wastes", 1);
    // Boros Swiftblade: a red and white 1/2 with double strike.
    let blade = t.battlefield(P1, "Boros Swiftblade");
    // In P1's first main phase, with nothing on the stack, P0 chooses the Swiftblade.
    t.set_step(P1, Step::PrecombatMain);
    assert!(t.g.stack.is_empty());
    circle_choosing(&mut t, circle, blade);
    assert_eq!(t.life(P0), 20);
    // It attacks: its first-strike damage is the next damage it deals to P0 this turn and
    // is prevented; its regular combat damage isn't.
    attack_with(&mut t, &[(blade, Entity::Player(P0))]);
    block_and_finish(&mut t, P0, &[]);
    assert_eq!(t.life(P0), 19);
}

#[test]
fn a_source_can_be_a_spell_or_an_object_an_ability_on_the_stack_refers_to() {
    cr!("609.7a", "615.1a");
    ruling!(
        "Circle of Protection: Red",
        "A source of damage is a permanent, a spell on the stack (including one that creates a permanent), or any object referred to by an object on the stack. A source doesn’t need to be capable of dealing damage to be a legal choice."
    );
    supported("Circle of Protection: Red");
    // A spell on the stack (Lightning Bolt); Fervor, a red enchantment that can't deal
    // damage, can be chosen too.
    let mut t = TestGame::new(2);
    let circle = t.battlefield(P0, "Circle of Protection: Red");
    let fervor = t.battlefield(P1, "Fervor");
    t.lands(P0, "Wastes", 1);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(P0).go();
    let offered = circle_choosing(&mut t, circle, spell);
    assert!(offered.contains(&Entity::Object(spell)));
    assert!(offered.contains(&Entity::Object(fervor)));
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // A permanent spell (Ball Lightning): the creature it becomes deals no combat damage to
    // P0 the first time.
    let mut t = TestGame::new(2);
    let circle = t.battlefield(P0, "Circle of Protection: Red");
    t.lands(P0, "Wastes", 1);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 3);
    let ball = t.hand(P1, "Ball Lightning");
    let spell = t.cast(P1, ball).go();
    circle_choosing(&mut t, circle, spell);
    t.resolve_all();
    let ball = t.g.current(ball);
    assert!(t.on_battlefield(ball));
    attack_with(&mut t, &[(ball, Entity::Player(P0))]);
    block_and_finish(&mut t, P0, &[]);
    assert_eq!(t.life(P0), 20);
    // An object an ability on the stack refers to: Mogg Fanatic, sacrificed to pay for
    // "It deals 1 damage to any target", is in the graveyard.
    let mut t = TestGame::new(2);
    let circle = t.battlefield(P0, "Circle of Protection: Red");
    t.lands(P0, "Wastes", 1);
    let fanatic = t.battlefield(P1, "Mogg Fanatic");
    t.activate(P1, fanatic, 0, &[Entity::Player(P0)]).unwrap();
    assert!(t.in_graveyard(P1, "Mogg Fanatic"));
    let offered = circle_choosing(&mut t, circle, fanatic);
    assert!(offered.contains(&Entity::Object(fanatic)));
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}
