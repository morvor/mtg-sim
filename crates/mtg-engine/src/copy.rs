//! Copying spells (CR 707.10).

use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::game::Game;
use crate::object::*;
use crate::types::*;

/// Puts a copy of a spell (or ability) on the stack under `controller`'s control,
/// optionally letting them choose new targets (CR 707.10c). Returns the copy.
pub fn copy_spell(
    g: &mut Game,
    spell: ObjectId,
    controller: PlayerId,
    new_targets: bool,
) -> Option<ObjectId> {
    if !g.is_live(spell) || g.obj(spell).zone != Zone::Stack {
        return None;
    }
    let orig = g.obj(spell).clone();
    let mut copy = orig.clone();
    copy.kind = if orig.kind == ObjKind::StackAbility {
        ObjKind::StackAbility
    } else {
        ObjKind::SpellCopy
    };
    copy.controller = controller;
    copy.base_controller = controller;
    copy.prev = None;
    copy.next = None;
    if let Some(si) = copy.stack.as_mut() {
        // A copy isn't cast (CR 707.10).
        si.cast.was_cast = false;
    }
    let id = ObjectId(g.objects.len() as u32);
    copy.id = id;
    copy.timestamp = g.new_timestamp();
    g.objects.push(copy);
    g.stack.push(id);
    if let Some(ctx) = g.saved_ctx.get(&spell).cloned() {
        g.saved_ctx.insert(id, ctx);
    }
    g.dirty = true;
    g.recompute();
    if orig.kind != ObjKind::StackAbility {
        g.emit(crate::events::Event::SpellCopied {
            spell: id,
            player: controller,
        });
    }
    if new_targets {
        let body = g.stack_body(id);
        let mut ctx = Ctx::new(Some(id), controller);
        ctx.x = g.obj(id).stack.as_ref().and_then(|s| s.x).unwrap_or(0);
        let keep = g.ask(
            controller,
            Decision::YesNo {
                source: Some(id),
                prompt: "Choose new targets for the copy?".into(),
            },
        );
        if matches!(keep, Answer::Bool(true)) {
            let saved = g.obj(id).stack.as_ref().map(|s| s.chosen.clone());
            if !g.choose_modes_and_targets(id, &body, &mut ctx) {
                if let (Some(s), Some(si)) = (saved, g.objects[id.0 as usize].stack.as_mut()) {
                    si.chosen = s;
                }
            }
        }
    }
    Some(id)
}
