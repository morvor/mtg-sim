//! "~ loses this ability and becomes an Aura enchantment with enchant creature. Attach it
//! to target creature. You may pay {W} to end this effect." (the Licids):
//!
//! * [`LOSE_THIS_ABILITY`]: a layer-6 modification removing the ability whose text says
//!   the object loses it (CR 613.1f);
//! * [`END_MARKER`]: marks the continuous effects a later special action may end; it
//!   changes nothing itself (an effect that makes its object lose this ability is marked
//!   by that too);
//! * [`END_THIS_EFFECT`]: the special action's effect (CR 116.2c): the marked effects
//!   from the same source end, so the object is what it was before them.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Characteristics;
use crate::types::*;

pub const LOSE_THIS_ABILITY: &str = "end this effect: loses this ability";
pub const END_MARKER: &str = "end this effect: marker";
pub const END_THIS_EFFECT: &str = "end this effect";

pub struct EndThisEffect;

impl KeywordRules for EndThisEffect {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_modification(
        &self,
        _g: &Game,
        name: &str,
        chars: &mut Characteristics,
        _ctx: &Ctx,
        _target: ObjectId,
    ) -> bool {
        match name {
            LOSE_THIS_ABILITY => {
                chars
                    .abilities
                    .retain(|a| !a.text.to_lowercase().contains("loses this ability"));
                true
            }
            END_MARKER => true,
            _ => false,
        }
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != END_THIS_EFFECT {
            return false;
        }
        let Some(src) = ctx.source else {
            return true;
        };
        let marked = |e: &crate::game::ContinuousEffect| {
            e.source == Some(src)
                && e.mods.iter().any(|m| {
                    matches!(m, Modification::Custom { name, .. }
                        if name == END_MARKER || name == LOSE_THIS_ABILITY)
                })
        };
        let before = g.effects.len();
        g.effects.retain(|e| !marked(e));
        if g.effects.len() != before {
            g.dirty = true;
            g.recompute();
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&EndThisEffect) }
