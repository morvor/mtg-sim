//! "You may look at face-down creatures you don't control any time." (Keeper of the Lens,
//! Lens of Clarity; "Until end of turn, ..." for Lumbering Laundry): normally a player may
//! look only at the face-down permanents they control (CR 708.5); this lets them look at
//! the others too, while the ability (or the effect) applies.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::PlayerModification;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::types::*;

/// `StaticEffect::Custom` (and `PlayerModification::Custom` for an effect lasting a
/// while): the player may look at face-down creatures they don't control.
pub const LOOK_AT_FACE_DOWN_CREATURES: &str = "may look at face-down creatures you don't control";

/// Whether `p` may look at `id` because of such an ability or effect.
pub fn may_look(g: &Game, p: PlayerId, id: ObjectId) -> bool {
    let o = g.obj(id);
    if o.zone != Zone::Battlefield || !o.face_down || !o.chars.is(CardType::Creature) {
        return false;
    }
    if o.controller == p {
        return false;
    }
    g.statics
        .customs
        .iter()
        .any(|(_, ctl, n)| *ctl == p && n == LOOK_AT_FACE_DOWN_CREATURES)
        || g.player(p).has_mod(
            |m| matches!(m, PlayerModification::Custom(n) if n == LOOK_AT_FACE_DOWN_CREATURES),
        )
}

pub struct LookFaceDown;

impl KeywordRules for LookFaceDown {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
}

inventory::submit! { KeywordRegistration(&LookFaceDown) }
