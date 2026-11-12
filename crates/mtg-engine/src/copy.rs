//! Copying spells (CR 707.10).

use crate::decision::{Answer, Decision};
use crate::game::Game;
use crate::object::*;
use crate::types::*;

/// The additional copies of `spell` that `controller` makes as an effect has them copy it
/// one or more times ("If you would copy a spell one or more times, instead copy it that
/// many times plus an additional time. You may choose new targets for the additional
/// copy."): each is put onto the stack with new targets allowed. None for an ability.
pub fn additional_copies(g: &mut Game, spell: ObjectId, controller: PlayerId) -> Vec<ObjectId> {
    if g.obj(spell).kind == ObjKind::StackAbility {
        return vec![];
    }
    let n = crate::kw::additional_spell_copies(g, controller);
    (0..n)
        .filter_map(|_| copy_spell(g, spell, controller, true))
        .collect()
}

/// Puts a copy of a spell (or ability) on the stack under `controller`'s control,
/// optionally letting them choose new targets (CR 707.10c). Returns the copy. Each
/// object or player the copy targets becomes its target: those whose target is changed
/// as the copy's targets are changed, the others as the copy is created (a copy of a
/// spell that targets you, its target unchanged, makes you the target of a spell).
pub fn copy_spell(
    g: &mut Game,
    spell: ObjectId,
    controller: PlayerId,
    new_targets: bool,
) -> Option<ObjectId> {
    let id = copy_spell_quiet(g, spell, controller)?;
    let before = chosen_targets(g, id);
    if new_targets && has_chosen_targets(g, id) {
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
    announce_unchanged_targets(g, id, &before);
    Some(id)
}

/// The targets of a spell or ability on the stack, in order (every mode and slot).
pub fn chosen_targets(g: &Game, id: ObjectId) -> Vec<Entity> {
    g.obj(id)
        .stack
        .as_deref()
        .map(|si| {
            si.chosen
                .iter()
                .flat_map(|m| m.targets.iter().flatten().copied())
                .collect()
        })
        .unwrap_or_default()
}

/// The copy `copy` was put on the stack with the targets `before` (as
/// [`chosen_targets`] lists them); each of them still targeted at the same place (not
/// changed since) becomes a target of the copy now (emits `BecameTarget`, once per
/// object or player). Changed targets were announced as they were changed.
pub fn announce_unchanged_targets(g: &mut Game, copy: ObjectId, before: &[Entity]) {
    let now = chosen_targets(g, copy);
    if now.len() != before.len() {
        return;
    }
    let controller = g.obj(copy).controller;
    let mut seen: Vec<Entity> = Vec::new();
    for (b, n) in before.iter().zip(&now) {
        if b == n && !seen.contains(n) {
            seen.push(*n);
            g.emit(crate::events::Event::BecameTarget {
                target: *n,
                by: copy,
                controller,
            });
        }
    }
}

/// [`copy_spell`] without announcing the copy's targets: for a caller that changes them
/// itself afterwards (and then calls [`announce_unchanged_targets`] or announces them).
pub fn copy_spell_quiet(g: &mut Game, spell: ObjectId, controller: PlayerId) -> Option<ObjectId> {
    // A spell that has left the stack (e.g. countered in response to storm's trigger) is
    // copied as it last existed there (CR 608.2h); its old object keeps that information.
    // So is an ability that has left it (countered in response to Rings of Brighthearth's
    // trigger): its object is kept, out of every zone.
    let o = g.obj(spell);
    let spell_lki = !g.is_live(spell) && o.kind != ObjKind::StackAbility && o.stack.is_some();
    let ability_lki =
        o.kind == ObjKind::StackAbility && o.zone == Zone::Nowhere && o.stack.is_some();
    if !ability_lki && (o.zone != Zone::Stack || !(g.is_live(spell) || spell_lki)) {
        return None;
    }
    // "This spell can't be copied." (CR 113.6g, 707.10).
    if crate::rule_statics::cant_be_copied::cant_be_copied(g, spell) {
        return None;
    }
    let orig = g.obj(spell).clone();
    let mut copy = orig.clone();
    copy.zone = Zone::Stack;
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
    // Being a commander is a property of the card, not a copiable value (CR 903.3): a copy
    // of a commander spell isn't a commander, nor is the token it becomes.
    copy.is_commander = false;
    if let Some(si) = copy.stack.as_mut() {
        // A copy isn't cast (CR 707.10), so no mana was spent to cast it (it copies
        // decisions such as X and additional costs, not the payment).
        si.cast.was_cast = false;
        si.cast.mana_spent.clear();
        si.cast.mana_spent_snow = 0;
        si.cast.mana_sources.clear();
    }
    let id = ObjectId(g.objects.len() as u32);
    copy.id = id;
    copy.timestamp = g.new_timestamp();
    g.objects.push(copy);
    g.stack.push(id);
    if let Some(ctx) = g.saved_ctx.get(&spell).cloned() {
        g.saved_ctx.insert(id, ctx);
    }
    // Cards spliced onto the spell were choices made while casting it (CR 702.47).
    crate::splice::copy_splices(g, spell, id);
    // A copy of a prepare spell is one too (CR 722.3d).
    crate::designations::spell_copied(g, spell, id);
    g.dirty = true;
    g.recompute();
    if orig.kind != ObjKind::StackAbility {
        g.emit(crate::events::Event::SpellCopied {
            spell: id,
            player: controller,
        });
    }
    Some(id)
}

/// Undoes [`copy_spell_quiet`] for a copy that turns out not to be created (CR 707.10e: its
/// specified target isn't a legal target for it): it leaves the stack with everything
/// putting it there set up (keep this in step with `copy_spell_quiet`), and the events
/// emitted since `events_before` (that the spell was copied) are withdrawn before anything
/// could trigger on them. The object stays out of every zone.
pub(crate) fn uncreate_copy(g: &mut Game, copy: ObjectId, events_before: usize) {
    g.stack.retain(|x| *x != copy);
    g.saved_ctx.remove(&copy);
    // Its spliced text (copy_splices).
    g.effects.retain(|ef| ef.source != Some(copy));
    g.special.prepare_spells.retain(|x| *x != copy);
    g.objects[copy.0 as usize].zone = Zone::Nowhere;
    g.events.truncate(events_before);
    g.dirty = true;
    g.recompute();
}

/// Makes `mods` exceptions of the copy effect that created the spell copy `copy`
/// ("except the copy isn't legendary", CR 707.9b): a copy effect on it that copies its
/// own copiable values with those exceptions, lasting for as long as it exists (a copy
/// of a permanent spell keeps its copiable values as it becomes a token, CR 608.3f).
pub fn add_copy_exceptions(
    g: &mut Game,
    copy: ObjectId,
    controller: PlayerId,
    mods: &[crate::ability::Modification],
) {
    if mods.is_empty() {
        return;
    }
    let values = g.obj(copy).copiable.clone();
    let id = g.new_effect_id();
    let timestamp = g.new_timestamp();
    let created_turn = g.turn.number;
    g.effects.push(crate::game::ContinuousEffect {
        id,
        source: Some(copy),
        controller,
        timestamp,
        duration: crate::ability::Duration::Permanent,
        affected: crate::game::Affected::Objects(vec![copy]),
        mods: vec![],
        layer1: Some(crate::game::Layer1::Copy {
            values: Box::new(values),
            exceptions: mods.to_vec(),
        }),
        created_turn,
    });
    g.dirty = true;
    g.recompute();
}

/// CR 405.3: several copies put on the stack at the same time by one effect are put
/// there in the order their controller chooses. `copies` are the new objects, which
/// sit together on top of the stack in creation order; the controller is asked for
/// their relative order (unless they're indistinguishable: copies of the same spell or
/// ability with the same modes and targets).
pub fn order_copies(g: &mut Game, controller: PlayerId, copies: &[ObjectId]) {
    if copies.len() < 2 {
        return;
    }
    let choices = |g: &Game, id: ObjectId| {
        g.obj(id)
            .stack
            .as_deref()
            .map(|si| {
                si.chosen
                    .iter()
                    .map(|m| (m.mode, m.targets.clone(), m.divided.clone()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };
    // Copies of the same spell or ability (by name) with the same choices can't be told
    // apart.
    let key = |g: &Game, id: ObjectId| (g.obj(id).name().to_string(), choices(g, id));
    let first = key(g, copies[0]);
    if copies.iter().all(|c| key(g, *c) == first) {
        return;
    }
    // Their positions on the stack, bottom first.
    let mut slots: Vec<usize> = copies
        .iter()
        .filter_map(|c| g.stack.iter().position(|s| s == c))
        .collect();
    if slots.len() != copies.len() {
        return;
    }
    slots.sort();
    let items = copies
        .iter()
        .map(|c| {
            let targets: Vec<String> = choices(g, *c)
                .iter()
                .flat_map(|(_, t, _)| t.iter().flatten().map(|e| format!("{e:?}")))
                .collect();
            format!("{} targeting [{}]", g.obj(*c).name(), targets.join(", "))
        })
        .collect();
    let Answer::Indices(order) = g.ask(
        controller,
        Decision::Order {
            prompt: "Order the copies (first is put onto the stack first)".into(),
            items,
        },
    ) else {
        return;
    };
    let mut sorted = order.clone();
    sorted.sort();
    if sorted != (0..copies.len()).collect::<Vec<_>>() {
        return;
    }
    for (slot, i) in slots.into_iter().zip(order) {
        g.stack[slot] = copies[i];
    }
}

/// "[objects] become(s) a copy of [object] [for the duration], [except ...]" (CR 707.2,
/// 613.2a): a layer-1 copy effect giving the objects the other object's copiable values
/// with the exceptions (CR 707.9b), which are part of their copiable values then.
pub fn become_copy(
    g: &mut Game,
    what: &crate::ability::Sel,
    of: &crate::ability::Sel,
    duration: &crate::ability::Duration,
    exceptions: &[crate::ability::Modification],
    ctx: &mut crate::eval::Ctx,
) {
    use crate::game::{Affected, ContinuousEffect, Layer1};
    // CR 702.26e: a phased-out permanent isn't among the objects it affects, even one it
    // names specifically ("this creature becomes a copy").
    let targets: Vec<ObjectId> = g
        .resolve_objects(what, ctx)
        .into_iter()
        .filter(|o| !crate::kw::phasing::is_phased_out(g, *o))
        .collect();
    if targets.is_empty() {
        return;
    }
    // A creature that left the battlefield ("becomes a copy of that creature" after it
    // died) is copied as it last existed there (CR 608.2h), like a token copy of it.
    let Some(src) = crate::copy_rules::token_copy_sources(g, of, ctx)
        .into_iter()
        .next()
    else {
        return;
    };
    // CR 611.2b: a "for as long as" duration that already ended means the effect does
    // nothing; the objects its condition names are fixed as it begins.
    if let crate::ability::Duration::WhileCondition(c) = duration {
        let mut c2 = ctx.clone();
        c2.set_var(
            crate::ability::vars::AFFECTED,
            targets
                .iter()
                .map(|o| crate::types::Entity::Object(*o))
                .collect(),
        );
        if !g.eval_cond(c, &c2) {
            return;
        }
    }
    let duration = &crate::duration_refs::bind_duration_objects(g, duration, ctx);
    let values = Box::new(g.obj(src).copiable.clone());
    // "except it has this ability": the resolving ability (CR 707.9a).
    let exceptions = g.fix_mods(exceptions, ctx);
    let id = g.new_effect_id();
    let ts = g.new_timestamp();
    g.effects.push(ContinuousEffect {
        id,
        source: ctx.source,
        controller: ctx.controller,
        timestamp: ts,
        duration: duration.clone(),
        affected: Affected::Objects(targets),
        mods: vec![],
        layer1: Some(Layer1::Copy { values, exceptions }),
        created_turn: g.turn.number,
    });
    g.dirty = true;
}

/// Whether the spell or ability `id` on the stack has any chosen targets. Only those can
/// be changed (CR 707.10c, 115.7), so a copy without targets offers no "choose new
/// targets" choice.
pub fn has_chosen_targets(g: &Game, id: ObjectId) -> bool {
    g.obj(id).stack.as_deref().is_some_and(|si| {
        si.chosen
            .iter()
            .any(|m| m.targets.iter().any(|t| !t.is_empty()))
    })
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
        // Changeling and devoid are characteristic-defining abilities (CR 702.73a,
        // 702.114a).
        AbilityKind::Keyword(k) => match k.kind {
            KeywordKind::Changeling => !sets_creature_types,
            KeywordKind::Devoid => !sets_color,
            _ => true,
        },
        _ => true,
    });
}
