//! "Damage that would reduce your life total to less than 1 reduces it to 1 instead" (Ali
//! from Cairo, Angel's Grace; `src/oracle/patterns/replacements_life.rs`) is a replacement
//! effect on the life loss damage dealt to a player causes (CR 614.1a, 120.3a), and "your
//! life total" is its controller's: damage dealt to another player reduces that player's
//! life total as usual. Both forms are tested: the static ability and the one-shot "Until
//! end of turn, ...".

use crate::r614_audit_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

fn set_life(t: &mut TestGame, p: PlayerId, life: i32) {
    t.g.players[p.idx()].life = life;
}

/// P0 and P1 at 2 life; P0 casts Lightning Bolt at P0, then at P1.
fn bolt_both(t: &mut TestGame) {
    set_life(t, P0, 2);
    set_life(t, P1, 2);
    cast_resolve(t, P0, "Lightning Bolt", &[Entity::Player(P0)]);
    assert_eq!(t.life(P0), 1, "P0's own life total stops at 1");
    cast_resolve(t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    assert_eq!(t.life(P1), -1, "P1's life total isn't P0's");
    assert!(t.has_lost(P1));
}

#[test]
fn angels_grace_keeps_only_its_controllers_life_total_at_1() {
    cr!("614.1a", "120.3a");
    ruling!(
        "Angel's Grace",
        "It only changes the result of damage dealt to you."
    );
    let mut t = TestGame::new(2);
    cast_resolve(&mut t, P0, "Angel's Grace", &[]);
    bolt_both(&mut t);
}

#[test]
fn ali_from_cairo_keeps_only_its_controllers_life_total_at_1() {
    cr!("614.1a", "120.3a");
    supported("Ali from Cairo");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ali from Cairo");
    bolt_both(&mut t);
}
