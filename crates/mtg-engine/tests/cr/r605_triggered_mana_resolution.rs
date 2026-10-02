//! CR 605.4a: a triggered mana ability resolves immediately after the mana ability that
//! triggered it — and it counts as having resolved this turn, like an ability that
//! resolves from the stack.

use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::triggers::turn_keys;
use mtg_engine::*;

#[test]
fn a_triggered_mana_ability_resolves_immediately_and_is_counted() {
    cr!("605.4a");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let aura = t.battlefield(P0, "Wild Growth");
    assert!(t.g.attach(aura, Entity::Object(forest)));
    t.g.recompute();
    // "Whenever enchanted land is tapped for mana, its controller adds an additional {G}."
    let uid = t.g.obj(aura).chars.abilities[1].uid;
    t.activate(P0, forest, 0, &[]).unwrap();
    assert_eq!(t.g.players[0].mana_pool.count(ManaType::G), 2);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(
        t.g.obj(aura)
            .triggers_this_turn
            .get(&(uid | turn_keys::RESOLVED)),
        Some(&1)
    );
}
