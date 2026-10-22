//! "Once during each of your turns, you may cast a creature spell from your graveyard."
//! (Karador, Ghost Chieftain; Danitha, New Benalia's Light; Lurrus of the Dream-Den): a
//! permission to cast one spell with that quality from the graveyard each of its
//! controller's turns (CR 601.3). It's a static ability granting a
//! `StaticEffect::PlayPermission` whose condition is "it's your turn and you haven't used
//! it this turn" ([`once_unused`]). The spell is cast normally: its costs are paid, or an
//! alternative cost instead (CR 118.9); the usual timing rules apply.
//!
//! "During each of your turns, you may play a land and cast a permanent spell of each
//! permanent type from your graveyard." (Muldrotha, the Gravetide) is one such permission
//! per use — a land, an artifact spell, a creature spell, ... — each with its own slot
//! ([`once_unused`] with the type's name): a card with several permanent types uses the
//! slot of the type its player chooses as they play it (the permission they announce they're
//! using), judged by the characteristics it has as it's played (CR 601.3e).
//!
//! A permission is used when a card is played with it: the player announces which
//! permission they're using as they begin to play the card (CR 601.2, 305.1; see
//! `permissions.rs`), and it's recorded then (`TurnHistory::once_permissions_used`, keyed
//! by the object and the slot). A card cast with a keyword that is its own permission to
//! cast it from a graveyard (flashback, escape, ...), with an effect's permission for that
//! card ("You may cast that card this turn"), or with another permission doesn't use it. A
//! new object with the ability (another Karador) has a permission of its own.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use smol_str::SmolStr;

/// `Condition::Custom`: this object's once-each-turn permission hasn't been used this turn.
/// Followed by `:` and a slot name for one of several such permissions of the object.
pub const ONCE_UNUSED: &str = "once each turn: permission unused";

/// The condition "this object's once-each-turn permission `slot` ("" for its only one)
/// hasn't been used this turn".
pub fn once_unused(slot: &str) -> Condition {
    if slot.is_empty() {
        Condition::Custom(ONCE_UNUSED.into())
    } else {
        Condition::Custom(format!("{ONCE_UNUSED}:{slot}").into())
    }
}

/// The slot named by a [`once_unused`] condition.
fn slot_of(name: &str) -> Option<SmolStr> {
    let rest = name.strip_prefix(ONCE_UNUSED)?;
    if rest.is_empty() {
        return Some(SmolStr::default());
    }
    rest.strip_prefix(':').map(SmolStr::new)
}

/// If the static ability is a once-each-turn permission, the slot it uses ("" for an
/// object's only one).
pub fn once_slot(s: &StaticAbility) -> Option<SmolStr> {
    if !matches!(s.effect, StaticEffect::PlayPermission(_)) {
        return None;
    }
    s.condition.as_ref().and_then(condition_slot)
}

/// The slot a condition (a static ability's) makes it a once-each-turn use of, if it's
/// [`once_unused`] or a conjunction with it.
pub fn condition_slot(c: &Condition) -> Option<SmolStr> {
    match c {
        Condition::Custom(n) => slot_of(n),
        Condition::And(v) => v.iter().find_map(condition_slot),
        _ => None,
    }
}

pub struct OnceEachTurnCast;

impl KeywordRules for OnceEachTurnCast {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        let slot = slot_of(name)?;
        let src = ctx.source?;
        Some(
            !g.history
                .once_permissions_used
                .iter()
                .any(|(o, s)| *o == src && *s == slot),
        )
    }
}

inventory::submit! { KeywordRegistration(&OnceEachTurnCast) }
