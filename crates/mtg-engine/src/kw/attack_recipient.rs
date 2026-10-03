//! "the player or planeswalker [it's / that creature is] attacking" (Hellrider, Raid
//! Bombardment, Mage Slayer, Scorch Spitter, Myr Battlesphere): the player, planeswalker
//! or battle an attacking creature is attacking (CR 506.2, 508.1b). It isn't a target.
//!
//! The recipient is looked up as the ability resolves: what the creature is attacking now,
//! or, if it's no longer attacking (it left the battlefield or was removed from combat),
//! what it was attacking when the ability triggered — the trigger event records it — so
//! the ability still deals its damage (Cavalcade of Calamity, Raid Bombardment rulings).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{Effect, Sel, Var};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// `Effect::Custom` prefix: store what the selected attacking creature is attacking in a
/// variable (JSON of `(Var, Sel)`).
pub const ATTACK_RECIPIENT: &str = "attack_recipient:";

/// The effect storing in `var` the player or permanent `attacker` is attacking.
pub fn attack_recipient_effect(var: Var, attacker: &Sel) -> Option<Effect> {
    let json = serde_json::to_string(&(var, attacker)).ok()?;
    Some(Effect::Custom(format!("{ATTACK_RECIPIENT}{json}").into()))
}

/// What the creature selected by `attacker` is attacking (see the module docs).
pub fn attack_recipient(g: &Game, attacker: &Sel, ctx: &Ctx) -> Option<Entity> {
    let combat = g.combat.as_ref();
    let live = g
        .eval_sel_objects(attacker, ctx)
        .into_iter()
        .find_map(|o| combat.and_then(|c| c.attack_target(g.current(o))));
    if live.is_some() {
        return live;
    }
    // The creature is no longer attacking: what it attacked when the ability triggered.
    let ev = ctx.event.as_ref()?;
    if let Some(o) = ev.other {
        return Some(Entity::Object(o));
    }
    ev.player.map(Entity::Player)
}

pub struct AttackRecipient;

impl KeywordRules for AttackRecipient {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some(json) = name.strip_prefix(ATTACK_RECIPIENT) else {
            return false;
        };
        let Ok((var, attacker)) = serde_json::from_str::<(Var, Sel)>(json) else {
            return true;
        };
        let to: Vec<Entity> = attack_recipient(g, &attacker, ctx).into_iter().collect();
        ctx.prev_happened = !to.is_empty();
        ctx.set_var(var, to);
        true
    }
}

inventory::submit! { KeywordRegistration(&AttackRecipient) }
