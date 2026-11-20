//! CR 615.7: a prevention shield ("Prevent the next N damage that would be dealt to [a
//! permanent or player] this turn") applies only to damage dealt to the shielded permanent
//! or player. Damage dealt at the same time to recipients the shield doesn't cover is no
//! part of it: its controller divides what's left of it only among its own recipients
//! (`prevention::divide_shields`, register entry `shield-divided-among-recipients`), and a
//! recipient's controller orders the damage two or more sources deal to it only when a
//! shield applies to that damage (`prevention::order_for_shields`). Nothing is asked about
//! damage to other recipients.

use crate::r614_audit_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0's Samite Healer ("{T}: Prevent the next 1 damage that would be dealt to any target
/// this turn.") puts its shield on `target`.
fn healer_shields(t: &mut TestGame, target: ObjectId) {
    supported("Samite Healer");
    let healer = t.battlefield(P0, "Samite Healer");
    t.activate(P0, healer, 0, &[obj(target)])
        .expect("Samite Healer's ability");
    t.resolve_all();
}

#[test]
fn pyroclasm_leaves_a_shield_on_one_creature_undivided() {
    cr!("615.7");
    supported("Pyroclasm");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    healer_shields(&mut t, bears);
    // Pyroclasm deals 2 damage to the Bears and 2 to Samite Healer at once. The shield
    // covers only the Bears' damage, so there's nothing to divide: 1 of it is prevented.
    // (Asked to divide the shield between the two creatures, this answer would put none of
    // it on the Bears, which would die.)
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![0, 1]));
    cast_resolve(&mut t, P0, "Pyroclasm", &[]);
    let divisions = t
        .asked()
        .iter()
        .filter(|(_, d)| matches!(d, Decision::Divide { .. }))
        .count();
    assert_eq!(divisions, 0);
    assert!(t.on_battlefield(bears));
    assert_eq!(damage_on(&t, bears), 1);
    assert!(t.in_graveyard(P0, "Samite Healer"));
}

#[test]
fn two_attackers_damage_to_a_player_isnt_ordered_for_a_shield_on_a_creature() {
    cr!("615.7", "510.2");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let other = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    healer_shields(&mut t, bears);
    // P1's Hill Giant and Grizzly Bears deal their combat damage to P0 at once. The shield
    // is on P0's Bears, not on P0: P0 isn't asked which of that damage it prevents.
    t.attack(
        &[(giant, Entity::Player(P0)), (other, Entity::Player(P0))],
        &[],
    );
    let orders = t
        .asked()
        .iter()
        .filter(|(_, d)| matches!(d, Decision::Order { prompt, .. } if prompt.contains("shield")))
        .count();
    assert_eq!(orders, 0);
    assert_eq!(t.life(P0), 15);
    assert_eq!(damage_on(&t, bears), 0);
}
