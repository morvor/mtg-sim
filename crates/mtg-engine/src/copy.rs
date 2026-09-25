//! Copying spells (CR 707.10).

use crate::decision::{Answer, Decision};
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
        let keep = g.ask(
            controller,
            Decision::YesNo {
                source: Some(id),
                prompt: "Choose new targets for the copy?".into(),
            },
        );
        if matches!(keep, Answer::Bool(true)) {
            // CR 707.10c, 115.7d: the copy's modes stay; any of its targets may be changed.
            crate::target_rules::change_targets(
                g,
                controller,
                id,
                crate::ability::TargetChange::ChooseNew,
                None,
            );
        }
    }
    Some(id)
}
