//! CR 701.64: harness.
//!
//! * "Harness [this permanent]" means "If this permanent isn't harnessed, it becomes
//!   harnessed" (CR 701.64a).
//! * Harnessed is a designation of permanents with no rules meaning of its own: only a
//!   permanent can become harnessed, and it stays harnessed until it leaves the
//!   battlefield (a new object isn't harnessed, CR 400.7). It's neither an ability nor a
//!   copiable value (CR 701.64b). The filter [`HARNESSED`] checks the designation; the
//!   Infinity keyword ("∞ — [ability]", CR 702.186) grants its ability as long as the
//!   permanent is harnessed.

use super::*;
use crate::keywords::KeywordKind;

/// `Filter::Custom` name: "[this permanent] is harnessed"; also the `Event::Custom` name
/// reported when a permanent becomes harnessed.
pub const HARNESSED: &str = "harnessed";

/// Whether `id` is a harnessed permanent.
pub fn is_harnessed(g: &Game, id: ObjectId) -> bool {
    on_battlefield(g, id) && g.kwa.harnessed.contains(&id)
}

/// Harnesses `obj` (CR 701.64a). Returns true if it became harnessed.
pub fn harness(g: &mut Game, obj: ObjectId) -> bool {
    if !on_battlefield(g, obj) || is_harnessed(g, obj) {
        return false;
    }
    g.kwa.harnessed.push(obj);
    g.dirty = true;
    let p = g.obj(obj).controller;
    g.log(|g| format!("{} becomes harnessed", g.describe(obj)));
    emit(g, HARNESSED, p, Some(obj), 0);
    true
}

pub struct Harness;

impl KeywordActionRules for Harness {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Harness]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let mut any = false;
        for obj in g.resolve_objects(a.what, ctx) {
            any |= harness(g, obj);
        }
        ctx.prev_happened = any;
    }
}

inventory::submit! { KeywordActionRegistration(&Harness) }

struct HarnessedRules;

impl crate::kw::KeywordRules for HarnessedRules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        (name == HARNESSED).then(|| is_harnessed(g, id))
    }
}

inventory::submit! { crate::kw::KeywordRegistration(&HarnessedRules) }
