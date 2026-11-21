//! "If ~ is in a graveyard, effects from spells named Flame Burst count it as a card
//! named Flame Burst." (Pardic Firecat; Diligent Farmhand with Muscle Burst): while the
//! card is in a graveyard, an effect of a spell with that name sees it as a card with that
//! name too (CR 201.2a), as when Flame Burst counts "cards named Flame Burst in all
//! graveyards". Other effects see its own name only.
//!
//! The ability is a static ability with `StaticEffect::Custom` named [`PREFIX`] plus the
//! name, read from the card's own abilities.

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::object::Zone;
use crate::types::ObjectId;

/// `StaticEffect::Custom` prefix, followed by the name.
pub const PREFIX: &str = "counts as named for spells named:";

/// Whether the effect of `ctx` counts `id` as a card named `name`: `id` is in a graveyard
/// with the ability for that name, and the effect's source is a spell with that name.
pub fn counts_as_named(g: &Game, id: ObjectId, name: &str, ctx: &Ctx) -> bool {
    let o = g.obj(id);
    if !matches!(o.zone, Zone::Graveyard(_)) {
        return false;
    }
    let has = o.chars.abilities.iter().any(|a| {
        matches!(&a.kind, AbilityKind::Static(st)
            if matches!(&st.effect, StaticEffect::Custom(n)
                if n.strip_prefix(PREFIX) == Some(name)))
    });
    if !has {
        return false;
    }
    let Some(src) = ctx.source else {
        return false;
    };
    let s = g.obj(src);
    (s.is(crate::types::CardType::Instant) || s.is(crate::types::CardType::Sorcery))
        && s.chars.has_name(name)
}

/// [`counts_as_named`] for any of the names of the objects `sel` selects ("cards named ~"
/// on the spell itself).
pub fn counts_as_named_like(g: &Game, id: ObjectId, sel: &Sel, ctx: &Ctx) -> bool {
    g.eval_sel(sel, ctx)
        .iter()
        .filter_map(|e| e.object())
        .any(|x| {
            g.obj(x)
                .chars
                .names()
                .any(|n| counts_as_named(g, id, n, ctx))
        })
}
