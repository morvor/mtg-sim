//! Delayed triggered abilities about a batch of simultaneous events (CR 603.2c, 603.7):
//! "Until your next turn, whenever either of those creatures deals combat damage, you draw
//! a card." triggers once for each of those creatures that deals combat damage, however
//! many objects and players it deals damage to at once (Tamiyo, Field Researcher's
//! rulings). Printed "one or more" abilities are detected in `triggers.rs`; this is the
//! same grouping for delayed triggered abilities ([`TriggerCond::Batched`] inside their
//! duration).

use crate::ability::*;
use crate::events::Event;
use crate::game::{Game, PendingTrigger};
use crate::object::EventInfo;
use crate::types::*;

/// The batched condition of a delayed trigger, under its duration.
fn batched(t: &TriggerCond) -> Option<(&TriggerCond, BatchPer)> {
    match t {
        TriggerCond::ThisTurn(i)
        | TriggerCond::UntilYourNextTurn(i)
        | TriggerCond::ThisCombat(i) => batched(i),
        TriggerCond::Batched { trigger, per } => Some((trigger, *per)),
        _ => None,
    }
}

/// The delayed triggered abilities a batch of simultaneous events triggers. Those that
/// trigger only once are removed (CR 603.7b).
pub fn triggered_by_batch(g: &mut Game, batch: &[Event]) -> Vec<PendingTrigger> {
    let mut out = Vec::new();
    for d in g.delayed_triggers.clone() {
        let Some((trigger, per)) = batched(&d.trigger) else {
            continue;
        };
        let mut base = d.ctx.clone();
        base.source = d.source;
        base.controller = d.performer.unwrap_or(d.controller);
        let infos: Vec<EventInfo> = batch
            .iter()
            .flat_map(|ev| g.trigger_matches_ctx(trigger, &base, ev))
            .collect();
        if infos.is_empty() {
            continue;
        }
        let key = |i: &EventInfo| match per {
            BatchPer::Batch => None,
            BatchPer::Player | BatchPer::AnyPlayer => i.player.map(Entity::Player),
            BatchPer::Recipient => i
                .object
                .map(Entity::Object)
                .or(i.player.map(Entity::Player)),
            BatchPer::Object => i.object.map(Entity::Object),
            BatchPer::Other => i.other.map(Entity::Object),
        };
        let mut groups: Vec<Vec<EventInfo>> = Vec::new();
        for info in infos {
            match groups.iter().position(|x| key(&x[0]) == key(&info)) {
                Some(i) => groups[i].push(info),
                None => groups.push(vec![info]),
            }
        }
        if per == BatchPer::AnyPlayer {
            groups.truncate(1);
        }
        if d.once {
            groups.truncate(1);
            g.delayed_triggers.retain(|x| x.id != d.id);
        }
        for grp in groups {
            let mut info = grp[0].clone();
            info.amount = grp.iter().map(|i| i.amount).sum();
            if grp.iter().all(|i| i.amount == 0) {
                info.amount = grp.len() as i32;
            }
            info.objects = Vec::new();
            for i in &grp {
                for o in i.object.or(i.other).into_iter() {
                    if !info.objects.contains(&o) {
                        info.objects.push(o);
                    }
                }
            }
            g.trigger_order += 1;
            out.push(g.delayed_pending(&d, info));
        }
    }
    out
}
