//! One-shot effects that last "until" an event (CR 610.3, 610.4): "exile target creature
//! until this creature leaves the battlefield", "target creature phases out until this
//! leaves the battlefield".
//!
//! The initial one-shot effect exiles (or phases out) the objects and records what's
//! waiting. Immediately after the specified event, a second one-shot effect returns them
//! (it isn't a triggered ability and doesn't use the stack). Returns created by the same
//! events are performed simultaneously (CR 610.3d, 610.4d).

use crate::ability::*;
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::Game;
use crate::object::*;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::*;

/// What an "until" effect will undo.
#[derive(Clone, Debug)]
pub enum UntilKind {
    /// Exiled objects (their ids in exile) and the zones they came from.
    Exiled(Vec<(ObjectId, Zone)>),
    /// Permanents phased out this way.
    PhasedOut(Vec<ObjectId>),
}

/// A pending "until" effect.
#[derive(Clone, Debug)]
pub struct UntilEffect {
    pub id: u32,
    /// The object whose leaving the battlefield ends the effect.
    pub source: ObjectId,
    pub controller: PlayerId,
    pub until: UntilEvent,
    pub kind: UntilKind,
}

/// Whether the event an "until" effect waits for has happened (or had already happened
/// when the effect would start, CR 610.3a–b, 610.4b–c).
fn ended(g: &Game, source: Option<ObjectId>, until: &UntilEvent) -> bool {
    match until {
        UntilEvent::SourceLeavesBattlefield => source.is_none_or(|s| {
            let o = g.obj(s);
            !g.is_live(s) || o.zone != Zone::Battlefield
        }),
    }
}

/// "Exile [objects] until [event]" (CR 610.3). If the event already happened after the
/// spell or ability was put on the stack (or triggered), nothing is exiled
/// (CR 610.3a–b).
pub fn exec_exile_until(g: &mut Game, objs: Vec<ObjectId>, until: &UntilEvent, ctx: &mut Ctx) {
    if ended(g, ctx.source, until) {
        ctx.prev_affected.clear();
        return;
    }
    let moves: Vec<MoveEv> = objs
        .iter()
        .filter(|o| g.is_live(**o))
        .map(|o| MoveEv {
            obj: *o,
            to: Zone::Exile,
            pos: LibraryPosition::Top,
            cause: MoveCause::Exile,
            by: Some(ctx.controller),
            etb: EtbInfo::default(),
            source: ctx.source,
        })
        .collect();
    let from: Vec<Zone> = moves.iter().map(|m| g.obj(m.obj).zone).collect();
    let prev_link = g.current_link;
    g.current_link = ctx.link;
    let res = g.move_objects(moves);
    g.current_link = prev_link;
    let mut exiled = Vec::new();
    for (new, zone) in res.into_iter().zip(from) {
        if let Some(n) = new {
            if g.obj(n).zone == Zone::Exile {
                exiled.push((n, zone));
            }
        }
    }
    ctx.prev_affected = exiled.iter().map(|(o, _)| Entity::Object(*o)).collect();
    ctx.set_var(vars::IT, ctx.prev_affected.clone());
    if exiled.is_empty() {
        return;
    }
    let id = g.new_effect_id();
    g.untils.push(UntilEffect {
        id,
        source: ctx.source.unwrap_or(ObjectId(0)),
        controller: ctx.controller,
        until: until.clone(),
        kind: UntilKind::Exiled(exiled),
    });
}

/// "[Permanents] phase out until [event]" (CR 610.4).
pub fn exec_phase_out_until(g: &mut Game, objs: Vec<ObjectId>, until: &UntilEvent, ctx: &mut Ctx) {
    if ended(g, ctx.source, until) {
        return;
    }
    let objs: Vec<ObjectId> = objs
        .into_iter()
        .filter(|o| g.is_live(*o) && g.obj(*o).zone == Zone::Battlefield && !g.obj(*o).phased_out)
        .collect();
    if objs.is_empty() {
        return;
    }
    crate::keyword_impls::phase_out(g, objs.clone());
    let id = g.new_effect_id();
    g.untils.push(UntilEffect {
        id,
        source: ctx.source.unwrap_or(ObjectId(0)),
        controller: ctx.controller,
        until: until.clone(),
        kind: UntilKind::PhasedOut(objs),
    });
}

/// A permanent phased out by an "until" effect doesn't phase in during its controller's
/// untap step (CR 610.4a).
pub fn held_phased_out(g: &Game, id: ObjectId) -> bool {
    g.untils
        .iter()
        .any(|u| matches!(&u.kind, UntilKind::PhasedOut(v) if v.contains(&id)))
}

/// A permanent that phases in because of another effect is no longer waiting: the second
/// one-shot effect won't happen, even if it phases out again (CR 610.4a).
pub fn phased_in_otherwise(g: &mut Game, id: ObjectId) {
    for u in g.untils.iter_mut() {
        if let UntilKind::PhasedOut(v) = &mut u.kind {
            v.retain(|o| *o != id);
        }
    }
    g.untils
        .retain(|u| !matches!(&u.kind, UntilKind::PhasedOut(v) if v.is_empty()));
}

/// Performs the second one-shot effects of every "until" effect whose event has
/// happened. All returns are performed simultaneously (CR 610.3d, 610.4d). Returned
/// objects come back under their owners' control (CR 610.3c).
pub fn check_untils(g: &mut Game) {
    if g.untils.is_empty() {
        return;
    }
    let (done, keep): (Vec<UntilEffect>, Vec<UntilEffect>) = std::mem::take(&mut g.untils)
        .into_iter()
        .partition(|u| ended(g, Some(u.source), &u.until));
    g.untils = keep;
    if done.is_empty() {
        return;
    }
    let mut moves: Vec<MoveEv> = Vec::new();
    let mut phase_in: Vec<ObjectId> = Vec::new();
    for u in done {
        match u.kind {
            UntilKind::Exiled(v) => {
                for (o, zone) in v {
                    // The card must still be the same object in exile.
                    if !g.is_in_zone(o, Zone::Exile) {
                        continue;
                    }
                    let owner = g.obj(o).owner;
                    let to = match zone {
                        Zone::Battlefield => Zone::Battlefield,
                        Zone::Hand(_) => Zone::Hand(owner),
                        Zone::Graveyard(_) => Zone::Graveyard(owner),
                        Zone::Library(_) => Zone::Library(owner),
                        other => other,
                    };
                    moves.push(MoveEv {
                        obj: o,
                        to,
                        pos: LibraryPosition::Top,
                        cause: MoveCause::Return,
                        by: Some(owner),
                        etb: EtbInfo {
                            controller: if to == Zone::Battlefield {
                                Some(owner)
                            } else {
                                None
                            },
                            ..Default::default()
                        },
                        source: None,
                    });
                }
            }
            UntilKind::PhasedOut(v) => {
                phase_in.extend(
                    v.into_iter()
                        .filter(|o| g.is_live(*o) && g.obj(*o).phased_out),
                );
            }
        }
    }
    for o in phase_in {
        crate::keyword_impls::phase_in(g, o);
    }
    if !moves.is_empty() {
        g.move_objects(moves);
    }
}
