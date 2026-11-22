//! Shared helpers for the tests of rulings batch P079 (`r_p079_*.rs`): cards that punish
//! or answer attackers ("hate-attacker" cards): spells for attacking or blocking
//! creatures, attack triggers of the defending player, and combat bookkeeping.

#![allow(dead_code)]

use mtg_engine::decision::Action;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Puts the real card `name` onto the battlefield under `p`'s control attacking `target`
/// (as "put onto the battlefield tapped and attacking" would, CR 508.4).
pub fn enter_attacking(t: &mut TestGame, p: PlayerId, name: &str, target: Entity) -> ObjectId {
    let id = t.custom(p, (*mtg_engine::card::card(name)).clone(), Zone::Exile);
    let new =
        t.g.move_object_ev(mtg_engine::replacement::MoveEv {
            obj: id,
            to: Zone::Battlefield,
            pos: mtg_engine::ability::LibraryPosition::Top,
            cause: mtg_engine::events::MoveCause::Effect,
            by: Some(p),
            etb: mtg_engine::replacement::EtbInfo {
                controller: Some(p),
                tapped: true,
                attacking: Some(target),
                ..Default::default()
            },
            source: None,
        })
        .expect("entered");
    t.g.flush_events();
    t.settle();
    new
}

/// Puts the back face of the real transforming card `name` onto the battlefield under
/// `p`'s control (transformed).
pub fn back_face(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.custom(p, (*mtg_engine::card::card(name)).clone(), Zone::Exile);
    let new =
        t.g.move_object_ev(mtg_engine::replacement::MoveEv {
            obj: id,
            to: Zone::Battlefield,
            pos: mtg_engine::ability::LibraryPosition::Top,
            cause: mtg_engine::events::MoveCause::Effect,
            by: Some(p),
            etb: mtg_engine::replacement::EtbInfo {
                controller: Some(p),
                transformed: true,
                ..Default::default()
            },
            source: None,
        })
        .expect("entered");
    t.g.flush_events();
    t.settle();
    // Not summoning sick: it's been there since the turn began.
    t.g.objects[new.0 as usize].summoning_sick = false;
    new
}

/// Adds `n` mana of type `ty` to `p`'s mana pool.
pub fn add(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

/// `p` puts the real card `name` into their hand and casts it with mana of `ty` in their
/// pool (enough for any mana cost up to 8), with `targets` (one per target slot).
pub fn cast_card(
    t: &mut TestGame,
    p: PlayerId,
    name: &str,
    targets: &[Entity],
) -> Result<ObjectId, mtg_engine::casting::Illegal> {
    let c = t.hand(p, name);
    crate::r_s25_common::lands_for_cost(t, p, name);
    if name.contains(" // ") {
        // A split card: cast its first half.
        for e in targets {
            t.answer_targets(p, &[*e]);
        }
        return t
            .cast(p, c)
            .method(mtg_engine::object::CastMethod::Half(0))
            .try_go();
    }
    t.cast_with(p, c, targets)
}

/// Whether `p` has `card` among the cards they could cast now.
pub fn can_cast_now(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let card = t.g.current(card);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Cast { card: c, .. } if *c == card))
}

/// `p` concedes (leaves the game, CR 800.4a).
pub fn concede(t: &mut TestGame, p: PlayerId) {
    t.g.take_action(p, Action::Concede);
    t.g.flush_events();
    t.settle();
}
