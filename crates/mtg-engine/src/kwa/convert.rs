//! CR 701.28: convert.
//!
//! * To convert a permanent is to turn it so that its other face is up, following the
//!   rules for transforming (CR 701.28a): `dfc::transform`, with the same event, so
//!   "whenever [a permanent] transforms" also triggers when it converts.
//! * Converting isn't turning a permanent face up or face down (CR 701.28b).
//! * A permanent that isn't a double-faced card or token doesn't convert (CR 701.28c), nor
//!   one whose other face is an instant or sorcery face (CR 701.28d).
//! * An ability of a permanent converts it only if it hasn't converted or transformed
//!   since the ability was put onto the stack (for a delayed triggered ability, since it
//!   was created) (CR 701.28e): converting and transforming are recorded together.
//! * A permanent that can't transform can't convert either (CR 701.28f):
//!   [`cant_transform`], checked for both.

use super::*;

/// Whether a "can't transform" effect applies to `id` (CR 701.28f): a static ability
/// ("Non-Human Werewolves you control can't transform") or a rules effect of a resolved
/// spell or ability.
pub fn cant_transform(g: &Game, id: ObjectId) -> bool {
    g.statics.restrictions.iter().any(|(s, c, r)| match r {
        Restriction::CantTransform(f) => g.matches(id, f, &Ctx::new(Some(*s), *c)),
        _ => false,
    }) || g.rule_effects.iter().any(|e| match &e.restriction {
        Restriction::CantTransform(f) => {
            e.objects.as_ref().is_none_or(|v| v.contains(&id))
                && g.matches(id, f, &Ctx::new(e.source, e.controller))
        }
        _ => false,
    })
}

pub struct Convert;

impl KeywordActionRules for Convert {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Convert]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        for o in g.resolve_objects(a.what, ctx) {
            // CR 701.28e: not if it converted or transformed since its ability was put
            // onto the stack (or, for a delayed triggered ability, created).
            if crate::transform_rules::ability_may_transform(g, o, ctx) {
                crate::dfc::transform(g, o);
            }
        }
    }
}

inventory::submit! { KeywordActionRegistration(&Convert) }
