//! Rulings batch S06 — Auras of Amonkhet that care about Deserts: "When this Aura enters,
//! if you control a Desert or there is a Desert card in your graveyard, [effect]."

use crate::r_s01_common::*;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s04_common::*;
use crate::r_s05_common::move_to;
use crate::r_s06_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Casts Desert's Hold ("When this Aura enters, if you control a Desert or there is a
/// Desert card in your graveyard, you gain 3 life.") on an opponent's creature and puts
/// its enters trigger (if any) on the stack.
fn cast_deserts_hold(t: &mut TestGame) {
    let target = t.battlefield(P1, "Grizzly Bears");
    let hold = in_hand_with_mana(t, P0, "Desert's Hold");
    t.cast(P0, hold).target(target).go();
    t.resolve();
}

#[test]
fn one_desert_is_as_good_as_five() {
    cr!("603.4", "205.3i");
    ruling!(
        "Desert's Hold",
        "If an ability checks whether you control a Desert or there is a Desert card in your graveyard, having more than one doesn't matter. Controlling one is the same as controlling five. There is also no extra bonus for both controlling one and having one in your graveyard."
    );
    supported("Desert's Hold");
    supported("Desert of the True");
    // No Desert: no trigger.
    let mut t = TestGame::new(2);
    cast_deserts_hold(&mut t);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P0), 20);
    // Several Deserts on the battlefield and one in the graveyard: 3 life, once.
    let mut t = TestGame::new(2);
    t.lands(P0, "Desert of the True", 3);
    t.graveyard(P0, "Desert of the True");
    cast_deserts_hold(&mut t);
    assert_eq!(on_stack(&t, "gain 3 life"), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    // Only a Desert card in the graveyard.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Desert of the True");
    cast_deserts_hold(&mut t);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
}

#[test]
fn the_desert_condition_is_checked_on_trigger_and_on_resolution() {
    cr!("603.4");
    ruling!(
        "Desert's Hold",
        "For abilities that trigger only if you control a Desert or there is a Desert card in your graveyard, one condition must be true as the ability triggers and one must be true as the ability resolves. They don't have to be the same condition, though. For example, you could sacrifice your only Desert after the ability triggers but before it has resolved."
    );
    // The only Desert goes from the battlefield to the graveyard in response: still true.
    let mut t = TestGame::new(2);
    let desert = t.battlefield(P0, "Desert of the True");
    cast_deserts_hold(&mut t);
    assert_eq!(on_stack(&t, "gain 3 life"), 1);
    move_to(&mut t, desert, Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    // It's exiled instead: the condition is false as the ability resolves.
    let mut t = TestGame::new(2);
    let desert = t.battlefield(P0, "Desert of the True");
    cast_deserts_hold(&mut t);
    assert_eq!(on_stack(&t, "gain 3 life"), 1);
    move_to(&mut t, desert, Zone::Exile);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn unquenchable_thirst_needs_only_one_desert() {
    cr!("603.4");
    ruling!(
        "Unquenchable Thirst",
        "For abilities that trigger only if you control a Desert or there is a Desert card in your graveyard, one condition must be true as the ability triggers and one must be true as the ability resolves."
    );
    supported("Unquenchable Thirst");
    // "When this Aura enters, if you control a Desert or there is a Desert card in your
    // graveyard, tap enchanted creature."
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Desert of the True");
    let target = t.battlefield(P1, "Grizzly Bears");
    let aura = in_hand_with_mana(&mut t, P0, "Unquenchable Thirst");
    t.cast(P0, aura).target(target).go();
    t.resolve();
    t.resolve_all();
    assert!(t.obj_now(target).tapped);
    // Without a Desert, it isn't tapped.
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Grizzly Bears");
    let aura = in_hand_with_mana(&mut t, P0, "Unquenchable Thirst");
    t.cast(P0, aura).target(target).go();
    t.resolve();
    t.resolve_all();
    assert!(!t.obj_now(target).tapped);
}

#[test]
fn wall_of_forgotten_pharaohs_needs_a_desert_to_activate() {
    cr!("602.5");
    ruling!(
        "Wall of Forgotten Pharaohs",
        "If an ability checks whether you control a Desert or there is a Desert card in your graveyard, having more than one doesn't matter."
    );
    supported("Wall of Forgotten Pharaohs");
    // "{T}: This creature deals 1 damage to target player or planeswalker. Activate only if
    // you control a Desert or there is a Desert card in your graveyard."
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Forgotten Pharaohs");
    assert!(activate_containing(&mut t, P0, wall, "damage").is_err());
    t.lands(P0, "Desert of the True", 2);
    t.graveyard(P0, "Desert of the True");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, wall, "damage").expect("with Deserts");
    t.resolve();
    assert_eq!(t.life(P1), 19);
}
