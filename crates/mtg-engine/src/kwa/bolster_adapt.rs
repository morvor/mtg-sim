//! CR 701.39: bolster, and CR 701.46: adapt.
//!
//! * "Bolster N": choose a creature you control with the least toughness or tied for
//!   least toughness among creatures you control, and put N +1/+1 counters on it
//!   (CR 701.39a). The creature is chosen as the action is performed.
//! * "Adapt N": if this permanent has no +1/+1 counters on it, put N +1/+1 counters on it
//!   (CR 701.46a).

use super::*;
use crate::types::counters;

/// Bolster N for `p` (CR 701.39a). Returns the creature that got the counters.
pub fn bolster(g: &mut Game, p: PlayerId, n: u32, source: Option<ObjectId>) -> Option<ObjectId> {
    if g.dirty {
        g.recompute();
    }
    let mine: Vec<ObjectId> = g
        .permanents()
        .filter(|o| o.controller == p && o.is_creature())
        .map(|o| o.id)
        .collect();
    let least = mine.iter().map(|o| g.obj(*o).toughness()).min()?;
    let tied: Vec<ObjectId> = mine
        .into_iter()
        .filter(|o| g.obj(*o).toughness() == least)
        .collect();
    let pick = g
        .ask_objects(
            p,
            source,
            "Bolster: choose a creature with the least toughness",
            tied.clone(),
            1,
            1,
        )
        .first()
        .copied()
        .unwrap_or(tied[0]);
    g.add_counters(Entity::Object(pick), counters::PLUS1, n, source);
    Some(pick)
}

pub struct Bolster;

impl KeywordActionRules for Bolster {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Bolster]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let n = number(g, a.n, ctx);
        for p in g.eval_players(a.who, ctx) {
            if let Some(c) = bolster(g, p, n, ctx.source) {
                ctx.set_var(vars::IT, vec![Entity::Object(c)]);
            }
        }
    }
}

inventory::submit! { KeywordActionRegistration(&Bolster) }

/// Adapt N for `obj` (CR 701.46a). Returns true if it got counters.
pub fn adapt(g: &mut Game, obj: ObjectId, n: u32, source: Option<ObjectId>) -> bool {
    if !on_battlefield(g, obj) || g.obj(obj).counter(counters::PLUS1) > 0 {
        return false;
    }
    g.add_counters(Entity::Object(obj), counters::PLUS1, n, source) > 0
}

pub struct Adapt;

impl KeywordActionRules for Adapt {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Adapt]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let n = number(g, a.n, ctx);
        let mut any = false;
        for obj in g.resolve_objects(a.what, ctx) {
            any |= adapt(g, obj, n, ctx.source);
        }
        ctx.prev_happened = any;
    }
}

inventory::submit! { KeywordActionRegistration(&Adapt) }
