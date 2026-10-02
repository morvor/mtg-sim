//! Copying spells (CR 707.10).

use crate::decision::{Answer, Decision};
use crate::game::Game;
use crate::object::*;
use crate::types::*;

/// `StaticEffect::Custom` name of "This spell can't be copied." (it functions on the
/// stack, CR 113.6g).
pub const CANT_BE_COPIED: &str = "can't be copied";
/// `StaticEffect::Custom` name of "This ability can't be copied.", linked (CR 607) to the
/// activated or triggered ability it's about (they share a nonzero `link`).
pub const ABILITY_CANT_BE_COPIED: &str = "ability can't be copied";

/// Whether the spell or ability `id` on the stack can't be copied: a spell with "This
/// spell can't be copied.", or an ability whose source had "This ability can't be
/// copied." for it (as it last existed, CR 113.7a).
pub fn cant_be_copied(g: &Game, id: ObjectId) -> bool {
    let has = |c: &Characteristics, name: &str, link: Option<u16>| {
        c.abilities.iter().any(|a| {
            link.is_none_or(|l| a.link == l)
                && matches!(&a.kind, crate::ability::AbilityKind::Static(s)
                    if matches!(&s.effect, crate::ability::StaticEffect::Custom(n) if n == name))
        })
    };
    let o = g.obj(id);
    match o.stack.as_deref() {
        Some(StackInfo {
            kind: StackKind::Activated { ability, .. } | StackKind::Triggered { ability, .. },
            source_lki,
            ..
        }) => {
            ability.link != 0
                && source_lki
                    .as_deref()
                    .is_some_and(|c| has(c, ABILITY_CANT_BE_COPIED, Some(ability.link)))
        }
        _ => has(&o.chars, CANT_BE_COPIED, None),
    }
}

/// Puts a copy of a spell (or ability) on the stack under `controller`'s control,
/// optionally letting them choose new targets (CR 707.10c). Returns the copy.
pub fn copy_spell(
    g: &mut Game,
    spell: ObjectId,
    controller: PlayerId,
    new_targets: bool,
) -> Option<ObjectId> {
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
    if cant_be_copied(g, spell) {
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
    if let Some(si) = copy.stack.as_mut() {
        // A copy isn't cast (CR 707.10), so no mana was spent to cast it (it copies
        // decisions such as X and additional costs, not the payment).
        si.cast.was_cast = false;
        si.cast.mana_spent.clear();
        si.cast.mana_spent_snow = 0;
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
    Some(id)
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
    let targets = g.resolve_objects(what, ctx);
    // A creature that left the battlefield ("becomes a copy of that creature" after it
    // died) is copied as it last existed there (CR 608.2h), like a token copy of it.
    let Some(src) = crate::copy_rules::token_copy_sources(g, of, ctx)
        .into_iter()
        .next()
    else {
        return;
    };
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
