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
    // A copy of a spell is owned by the player under whose control it was put on the
    // stack (CR 112.2).
    if copy.kind == ObjKind::SpellCopy {
        copy.owner = controller;
    }
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

/// CR 707.9d: when a copy effect provides specific values for a characteristic (its
/// exceptions set power and toughness, colors, or creature types), the copied object's
/// characteristic-defining abilities that define that characteristic aren't copied, and
/// for color neither is its color indicator. Exceptions that only add types ("in
/// addition to its other types") don't count.
pub fn drop_overridden_cdas(
    v: &mut crate::object::Characteristics,
    exceptions: &[crate::ability::Modification],
) {
    use crate::ability::{AbilityKind, Modification, StaticEffect};
    use crate::keywords::KeywordKind;
    let sets_pt = exceptions
        .iter()
        .any(|m| matches!(m, Modification::SetPT(Some(_), Some(_))));
    let sets_color = exceptions
        .iter()
        .any(|m| matches!(m, Modification::SetColors(_)));
    let sets_creature_types = exceptions.iter().any(|m| {
        matches!(
            m,
            Modification::RemoveAllCreatureTypes | Modification::SetTypes { .. }
        )
    });
    if !(sets_pt || sets_color || sets_creature_types) {
        return;
    }
    if sets_color {
        v.color_indicator = None;
    }
    v.abilities.retain(|a| match &a.kind {
        AbilityKind::Static(s) if s.is_cda => match &s.effect {
            StaticEffect::Continuous { mods, .. } => !mods.iter().any(|m| match m {
                Modification::CdaPT(..) => sets_pt,
                Modification::SetColors(_) => sets_color,
                Modification::AllCreatureTypes => sets_creature_types,
                _ => false,
            }),
            _ => true,
        },
        // Changeling is a characteristic-defining ability (CR 702.73a).
        AbilityKind::Keyword(k) => !(sets_creature_types && k.kind == KeywordKind::Changeling),
        _ => true,
    });
}
