//! Players and objects an ability notes for the abilities linked to it (CR 607.1,
//! 607.2e): what the first ability affected, which the second refers to ("When ~ enters,
//! target player loses 6 life. When ~ leaves the battlefield, that player gains 6 life.",
//! Laquatus's Champion), or what a player chose ("Pay 1 life: Choose a creature card exiled
//! with ~. ~ has all activated and triggered abilities of the last chosen card.", Koh, the
//! Face Stealer).
//!
//! * [`Effect::NoteLinked`] records the selected players and objects for the source and
//!   the link id of the resolving ability, as it resolves: an ability that never resolves
//!   (countered, or all its targets illegal) affected no one (CR 607.1).
//! * [`PlayerRef::LinkedNoted`] and [`Sel::LinkedNoted`] read them for the source and link
//!   of the ability being evaluated. The players of a linked noting ability that is still
//!   on the stack are its current targets ("the player who gains 6 life is the player who
//!   was the target of the first ability (or, if that ability is still on the stack, the
//!   player who is its target)").
//! * The notes belong to the object (CR 400.7): a permanent that leaves the battlefield and
//!   returns is a new object with no notes. A noted object that has since changed zones is
//!   a new object the note doesn't find.

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::object::StackKind;
use crate::types::*;
use std::collections::BTreeMap;

/// The notes, by source object and link id.
#[derive(Clone, Debug, Default)]
pub struct LinkedNotes {
    notes: BTreeMap<(ObjectId, u16), Vec<Entity>>,
}

/// [`Effect::NoteLinked`]: notes what `what` selects for the abilities linked to the
/// resolving one.
pub fn exec(g: &mut Game, what: &Sel, replace: bool, ctx: &mut Ctx) {
    let Some(src) = ctx.source else {
        return;
    };
    let found = g.resolve_sel(what, ctx);
    let notes = g.linked_notes.notes.entry((src, ctx.link)).or_default();
    if replace {
        notes.clear();
    }
    for e in found {
        if !notes.contains(&e) {
            notes.push(e);
        }
    }
}

/// The notes made for `old` (an object about to enter the battlefield, as replacement
/// effects modify how it enters, CR 614.12a) become the notes of `new`, the permanent.
pub fn carry(g: &mut Game, old: ObjectId, new: ObjectId) {
    let keys: Vec<(ObjectId, u16)> = g
        .linked_notes
        .notes
        .keys()
        .filter(|(o, _)| *o == old)
        .copied()
        .collect();
    for k in keys {
        if let Some(v) = g.linked_notes.notes.remove(&k) {
            g.linked_notes.notes.insert((new, k.1), v);
        }
    }
}

/// The selections of the [`Effect::NoteLinked`] instructions of an effect that refer only
/// to its targets (and so are known while it waits on the stack).
fn target_notes(e: &Effect, out: &mut Vec<Sel>) {
    match e {
        Effect::NoteLinked { what, .. } if refers_to_targets(what) => out.push(what.clone()),
        Effect::Seq(v) => v.iter().for_each(|x| target_notes(x, out)),
        _ => {}
    }
}

fn refers_to_targets(s: &Sel) -> bool {
    match s {
        Sel::Target(_) | Sel::AllTargets => true,
        Sel::Players(PlayerRef::Target(_)) => true,
        Sel::Union(v) => v.iter().all(refers_to_targets),
        _ => false,
    }
}

/// What the linked abilities of the ability being evaluated (`ctx`'s source and link)
/// noted, and the targets of such abilities still on the stack.
pub fn noted(g: &Game, ctx: &Ctx) -> Vec<Entity> {
    let Some(src) = ctx.source else {
        return vec![];
    };
    let mut out = g
        .linked_notes
        .notes
        .get(&(src, ctx.link))
        .cloned()
        .unwrap_or_default();
    for &id in &g.stack {
        if Some(id) == ctx.stack_obj {
            continue;
        }
        let Some(si) = g.obj(id).stack.as_deref() else {
            continue;
        };
        let (source, ability) = match &si.kind {
            StackKind::Activated { source, ability } | StackKind::Triggered { source, ability } => {
                (*source, ability)
            }
            StackKind::Spell => continue,
        };
        if source != src || ability.link != ctx.link {
            continue;
        }
        let body = match &ability.kind {
            AbilityKind::Activated(a) => &a.body,
            AbilityKind::Triggered(t) => &t.body,
            _ => continue,
        };
        let mut sels = Vec::new();
        target_notes(&body.effect, &mut sels);
        if sels.is_empty() || body.modal.is_some() {
            continue;
        }
        let mut c = g.stack_ctx(id);
        c.targets = si
            .chosen
            .first()
            .map(|cm| cm.targets.clone())
            .unwrap_or_default();
        for s in sels {
            for e in g.eval_sel(&s, &c) {
                if !out.contains(&e) {
                    out.push(e);
                }
            }
        }
    }
    out
}

/// [`PlayerRef::LinkedNoted`]: the noted players.
pub fn players(g: &Game, ctx: &Ctx) -> Vec<PlayerId> {
    noted(g, ctx)
        .into_iter()
        .filter_map(|e| match e {
            Entity::Player(p) => Some(p),
            Entity::Object(_) => None,
        })
        .collect()
}

/// [`Sel::LinkedNoted`]: the noted players, and the noted objects that are still the same
/// objects (CR 400.7).
pub fn entities(g: &Game, ctx: &Ctx) -> Vec<Entity> {
    noted(g, ctx)
        .into_iter()
        .filter(|e| match e {
            Entity::Object(o) => g.obj(*o).next.is_none(),
            Entity::Player(_) => true,
        })
        .collect()
}
