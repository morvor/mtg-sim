//! CR 701.4 Behold. "Behold a [quality]" means "Reveal a [quality] card from your hand or
//! choose a [quality] permanent you control on the battlefield" (CR 701.4a).
//!
//! Beholding appears as an effect (`Effect::KeywordAction` with
//! [`KeywordAction::Behold`], the quality as `Sel::All(filter)` and the number as `n`) and,
//! wrapped in `CostPart::Effect`, as a cost. Whether "a [quality] was beheld" is decided by
//! the object's quality when the player beheld it (CR 701.4b): a behold cost that was paid
//! is recorded by name in the spell's `CastInfo::paid` ([`BEHOLD`]).

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::object::Zone;
use crate::types::*;

/// The name recorded in `CastInfo::paid` when a behold cost was paid.
pub const BEHOLD: &str = "behold";

/// Objects `p` could behold: [quality] cards in their hand and [quality] permanents they
/// control (CR 701.4a).
pub fn candidates(g: &Game, p: PlayerId, quality: &Filter, ctx: &Ctx) -> Vec<ObjectId> {
    let mut out: Vec<ObjectId> = g
        .player(p)
        .hand
        .iter()
        .copied()
        .filter(|c| g.matches(*c, quality, ctx))
        .collect();
    out.extend(
        g.permanents()
            .filter(|o| o.controller == p && !o.phased_out)
            .map(|o| o.id)
            .filter(|o| g.matches(*o, quality, ctx)),
    );
    out
}

/// The quality filter of a behold effect.
pub fn quality(what: &Sel) -> Filter {
    match what {
        Sel::All(f) => f.clone(),
        _ => Filter::Any,
    }
}

/// Whether `p` can behold `n` objects with the quality.
pub fn can_behold(g: &Game, p: PlayerId, quality: &Filter, n: u32, ctx: &Ctx) -> bool {
    candidates(g, p, quality, ctx).len() as u32 >= n
}

/// `p` beholds `n` objects with the quality: reveals cards from their hand and/or chooses
/// permanents they control (CR 701.4a). Returns what was beheld (nothing if they can't
/// behold that many).
pub fn behold(g: &mut Game, p: PlayerId, quality: &Filter, n: u32, ctx: &mut Ctx) -> Vec<ObjectId> {
    let cands = candidates(g, p, quality, ctx);
    if (cands.len() as u32) < n || n == 0 {
        ctx.prev_happened = false;
        return vec![];
    }
    let chosen = g.ask_objects(p, ctx.source, "Behold", cands, n, n);
    for c in &chosen {
        if g.obj(*c).zone == Zone::Hand(p) {
            crate::reveal::reveal(g, p, &[*c], ctx.source);
        }
    }
    g.log(|g| {
        let names: Vec<String> = chosen.iter().map(|c| g.describe(*c)).collect();
        format!("{p} beholds {}", names.join(", "))
    });
    let es: Vec<Entity> = chosen.iter().map(|o| Entity::Object(*o)).collect();
    ctx.prev_happened = !chosen.is_empty();
    ctx.prev_affected = es.clone();
    ctx.set_var(vars::IT, es);
    chosen
}
