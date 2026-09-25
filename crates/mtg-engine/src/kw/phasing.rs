//! CR 702.26 Phasing: the phasing event in the untap step, phasing out (directly and
//! indirectly) and in, and phased-out permanents whose controller left the game.
//!
//! While phased out, a permanent stays on the battlefield (CR 702.26d) but is treated as
//! though it doesn't exist (CR 702.26b): [`Game::permanents`] skips it, its abilities
//! don't function, and it can't be targeted, attack, or block.

use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// CR 502.1, 702.26a: during the active player's untap step, before they untap, their
/// phased-in permanents with phasing phase out and, simultaneously, the permanents that
/// phased out under their control phase in (those that phased out indirectly phase in
/// with the permanent they're attached to, CR 702.26g). A skipped untap step has no
/// phasing event (CR 702.26m).
pub fn untap_step(g: &mut Game, active: PlayerId) {
    let out: Vec<ObjectId> = g
        .battlefield
        .iter()
        .copied()
        .filter(|id| {
            let o = g.obj(*id);
            !o.phased_out && o.controller == active && o.has_keyword(KeywordKind::Phasing)
        })
        .collect();
    let back: Vec<ObjectId> = g
        .battlefield
        .iter()
        .copied()
        .filter(|id| {
            let o = g.obj(*id);
            o.phased_out
                && o.phased_out_under.unwrap_or(o.controller) == active
                && !o.phased_out_indirectly
                && !crate::until::held_phased_out(g, *id)
        })
        .collect();
    phase_out(g, out);
    for id in back {
        phase_in(g, id);
    }
}

/// Everything attached to the objects, recursively (Auras on Equipment, ...).
fn attached_closure(g: &Game, objs: &[ObjectId]) -> Vec<ObjectId> {
    let mut out: Vec<ObjectId> = Vec::new();
    let mut todo: Vec<ObjectId> = objs.to_vec();
    while let Some(o) = todo.pop() {
        for a in g.attachments_of(Entity::Object(o)) {
            if !out.contains(&a) {
                out.push(a);
                todo.push(a);
            }
        }
    }
    out
}

/// Phases permanents out (CR 702.26b), removing them from combat. Auras, Equipment, and
/// Fortifications attached to them phase out at the same time, indirectly (CR 702.26g);
/// one that would phase out both directly and indirectly just phases out indirectly
/// (CR 702.26h).
pub fn phase_out(g: &mut Game, objs: Vec<ObjectId>) {
    let objs: Vec<ObjectId> = objs
        .into_iter()
        .filter(|o| g.is_live(*o) && !g.obj(*o).phased_out)
        .collect();
    let indirect = attached_closure(g, &objs);
    let direct: Vec<ObjectId> = objs
        .iter()
        .copied()
        .filter(|o| !indirect.contains(o))
        .collect();
    let mut phased: Vec<ObjectId> = Vec::new();
    for id in direct.iter().chain(indirect.iter()).copied() {
        if g.obj(id).phased_out {
            continue;
        }
        let under = g.obj(id).controller;
        let o = &mut g.objects[id.0 as usize];
        o.phased_out = true;
        o.phased_out_indirectly = indirect.contains(&id);
        o.phased_out_under = Some(under);
        phased.push(id);
    }
    for id in &phased {
        crate::combat::remove_from_combat(g, *id);
    }
    // "Phases out" abilities of everything that phased out look back in time (CR 603.10b).
    for id in phased {
        g.emit(Event::PhasedOut { obj: id });
    }
    g.dirty = true;
}

/// Phases a permanent in (CR 702.26c) along with everything that phased out indirectly
/// with it. An Aura, Equipment, or Fortification that phased out directly phases in
/// attached to what it was attached to; if that's gone, state-based actions deal with it
/// (CR 702.26i). Phasing in doesn't cause attach or zone-change triggers (CR 702.26d,
/// 702.26j).
pub fn phase_in(g: &mut Game, id: ObjectId) {
    if !g.obj(id).phased_out {
        return;
    }
    crate::until::phased_in_otherwise(g, id);
    let mut todo = vec![id];
    let mut all: Vec<ObjectId> = Vec::new();
    while let Some(o) = todo.pop() {
        all.push(o);
        for a in g.battlefield.clone() {
            let ob = g.obj(a);
            if ob.phased_out
                && ob.phased_out_indirectly
                && ob.attached_to == Some(Entity::Object(o))
                && !all.contains(&a)
            {
                todo.push(a);
            }
        }
    }
    for o in &all {
        let ob = &mut g.objects[o.0 as usize];
        ob.phased_out = false;
        ob.phased_out_indirectly = false;
        ob.phased_out_under = None;
    }
    g.emit(Event::PhasedIn { obj: id });
    g.dirty = true;
}

/// CR 702.26n: a permanent that phased out under the control of a player who has left
/// the game phases in during the next untap step after that player's next turn would
/// have begun. Called as the (non-extra) turn of `next` begins after the turn of `after`:
/// the turns of players who left the game seated between them would have begun, so their
/// phased-out permanents phase in during `next`'s untap step.
pub fn turns_would_have_begun(g: &mut Game, after: PlayerId, next: PlayerId) {
    let n = g.players.len();
    if n == 0 {
        return;
    }
    let mut skipped: Vec<PlayerId> = Vec::new();
    let mut i = (after.idx() + 1) % n;
    while i != next.idx() && i != after.idx() {
        let q = PlayerId(i as u8);
        if !g.player(q).in_game() {
            skipped.push(q);
        }
        i = (i + 1) % n;
    }
    if skipped.is_empty() {
        return;
    }
    for id in g.battlefield.clone() {
        let o = &mut g.objects[id.0 as usize];
        if o.phased_out && o.phased_out_under.is_some_and(|p| skipped.contains(&p)) {
            o.phased_out_under = Some(next);
        }
    }
}
