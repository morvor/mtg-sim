//! "the player or planeswalker [it's / that creature is] attacking" (Hellrider, Raid
//! Bombardment, Mage Slayer, Scorch Spitter, Myr Battlesphere): the player, planeswalker
//! or battle an attacking creature is attacking (CR 506.2, 508.1b). It isn't a target.
//!
//! The recipient is looked up as the ability resolves: what the creature is attacking now,
//! or, if it left the battlefield, what it was attacking as it left, so the ability still
//! deals its damage (Cavalcade of Calamity, Raid Bombardment rulings). A creature removed
//! from combat but still on the battlefield isn't attacking anything (Fathom Fleet
//! Swordjack ruling).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{Effect, Sel, Var};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
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
///
/// - Still attacking: what it's attacking now (nothing if the planeswalker it attacked
///   was removed from combat, CR 506.4c).
/// - Still on the battlefield but removed from combat: nothing (Fathom Fleet Swordjack
///   ruling).
/// - Left the battlefield: what it was attacking as it left (Raid Bombardment, Cavalcade
///   of Calamity, Fathom Fleet Swordjack, Myr Battlesphere rulings).
///
/// A battle is neither a player nor a planeswalker: nothing (Raid Bombardment ruling).
pub fn attack_recipient(g: &Game, attacker: &Sel, ctx: &Ctx) -> Option<Entity> {
    let combat = g.combat.as_ref()?;
    let mut ids = g.eval_sel_objects(attacker, ctx);
    // The attacking creature of an attack / becomes-blocked trigger, if the selection no
    // longer finds it.
    if let Some(o) = ctx.event.as_ref().and_then(|e| e.object) {
        if !ids.contains(&o) {
            ids.push(o);
        }
    }
    let recipient = ids.iter().find_map(|&o| {
        let cur = g.current(o);
        if let Some(a) = combat.attacker(cur) {
            return Some(a.target);
        }
        if g.is_live(cur) && g.obj(cur).zone == Zone::Battlefield {
            // Removed from combat but still here.
            return Some(None);
        }
        combat
            .removed_attack_targets
            .iter()
            .rev()
            .find(|(id, _)| *id == o || *id == cur)
            .map(|(_, t)| Some(*t))
    })??;
    match recipient {
        Entity::Object(o) if g.try_obj(o).is_some_and(|ob| ob.is(CardType::Battle)) => None,
        t => Some(t),
    }
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
