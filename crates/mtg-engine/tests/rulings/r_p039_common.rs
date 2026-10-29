//! Shared helpers for the tests of rulings batch P039 (`r_p039_*.rs`): abilities that
//! count creatures, creatures entering the battlefield ("creaturefall"), lands and
//! Vehicles that become creatures, and "whenever a creature deals combat damage to a
//! player, draw a card" abilities. (The helpers of batches S01, S02 and S04 are used too.)

#![allow(dead_code)]

use crate::r_s01_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

pub fn o(id: ObjectId) -> Entity {
    Entity::Object(id)
}

pub fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// Adds `n` mana of each type to `p`'s pool.
pub fn mana_all(t: &mut TestGame, p: PlayerId, n: u32) {
    for ty in [
        ManaType::W,
        ManaType::U,
        ManaType::B,
        ManaType::R,
        ManaType::G,
        ManaType::C,
    ] {
        t.g.players[p.idx()].mana_pool.add_type(ty, n);
    }
}

/// Adds `n` mana of type `ty` to `p`'s pool.
pub fn mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

/// `p` casts the real card `name` from their hand (with lands for its mana cost) with
/// the given targets (one per slot); the spell is left on the stack.
pub fn cast_named(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    supported(name);
    give_mana_for(t, p, name);
    let c = t.hand(p, name);
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    t.cast(p, c).go()
}

/// `p` puts the real creature card `name` onto the battlefield through a real zone change
/// and the "enters" triggers are put on the stack.
pub fn enter(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.enter(p, name);
    t.g.flush_events();
    t.settle();
    id
}

/// A permanent that has been under its controller's control since the start of the turn
/// (not "entered this turn").
pub fn old(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.battlefield(p, name);
    t.g.objects[id.0 as usize].entered_turn = 0;
    id
}

/// The objects on the battlefield controlled by `p` named `name`.
pub fn named(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.name == name)
        .map(|o| o.id)
        .collect()
}

/// Number of decisions asked so far (an index into `t.asked()`).
pub fn mark(t: &TestGame) -> usize {
    t.asked().len()
}

/// The players asked a yes/no question whose text contains `text` since `from`.
pub fn asked_yes_no(t: &TestGame, from: usize, text: &str) -> Vec<PlayerId> {
    t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::YesNo { prompt, .. } if prompt.contains(text) => Some(*p),
            _ => None,
        })
        .collect()
}

/// All yes/no questions asked since `from`, with who was asked.
pub fn yes_nos(t: &TestGame, from: usize) -> Vec<(PlayerId, String)> {
    t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::YesNo { prompt, .. } => Some((*p, prompt.to_string())),
            _ => None,
        })
        .collect()
}

/// Goes to `p`'s combat (beginning of combat step), declares the attackers and blocks
/// and runs combat to the end of combat step; triggers along the way resolve with the
/// scripted answers (or defaults).
pub fn fight(t: &mut TestGame, attackers: &[(ObjectId, Entity)], blocks: &[(ObjectId, ObjectId)]) {
    let ap = t.g.turn.active;
    if !t.g.turn.step.is_combat() {
        t.set_step(ap, Step::BeginningOfCombat);
    }
    t.attack(attackers, blocks);
}

/// Answers `p`'s next yes/no question.
pub fn yes(t: &mut TestGame, p: PlayerId, y: bool) {
    t.answer(p, DecisionKind::YesNo, Answer::Bool(y));
}

/// The battlefield permanent `id` is a creature now.
pub fn creature_now(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).is(CardType::Creature)
}

/// Moves the card object `id` (created in no zone) onto the battlefield under `p`'s
/// control through a real zone change, and puts the triggers on the stack; the new object.
pub fn enter_obj(t: &mut TestGame, p: PlayerId, id: ObjectId) -> ObjectId {
    let new = enter_quiet(t, p, id);
    t.settle();
    new
}

/// Like [`enter_obj`], without putting the triggers on the stack yet (so that their
/// targets can be scripted naming the new object).
pub fn enter_quiet(t: &mut TestGame, p: PlayerId, id: ObjectId) -> ObjectId {
    let new = t
        .g
        .move_object_ev(mtg_engine::replacement::MoveEv {
            obj: id,
            to: mtg_engine::object::Zone::Battlefield,
            pos: mtg_engine::ability::LibraryPosition::Top,
            cause: mtg_engine::events::MoveCause::Effect,
            by: Some(p),
            etb: mtg_engine::replacement::EtbInfo {
                controller: Some(p),
                ..Default::default()
            },
            source: None,
        })
        .expect("failed to enter the battlefield");
    t.g.flush_events();
    new
}

/// A new card object for the real card `name`, owned by `p`, in no zone yet.
pub fn new_card(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    t.g.create_card_object(
        mtg_engine::card::card(name),
        p,
        mtg_engine::object::Zone::Nowhere,
    )
}

/// The casting methods `p` has for `card` now (from the legal actions).
pub fn cast_methods(t: &mut TestGame, p: PlayerId, card: ObjectId) -> Vec<mtg_engine::object::CastMethod> {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .into_iter()
        .filter_map(|a| match a {
            mtg_engine::decision::Action::Cast { card: c, method } if c == card => Some(method),
            _ => None,
        })
        .collect()
}
