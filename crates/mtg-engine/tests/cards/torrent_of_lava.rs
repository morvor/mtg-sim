//! Torrent of Lava: while it's on the stack each creature has "{T}: Prevent the next 1
//! damage that would be dealt to this creature by Torrent of Lava this turn." (a compiled
//! on-stack static, CR 113.6, `oracle/patterns/on_stack_static.rs`; a prevention shield,
//! CR 615.7).

use mtg_engine::testing::*;
use mtg_engine::*;

fn activated(t: &TestGame, id: ObjectId) -> usize {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, ability::AbilityKind::Activated(_)))
        .count()
}

#[test]
fn the_granted_ability_prevents_one_damage_from_the_torrent() {
    cr!("113.6", "615.7");
    ruling!("Torrent of Lava", "This allows the ability to be activated and resolved before the Torrent itself resolves");
    let mut t = TestGame::new(2);
    let shielded = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Grizzly Bears");
    // No ability before the Torrent is on the stack.
    assert_eq!(activated(&t, shielded), 0);
    t.lands(P0, "Mountain", 4);
    let torrent = t.hand(P0, "Torrent of Lava");
    t.cast(P0, torrent).x(2).go();
    assert_eq!(activated(&t, shielded), 1);
    t.g.turn.priority = Some(P1);
    t.activate(P1, shielded, 0, &[]).unwrap();
    assert!(t.obj_now(shielded).tapped);
    // The ability resolves first, then the Torrent deals 2 damage to each creature.
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.on_battlefield(shielded));
    assert_eq!(t.obj_now(shielded).damage, 1);
    assert!(!t.on_battlefield(other));
}

#[test]
fn the_shield_only_stops_damage_from_the_torrent() {
    cr!("615.7");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let torrent = t.hand(P0, "Torrent of Lava");
    t.cast(P0, torrent).x(0).go();
    t.g.turn.priority = Some(P1);
    t.activate(P1, bears, 0, &[]).unwrap();
    t.resolve();
    // A Shock in response still deals its 2 damage.
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(bears).go();
    t.resolve();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn the_ability_is_gone_once_the_torrent_has_resolved() {
    cr!("113.6");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 3);
    let torrent = t.hand(P0, "Torrent of Lava");
    t.cast(P0, torrent).x(1).go();
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 1);
    assert_eq!(activated(&t, giant), 0);
}
