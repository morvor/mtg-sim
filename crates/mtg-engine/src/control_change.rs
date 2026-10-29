//! Control-changing effects of resolving spells and abilities (CR 108.4, 613.1b):
//! [`Effect::GainControl`] and [`Effect::ExchangeControl`] (CR 701.12).
//!
//! A control change is planned first (the objects determined and the choices made) and
//! applied afterwards, so that "each player gains control of ..." can make every
//! player's choices before any control changes and change control of all of them at the
//! same time (CR 101.4, 608.2f; see `simultaneous.rs`).
//!
//! An exchange of control (CR 701.12) is between two sides: two permanents, or a spell and
//! a permanent ("exchange control of ~ and that spell"), or two groups of permanents each
//! controlled by one player ("those players exchange control of those creatures"). If
//! the whole exchange can't be done — an object is gone, the groups differ in size or one
//! side has several controllers — none of it is (CR 701.12a); if both sides have the same
//! controller it does nothing (CR 701.12b). Otherwise each side's controller gains
//! control of the other side at the same time. A spell whose control changes stays on
//! the stack; if it becomes a permanent, its new controller controls that permanent
//! (CR 608.3).

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::{Affected, ContinuousEffect, Game};
use crate::object::Zone;
use crate::types::*;

/// A planned control change: who gains control of what, for how long.
#[derive(Clone, Debug)]
pub struct Grant {
    pub objs: Vec<ObjectId>,
    pub to: PlayerId,
    pub duration: Duration,
    /// How many of them `to` didn't control yet ("for each one they gained control of
    /// this way").
    pub changed: usize,
}

/// Determines what [`Effect::GainControl`] would do (making its choices), without
/// changing control. `None` if it does nothing.
pub fn plan_gain_control(
    g: &mut Game,
    what: &Sel,
    who: &PlayerRef,
    duration: &Duration,
    ctx: &mut Ctx,
) -> Option<Grant> {
    // CR 611.2b (Master Thief). A condition about the affected objects ("for as long as
    // that creature is enchanted") is checked for them below.
    let about_affected = matches!(duration, Duration::WhileCondition(_));
    if !about_affected && g.effect_expired(duration, ctx.source, ctx.controller) {
        return None;
    }
    let objs: Vec<ObjectId> = g
        .resolve_objects(what, ctx)
        .into_iter()
        .filter(|o| g.is_live(*o))
        .collect();
    let p = g.eval_player(who, ctx)?;
    if objs.is_empty() {
        return None;
    }
    if let Duration::WhileCondition(c) = duration {
        let mut c2 = ctx.clone();
        c2.set_var(
            vars::AFFECTED,
            objs.iter().map(|o| Entity::Object(*o)).collect(),
        );
        if !g.eval_cond(c, &c2) {
            return None;
        }
    }
    let changed = objs.iter().filter(|o| g.obj(**o).controller != p).count();
    Some(Grant {
        objs,
        to: p,
        // "for as long as they're the monarch": the player as the effect begins.
        duration: {
            let d = bind_duration_players(g, duration, ctx);
            crate::duration_refs::bind_duration_objects(g, &d, ctx)
        },
        changed,
    })
}

/// A "for as long as [player condition]" duration with the players it names fixed as the
/// effect begins ("for as long as they're the monarch": that player, CR 611.2b).
fn bind_duration_players(g: &Game, d: &Duration, ctx: &Ctx) -> Duration {
    let Duration::WhileCondition(Condition::PlayerMatches(r, f)) = d else {
        return d.clone();
    };
    if !matches!(
        r,
        PlayerRef::TriggerPlayer | PlayerRef::Target(_) | PlayerRef::Iterated
    ) {
        return d.clone();
    }
    match g.eval_player(r, ctx) {
        Some(p) => {
            Duration::WhileCondition(Condition::PlayerMatches(PlayerRef::Player(p), f.clone()))
        }
        None => d.clone(),
    }
}

/// Creates the control-changing effect of a planned change (CR 613.1b). The caller
/// recomputes characteristics.
pub fn apply(g: &mut Game, grant: &Grant, ctx: &Ctx) {
    let objs: Vec<ObjectId> = grant
        .objs
        .iter()
        .copied()
        .filter(|o| g.is_live(*o))
        .collect();
    if objs.is_empty() {
        return;
    }
    let id = g.new_effect_id();
    let ts = g.new_timestamp();
    g.effects.push(ContinuousEffect {
        id,
        source: ctx.source,
        controller: ctx.controller,
        timestamp: ts,
        duration: grant.duration.clone(),
        affected: Affected::Objects(objs),
        mods: vec![Modification::SetController(PlayerRef::Player(grant.to))],
        layer1: None,
        created_turn: g.turn.number,
    });
    g.dirty = true;
}

/// Records a performed control change for "if they do" and "for each one they gained
/// control of this way".
pub fn record(grant: Option<&Grant>, ctx: &mut Ctx) {
    let changed = grant.map_or(0, |g| g.changed);
    ctx.prev_happened = changed > 0;
    if grant.is_some() {
        ctx.prev_value = changed as i64;
    }
}

/// [`Effect::GainControl`].
pub fn gain_control(g: &mut Game, what: &Sel, who: &PlayerRef, duration: &Duration, ctx: &mut Ctx) {
    // "If they do, ...": whether a player gained control of something.
    ctx.prev_happened = false;
    let Some(grant) = plan_gain_control(g, what, who, duration, ctx) else {
        return;
    };
    record(Some(&grant), ctx);
    apply(g, &grant, ctx);
    g.recompute();
}

/// Whether an object can have its control exchanged: a permanent or a spell.
fn exchangeable(g: &Game, o: ObjectId) -> bool {
    g.is_live(o) && matches!(g.obj(o).zone, Zone::Battlefield | Zone::Stack)
}

fn distinct(v: Vec<ObjectId>) -> Vec<ObjectId> {
    let mut out: Vec<ObjectId> = Vec::new();
    for o in v {
        if !out.contains(&o) {
            out.push(o);
        }
    }
    out
}

/// The one player controlling all of `objs`, if there is one.
fn sole_controller(g: &Game, objs: &[ObjectId]) -> Option<PlayerId> {
    let c = g.obj(*objs.first()?).controller;
    objs.iter().all(|o| g.obj(*o).controller == c).then_some(c)
}

/// [`Effect::ExchangeControl`] (CR 701.12a, 701.12b). Two equal selections name two
/// objects chosen together ("two target creatures controlled by different players").
pub fn exchange(g: &mut Game, a: &Sel, b: &Sel, ctx: &mut Ctx) {
    // Whether it happened is what "If you do" / "If you don't or can't make an exchange"
    // ask about.
    ctx.prev_happened = false;
    let same = serde_json::to_string(a).ok() == serde_json::to_string(b).ok();
    let (xs, ys) = if same {
        let both = distinct(g.resolve_objects(a, ctx));
        if both.len() != 2 {
            return;
        }
        (vec![both[0]], vec![both[1]])
    } else {
        let xs = distinct(g.resolve_objects(a, ctx));
        let ys: Vec<ObjectId> = distinct(g.resolve_objects(b, ctx))
            .into_iter()
            .filter(|o| !xs.contains(o))
            .collect();
        (xs, ys)
    };
    if xs.is_empty() || xs.len() != ys.len() {
        return;
    }
    if !xs.iter().chain(&ys).all(|o| exchangeable(g, *o)) {
        return;
    }
    let (Some(ca), Some(cb)) = (sole_controller(g, &xs), sole_controller(g, &ys)) else {
        return;
    };
    if ca == cb {
        return;
    }
    for (objs, to) in [(xs, cb), (ys, ca)] {
        let grant = Grant {
            changed: objs.len(),
            objs,
            to,
            duration: Duration::Permanent,
        };
        apply(g, &grant, ctx);
    }
    g.recompute();
    ctx.prev_happened = true;
}
