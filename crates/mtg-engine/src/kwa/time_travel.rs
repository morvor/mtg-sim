//! CR 701.56: time travel.
//!
//! To time travel means to choose any number of permanents you control with one or more
//! time counters on them and/or suspended cards you own in exile with one or more time
//! counters on them and, for each of those objects, put a time counter on it or remove a
//! time counter from it (CR 701.56a). The player decides for each object, then the
//! changes happen.

use super::*;
use crate::keywords::KeywordKind;
use crate::types::counters;

/// `Event::Custom` name reported when a player time travels.
pub const TIME_TRAVELED: &str = "time travel";

/// A suspended card (CR 702.62b): in exile, with suspend, with a time counter on it.
fn suspended(g: &Game, id: ObjectId) -> bool {
    let o = g.obj(id);
    o.zone == Zone::Exile
        && o.chars.has_keyword(KeywordKind::Suspend)
        && o.counter(counters::TIME) > 0
}

/// The objects `p` may choose when time traveling.
pub fn candidates(g: &Game, p: PlayerId) -> Vec<ObjectId> {
    let mut out: Vec<ObjectId> = g
        .permanents()
        .filter(|o| o.controller == p && o.counter(counters::TIME) > 0)
        .map(|o| o.id)
        .collect();
    out.extend(
        g.exile
            .iter()
            .copied()
            .filter(|id| g.obj(*id).owner == p && suspended(g, *id)),
    );
    out
}

/// `p` time travels (CR 701.56a).
pub fn time_travel(g: &mut Game, p: PlayerId, source: Option<ObjectId>) {
    if g.dirty {
        g.recompute();
    }
    let mut add = Vec::new();
    let mut remove = Vec::new();
    for id in candidates(g, p) {
        let prompt = format!("Time travel: {}", g.describe(id));
        match g.ask_option(
            p,
            source,
            &prompt,
            vec![
                "Put a time counter on it".into(),
                "Remove a time counter from it".into(),
                "Neither".into(),
            ],
        ) {
            0 => add.push(id),
            1 => remove.push(id),
            _ => {}
        }
    }
    for id in add {
        g.add_counters(Entity::Object(id), counters::TIME, 1, source);
    }
    for id in remove {
        g.remove_counters_by(Entity::Object(id), counters::TIME, 1, Some(p));
    }
    emit(g, TIME_TRAVELED, p, None, 0);
}

pub struct TimeTravel;

impl KeywordActionRules for TimeTravel {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::TimeTravel]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let times = number(g, a.n, ctx);
        for p in g.eval_players(a.who, ctx) {
            for _ in 0..times {
                time_travel(g, p, ctx.source);
            }
        }
    }
}

inventory::submit! { KeywordActionRegistration(&TimeTravel) }
