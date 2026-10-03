//! CR 116.2m: paying the unlock cost of a locked half of a Room is a special action (with
//! the Room rules it relies on, CR 709.5).

use super::r114_common::*;
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::mana::ManaType;
use mtg_engine::object::*;
use mtg_engine::rooms;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn add_mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

fn unlock_actions(t: &mut TestGame, p: PlayerId, room: ObjectId) -> Vec<SpecialAction> {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .into_iter()
        .filter_map(|a| match a {
            Action::Special(s @ SpecialAction::Other { obj: Some(o), .. }) if o == room => Some(s),
            _ => None,
        })
        .collect()
}

fn unlock_door(half: usize, room: ObjectId) -> SpecialAction {
    SpecialAction::Other {
        name: format!("{}{half}", rooms::UNLOCK_ACTION),
        obj: Some(room),
    }
}

/// Casts Glassworks (the left half of Glassworks // Shattered Yard) and resolves it,
/// aiming its unlock trigger at `target`.
fn cast_glassworks(t: &mut TestGame, target: ObjectId) -> ObjectId {
    let card = t.hand(P0, "Glassworks // Shattered Yard");
    add_mana(t, P0, ManaType::R, 1);
    add_mana(t, P0, ManaType::C, 2);
    t.cast(P0, card).method(CastMethod::Half(0)).go();
    t.answer_targets(P0, &[Entity::Object(target)]);
    t.resolve_all();
    t.named_on_battlefield("Glassworks")[0]
}

#[test]
fn paying_a_locked_doors_mana_cost_unlocks_it_as_a_special_action() {
    cr!("116.2m", "709.5", "709.5c", "709.5d", "709.5e", "709.5h");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let ogre = t.battlefield(P1, "Gray Ogre");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Glassworks {2}{R}: "When you unlock this door, this Room deals 4 damage to target
    // creature an opponent controls." Casting it unlocks that door on the battlefield,
    // which triggers the ability.
    let room = cast_glassworks(&mut t, ogre);
    assert!(!t.on_battlefield(ogre));
    assert_eq!(rooms::unlocked(&t.g, room), [true, false]);
    // Shattered Yard is locked: the permanent doesn't have its name, mana cost or text.
    assert_eq!(t.obj(room).chars.name, "Glassworks");
    assert_eq!(t.obj(room).chars.mana_value(), 3);
    assert_eq!(t.obj(room).chars.abilities.len(), 1);
    // P0 may pay Shattered Yard's mana cost {4}{R} to unlock it.
    assert!(unlock_actions(&mut t, P0, room).is_empty());
    add_mana(&mut t, P0, ManaType::R, 1);
    add_mana(&mut t, P0, ManaType::C, 4);
    assert_eq!(unlock_actions(&mut t, P0, room), vec![unlock_door(1, room)]);
    t.g.perform_action(P0, Action::Special(unlock_door(1, room)))
        .unwrap();
    t.g.recompute();
    assert_eq!(t.player(P0).mana_pool.total(), 0);
    assert_eq!(rooms::unlocked(&t.g, room), [true, true]);
    assert_eq!(t.obj(room).chars.name, "Glassworks // Shattered Yard");
    assert_eq!(t.obj(room).chars.mana_value(), 8);
    assert_eq!(t.obj(room).chars.abilities.len(), 2);
    // It doesn't use the stack, and unlocking the other door doesn't trigger
    // Glassworks's ability: nothing is on the stack and the Bears are unharmed.
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.obj(bears).damage, 0);
    // Nothing is left to unlock.
    add_mana(&mut t, P0, ManaType::R, 5);
    assert!(unlock_actions(&mut t, P0, room).is_empty());
}

#[test]
fn unlocking_is_only_possible_at_sorcery_timing() {
    cr!("116.2m");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let ogre = t.battlefield(P1, "Gray Ogre");
    let room = cast_glassworks(&mut t, ogre);
    add_mana(&mut t, P0, ManaType::R, 1);
    add_mana(&mut t, P0, ManaType::C, 4);
    // Not with a spell on the stack.
    let gift = t.custom(P0, free_instant("Quick Gift"), Zone::Hand(P0));
    t.cast(P0, gift).go();
    assert!(unlock_actions(&mut t, P0, room).is_empty());
    assert!(t
        .g
        .perform_action(P0, Action::Special(unlock_door(1, room)))
        .is_err());
    t.resolve_all();
    // Not during combat.
    t.set_step(P0, Step::DeclareAttackers);
    assert!(unlock_actions(&mut t, P0, room).is_empty());
    // Not during an opponent's turn, even in their main phase.
    t.set_step(P1, Step::PrecombatMain);
    assert!(unlock_actions(&mut t, P0, room).is_empty());
    // Only the Room's controller may do it.
    t.set_step(P1, Step::PostcombatMain);
    add_mana(&mut t, P1, ManaType::R, 1);
    add_mana(&mut t, P1, ManaType::C, 4);
    assert!(unlock_actions(&mut t, P1, room).is_empty());
    // In P0's second main phase with an empty stack, P0 may.
    t.set_step(P0, Step::PostcombatMain);
    assert_eq!(unlock_actions(&mut t, P0, room), vec![unlock_door(1, room)]);
}

#[test]
fn a_room_put_onto_the_battlefield_without_being_cast_has_both_doors_locked() {
    cr!("116.2m", "709.5", "709.5d", "709.5i");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // Entity Tracker: "Eerie — Whenever an enchantment you control enters and whenever
    // you fully unlock a Room, draw a card."
    t.battlefield(P0, "Entity Tracker");
    for _ in 0..5 {
        t.library_top(P0, "Island");
    }
    let room = t.enter(P0, "Glassworks // Shattered Yard");
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(rooms::unlocked(&t.g, room), [false, false]);
    // No name, no mana cost (it's colorless), no abilities.
    assert_eq!(t.obj(room).chars.name, "");
    assert_eq!(t.obj(room).chars.mana_value(), 0);
    assert_eq!(t.obj(room).chars.colors, ColorSet::NONE);
    assert!(t.obj(room).chars.abilities.is_empty());
    assert!(t.obj(room).chars.has_subtype("Room"));
    // Both doors can be unlocked, each for its own cost.
    add_mana(&mut t, P0, ManaType::R, 1);
    add_mana(&mut t, P0, ManaType::C, 2);
    assert_eq!(unlock_actions(&mut t, P0, room), vec![unlock_door(0, room)]);
    t.g.perform_action(P0, Action::Special(unlock_door(0, room)))
        .unwrap();
    // Glassworks's unlock trigger has no legal target (P1 controls no creature), so it's
    // removed; P0 hasn't fully unlocked the Room yet.
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    add_mana(&mut t, P0, ManaType::R, 1);
    add_mana(&mut t, P0, ManaType::C, 4);
    t.g.perform_action(P0, Action::Special(unlock_door(1, room)))
        .unwrap();
    t.settle();
    t.resolve_all();
    // Unlocking the second door fully unlocks it: Entity Tracker draws a card.
    assert_eq!(t.hand_size(P0), 2);
}
