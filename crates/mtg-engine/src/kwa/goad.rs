//! CR 701.15: goad, and ending it ("[creatures] are no longer goaded").
//!
//! Goading a creature makes it goaded by the controller of the spell or ability until
//! that player's next turn (CR 701.15a; `GameObject::goaded_by`, cleared as that turn
//! begins). The same player goading it again has no effect (CR 701.15d); several players
//! can goad it (CR 701.15c). Goad with another duration ("for the rest of the game") is a
//! [`Restriction::Goaded`] effect locked onto the creature (see
//! `oracle/patterns/goad_durations.rs`). Ending goad removes both kinds of designation
//! (a static ability's "is goaded" effect keeps applying).

use super::*;

/// Goads `obj` for `by` until `by`'s next turn (CR 701.15a).
pub fn goad(g: &mut Game, obj: ObjectId, by: PlayerId) {
    if !g.is_live(obj) || g.obj(obj).zone != Zone::Battlefield || !g.obj(obj).is_creature() {
        return;
    }
    let o = &mut g.objects[obj.0 as usize];
    if !o.goaded_by.contains(&by) {
        o.goaded_by.push(by);
        g.log(|g| format!("{by} goads {}", g.describe(obj)));
    }
}

/// `obj` is no longer goaded: by goad actions, nor by effects of resolved spells and
/// abilities that goaded it for a duration.
pub fn ungoad(g: &mut Game, obj: ObjectId) {
    g.objects[obj.0 as usize].goaded_by.clear();
    for e in g.rule_effects.iter_mut() {
        if let (Restriction::Goaded(_), Some(objs)) = (&e.restriction, e.objects.as_mut()) {
            objs.retain(|o| *o != obj);
        }
    }
    g.rule_effects.retain(|e| {
        !(matches!(e.restriction, Restriction::Goaded(_))
            && e.objects.as_ref().is_some_and(|v| v.is_empty()))
    });
    g.dirty = true;
}

pub struct Goad;

impl KeywordActionRules for Goad {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Goad]
    }

    /// "Goad [creatures]", or (`undo`) "[creatures] are no longer goaded".
    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let objs = g.resolve_objects(a.what, ctx);
        let undo = a.spec.is_some_and(|s| s.undo);
        for o in objs {
            if undo {
                ungoad(g, o);
            } else {
                goad(g, o, ctx.controller);
            }
        }
    }
}

inventory::submit! { KeywordActionRegistration(&Goad) }
