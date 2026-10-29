//! CR 701.37: monstrosity.
//!
//! * "Monstrosity N" means "If this permanent isn't monstrous, put N +1/+1 counters on it
//!   and it becomes monstrous" (CR 701.37a).
//! * Monstrous is a designation of permanents (`GameObject::monstrous`): only a
//!   permanent can become monstrous, and it stays monstrous until it leaves the
//!   battlefield (a new object isn't monstrous, CR 400.7). It's neither an ability nor a
//!   copiable value (CR 701.37b): the filter [`MONSTROUS`] checks the designation itself.
//! * Becoming monstrous reports a [`MONSTROUS`] event whose amount is N, so abilities
//!   that trigger "when [this] becomes monstrous" can refer to the X it became monstrous
//!   with (CR 701.37c).

use super::*;
use crate::keywords::KeywordKind;
use crate::types::counters;

/// `Filter::Custom` name ("[this] is monstrous") and `Event::Custom` name (a permanent
/// became monstrous; the amount is its N).
pub const MONSTROUS: &str = "monstrous";

/// "Monstrosity N" for `obj`: returns true if it became monstrous.
pub fn monstrosity(g: &mut Game, obj: ObjectId, n: u32, source: Option<ObjectId>) -> bool {
    // Only a permanent can become monstrous (CR 701.37b), and not again.
    if !on_battlefield(g, obj) || g.obj(obj).monstrous {
        return false;
    }
    g.add_counters(Entity::Object(obj), counters::PLUS1, n, source);
    if !on_battlefield(g, obj) {
        return false;
    }
    g.objects[obj.0 as usize].monstrous = true;
    g.dirty = true;
    let p = g.obj(obj).controller;
    g.log(|g| format!("{} becomes monstrous", g.describe(obj)));
    emit(g, MONSTROUS, p, Some(obj), n as i32);
    true
}

pub struct Monstrosity;

impl KeywordActionRules for Monstrosity {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Monstrosity]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let n = number(g, a.n, ctx);
        let mut any = false;
        for obj in g.resolve_objects(a.what, ctx) {
            any |= monstrosity(g, obj, n, ctx.source);
        }
        ctx.prev_happened = any;
    }
}

inventory::submit! { KeywordActionRegistration(&Monstrosity) }

/// The monstrous designation as an object filter.
struct MonstrousRules;

impl crate::kw::KeywordRules for MonstrousRules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        (name == MONSTROUS).then(|| g.obj(id).monstrous && g.obj(id).zone == Zone::Battlefield)
    }
}

inventory::submit! { crate::kw::KeywordRegistration(&MonstrousRules) }
