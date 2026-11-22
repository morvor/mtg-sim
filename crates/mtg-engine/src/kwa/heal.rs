//! CR 701.69: heal.
//!
//! To heal damage already dealt to a permanent is to remove that marked damage from it;
//! damage that "is healed" is all of it (CR 701.69a). The keyword action heals N damage
//! from each object (all of it if N is negative).
//!
//! "If damage would be dealt to [this], instead that damage is dealt, but all other damage
//! already dealt to him is healed" (Wolverine) heals, right after the damage is dealt, the
//! damage marked on it other than that damage: N = [`DAMAGE_MARKED_ON_IT`] minus the
//! event's amount.

use super::*;
use crate::keywords::KeywordKind;

/// `Value::Custom`: the damage marked on the source of the ability.
pub const DAMAGE_MARKED_ON_IT: &str = "heal: damage marked on it";

/// Heals `n` damage marked on `obj` (all of it if `None`). Returns the damage healed.
pub fn heal(g: &mut Game, obj: ObjectId, n: Option<u32>) -> u32 {
    if !on_battlefield(g, obj) {
        return 0;
    }
    let marked = g.obj(obj).damage;
    let healed = n.map_or(marked, |n| n.min(marked));
    if healed == 0 {
        return 0;
    }
    let o = &mut g.objects[obj.0 as usize];
    o.damage -= healed;
    if o.damage == 0 {
        o.deathtouch_damage = false;
    }
    g.dirty = true;
    g.log(|g| format!("{healed} damage on {} is healed", g.describe(obj)));
    healed
}

pub struct Heal;

impl KeywordActionRules for Heal {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Heal]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let n = g.eval_value(a.n, ctx);
        let n = (n >= 0).then_some(n as u32);
        for o in g.resolve_objects(a.what, ctx) {
            heal(g, o, n);
        }
    }
}

inventory::submit! { KeywordActionRegistration(&Heal) }

/// The value [`DAMAGE_MARKED_ON_IT`].
pub struct HealValues;

impl crate::kw::KeywordRules for HealValues {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        (name == DAMAGE_MARKED_ON_IT).then(|| ctx.source.map_or(0, |s| g.obj(s).damage as i64))
    }
}

inventory::submit! { crate::kw::KeywordRegistration(&HealValues) }
