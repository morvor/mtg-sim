//! CR 701.69: heal.

use crate::a701_028_071_common::*;
use mtg_engine::ability::{Effect, KeywordAction, Sel, Value};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn damage(t: &mut TestGame, to: ObjectId, n: i32) {
    let src = t.battlefield(P1, "Prodigal Pyromancer");
    run(
        t,
        P1,
        Some(src),
        Effect::DealDamage {
            source: Sel::This,
            amount: Value::c(n),
            to: Sel::Target(0),
        },
        &[Entity::Object(to)],
    );
}

#[test]
fn healing_removes_marked_damage() {
    cr!("701.69a");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.g.objects[giant.0 as usize].damage = 2;
    // Heal 1 damage, then all of it.
    run(
        &mut t,
        P0,
        None,
        ka(KeywordAction::Heal, Sel::Target(0), 1),
        &[Entity::Object(giant)],
    );
    assert_eq!(t.obj_now(giant).damage, 1);
    run(
        &mut t,
        P0,
        None,
        ka(KeywordAction::Heal, Sel::Target(0), -1),
        &[Entity::Object(giant)],
    );
    assert_eq!(t.obj_now(giant).damage, 0);
}

#[test]
fn all_other_damage_already_dealt_is_healed() {
    cr!("701.69a");
    // Wolverine, Fierce Fighter (3/5): "If damage would be dealt to Wolverine, instead that
    // damage is dealt, but all other damage already dealt to him is healed."
    let mut t = TestGame::new(2);
    let wolverine = t.battlefield(P0, "Wolverine, Fierce Fighter");
    damage(&mut t, wolverine, 3);
    t.settle();
    assert_eq!(t.obj_now(wolverine).damage, 3);
    // Three more damage would be lethal; the other three are healed.
    damage(&mut t, wolverine, 3);
    t.settle();
    assert!(t.on_battlefield(wolverine));
    assert_eq!(t.obj_now(wolverine).damage, 3);
    damage(&mut t, wolverine, 1);
    t.settle();
    assert_eq!(t.obj_now(wolverine).damage, 1);
    // Lethal damage at once still destroys him.
    damage(&mut t, wolverine, 5);
    t.settle();
    assert!(!t.on_battlefield(wolverine));
}
