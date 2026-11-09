//! Shared helpers for the tests of rulings batch P049 (`r_p049_*.rs`): "drain" spells and
//! abilities (damage or life loss plus life gain), crimes, token copies, and the
//! Conspiracy draft-matters cards. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::ability::{LibraryPosition, TokenSpec};
use mtg_engine::card::card;
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

pub use crate::r_p046_common::{cost_to_cast, fill_pool, pool, tap_lands};
pub use crate::r_p048_common::*;
pub use crate::r_p127_common::untap;
pub use crate::r_p108_common::{end_step, resolved, two_headed_giant};
pub use crate::r_s01_common::{attack_with, block_and_finish, supported, triggers_on_stack, watch};
pub use crate::r_s02_common::{can_activate, can_cast, create_token, target_candidates};
pub use crate::r_s06_common::give_control;
pub use crate::r_s24_common::enter_together;
pub use crate::r_s25_common::{abilities_from, cast_new, lands_for_cost, name_now, spell_copies, x_of};
pub use crate::r_s30_common::set_life;

/// The real card `name`, owned by `owner`, enters the battlefield under `controller`'s
/// control (one zone change: enters abilities trigger and replacement effects apply).
pub fn enter_owned_by(t: &mut TestGame, owner: PlayerId, controller: PlayerId, name: &str) -> ObjectId {
    let id = t.g.create_card_object(card(name), owner, Zone::Nowhere);
    let r = t
        .g
        .move_object_ev(MoveEv {
            obj: id,
            to: Zone::Battlefield,
            pos: LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(controller),
            etb: EtbInfo {
                controller: Some(controller),
                ..Default::default()
            },
            source: None,
        })
        .expect("failed to enter the battlefield");
    t.g.flush_events();
    t.settle();
    r
}

/// `p` creates a token with the characteristics of the real card `name` (as a token copy
/// of that permanent would have), as a resolving effect would, and it's returned. Its
/// enters abilities trigger and "enters with" replacement effects apply.
pub fn token_like(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    use mtg_engine::ability::{Effect, PlayerRef, Value};
    let c = card(name);
    let ch = &c.front().chars;
    let spec = TokenSpec {
        name: ch.name.clone(),
        colors: ch.colors,
        supertypes: ch.supertypes.iter().collect(),
        card_types: ch.card_types.iter().collect(),
        subtypes: ch.subtypes.iter().cloned().collect(),
        power: ch.power,
        toughness: ch.toughness,
        abilities: ch.abilities.clone(),
        scryfall_name: None,
        pt_values: None,
    };
    let before = t.g.battlefield.clone();
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    t.g.exec(
        &Effect::CreateToken {
            spec,
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
        },
        &mut ctx,
    );
    t.g.recompute();
    let token = t
        .g
        .battlefield
        .iter()
        .copied()
        .find(|id| !before.contains(id))
        .expect("no token was created");
    t.g.objects[token.0 as usize].summoning_sick = false;
    t.g.flush_events();
    t.settle();
    token
}

/// The life `p` has lost this turn (CR 119.3), as the game's history records it.
pub fn life_lost(t: &TestGame, p: PlayerId) -> u32 {
    t.g.history.life_lost.get(&p).copied().unwrap_or(0)
}

/// The number of crimes `p` has committed this turn (CR 700.13).
pub fn crimes(t: &TestGame, p: PlayerId) -> u32 {
    t.g.history.crimes.get(&p).copied().unwrap_or(0)
}

/// The objects on the stack, bottom first.
pub fn stack(t: &TestGame) -> Vec<ObjectId> {
    t.g.stack.clone()
}

/// Whether the permanent (followed across zone changes) is tapped now.
pub fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

/// `p`'s mana pool total.
pub fn mana(t: &TestGame, p: PlayerId) -> usize {
    t.g.player(p).mana_pool.total()
}
