//! CR 508.1c, 508.5, 310.9b: "~ can't attack unless defending player controls an Island"
//! is checked against the player it would attack, and a creature attacking a battle
//! attacks that battle's protector: the restriction applies to attacking a battle as it
//! does to attacking the protector directly.

use mtg_engine::battle;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn sea_monster_cant_attack_a_battle_whose_protector_controls_no_island() {
    cr!("508.1c", "508.5", "310.9b");
    let mut t = TestGame::new(2);
    // P0's Invasion of Segovia (a Siege) is protected by P1.
    t.answer_choose(P0, &[Entity::Player(P1)]);
    let siege = t.enter(P0, "Invasion of Segovia");
    t.resolve_all();
    assert_eq!(battle::protector(&t.g, siege), Some(P1));
    let monster = t.battlefield(P0, "Sea Monster");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!t.g.can_attack_target(monster, Entity::Player(P1)));
    assert!(!t.g.can_attack_target(monster, Entity::Object(siege)));
    // Once P1 controls an Island, it can attack either.
    t.battlefield(P1, "Island");
    assert!(t.g.can_attack_target(monster, Entity::Player(P1)));
    assert!(t.g.can_attack_target(monster, Entity::Object(siege)));
}
