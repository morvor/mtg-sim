//! Rules for copies beyond the basic copy effect (CR 707):
//!
//! * exceptions to entering as a copy that are additional effects, conditional, or
//!   linked triggered abilities (CR 707.9e–707.9g), see [`CopyExtra`];
//! * copying a spell for each object or player it could target, or with a specified new
//!   target (CR 707.10d, 707.10e);
//! * token copies of double-faced permanents and cards (CR 707.8a);
//! * copies of cards: in the card's zone, to be cast (CR 707.12); by name, from the
//!   Oracle card reference (CR 707.13); from a noted card's last known information
//!   (CR 707.14).

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::{Game, Layer1};
use crate::object::*;
use crate::types::*;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// A part of a copy exception that isn't a plain modification of the copiable values:
/// an additional effect (CR 707.9e), an exception that applies only if the copy has
/// certain characteristics (CR 707.9f), or a linked triggered ability (CR 707.9g). It's
/// recorded as the permanent's entry is being modified and performed as it enters, if it
/// enters as a copy and no other copy effect was applied after the one it belongs to.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CopyExtra {
    pub ctx: Ctx,
    pub only_if: Option<Filter>,
    pub mods: Vec<Modification>,
    pub effect: Effect,
}

/// How a copy of a card defined by name is made.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NamedCopy {
    /// "Choose a card name [that hasn't been chosen] from among [names]. Create a copy of
    /// the card with the chosen name." The copy's characteristics come from the Oracle
    /// card reference, and it's created outside the game (CR 707.13). With `unchosen`, the
    /// source remembers the names chosen for it.
    OneOf { names: Vec<SmolStr>, unchosen: bool },
    /// "Create a copy of the card with the noted name": the characteristics of that card
    /// as it last existed where it was noted (a graveyard) are its copiable values
    /// (CR 707.14). The copy is created outside the game.
    LastKnown,
}

/// Performs the copy exception extras of a permanent that entered as a copy (`copy`: the
/// copy effect it entered with). Counters it enters with are added to `counters`; other
/// effects are returned to be performed as it enters.
pub fn apply_copy_extras(
    g: &mut Game,
    id: ObjectId,
    copy: Option<u32>,
    extras: &[CopyExtra],
    counters: &mut Vec<(CounterKind, u32)>,
) -> Vec<(Ctx, Effect)> {
    let Some(eid) = copy else {
        // Not a copy: the exceptions of a copy effect don't apply.
        return vec![];
    };
    if extras.is_empty() {
        return vec![];
    }
    // CR 707.9f: whether a conditional exception applies is determined from what the
    // permanent would be if the copy effect applied without it (its other exceptions
    // included): the characteristics it has now.
    let applying: Vec<&CopyExtra> = extras
        .iter()
        .filter(|x| {
            x.only_if.as_ref().is_none_or(|f| {
                let ctl = g.obj(id).controller;
                g.matches(id, f, &Ctx::new(Some(id), ctl))
            })
        })
        .collect();
    let mods: Vec<Modification> = applying.iter().flat_map(|x| x.mods.clone()).collect();
    if !mods.is_empty() {
        if let Some(e) = g.effects.iter_mut().find(|e| e.id == eid) {
            if let Some(Layer1::Copy { exceptions, .. }) = &mut e.layer1 {
                exceptions.extend(mods);
            }
        }
        g.dirty = true;
        g.recompute();
    }
    let mut later = Vec::new();
    for x in applying {
        match &x.effect {
            Effect::Noop => {}
            Effect::EnterWithCounters { kind, n } => {
                let mut c = x.ctx.clone();
                c.source = Some(id);
                let k = g.eval_value(n, &c).max(0) as u32;
                if k > 0 {
                    counters.push((kind.clone(), k));
                }
            }
            e => later.push((x.ctx.clone(), e.clone())),
        }
    }
    later
}

/// The target slots (per chosen mode) of a spell or ability on the stack.
fn target_slots(g: &Game, id: ObjectId) -> Vec<(usize, usize, TargetSpec)> {
    let body = g.stack_body(id);
    let Some(si) = g.obj(id).stack.as_deref() else {
        return vec![];
    };
    let mut out = Vec::new();
    for (cm, m) in si.chosen.iter().enumerate() {
        let specs = match (m.mode, &body.modal) {
            (Some(k), Some(modal)) => modal
                .modes
                .get(k)
                .map(|x| x.targets.clone())
                .unwrap_or_default(),
            _ => body.targets.clone(),
        };
        for (slot, v) in m.targets.iter().enumerate() {
            if let Some(spec) = specs.get(slot) {
                for _ in v {
                    out.push((cm, slot, spec.clone()));
                }
            }
        }
    }
    out
}

/// Whether `e` would be a legal target for each instance of the word "target" of the
/// spell or ability `id` (CR 707.10d, 707.10e).
fn legal_for_each_target(g: &Game, id: ObjectId, e: Entity) -> bool {
    let slots = target_slots(g, id);
    if slots.is_empty() || e == Entity::Object(id) {
        return false;
    }
    let ctx = g.stack_ctx(id);
    slots
        .iter()
        .all(|(_, _, spec)| g.is_legal_target(spec, e, &ctx, id))
}

/// Puts a copy of `spell` onto the stack under `controller`'s control with each of its
/// targets changed to `e`.
fn copy_targeting(
    g: &mut Game,
    spell: ObjectId,
    controller: PlayerId,
    e: Entity,
) -> Option<ObjectId> {
    let copy = crate::copy::copy_spell(g, spell, controller, false)?;
    if let Some(si) = g.objects[copy.0 as usize].stack.as_mut() {
        for m in si.chosen.iter_mut() {
            for v in m.targets.iter_mut() {
                for t in v.iter_mut() {
                    *t = e;
                }
            }
        }
    }
    g.emit(crate::events::Event::BecameTarget {
        target: e,
        by: copy,
        controller,
    });
    g.dirty = true;
    Some(copy)
}

/// CR 707.10d: copies `spell` for each other object or player it could target; each copy
/// targets a different one of them. The copies are put onto the stack in the order their
/// controller chooses.
pub fn copy_for_each_target(g: &mut Game, spell: ObjectId, controller: PlayerId) -> Vec<ObjectId> {
    if !g.is_live(spell) || g.obj(spell).zone != Zone::Stack {
        return vec![];
    }
    if g.dirty {
        g.recompute();
    }
    let current: Vec<Entity> = g
        .obj(spell)
        .stack
        .as_deref()
        .map(|si| {
            si.chosen
                .iter()
                .flat_map(|m| m.targets.iter().flatten().copied())
                .collect()
        })
        .unwrap_or_default();
    let slots = target_slots(g, spell);
    let Some((_, _, spec)) = slots.first() else {
        return vec![];
    };
    let ctx = g.stack_ctx(spell);
    let mut cands: Vec<Entity> = g
        .legal_target_candidates(spec, &ctx, spell)
        .into_iter()
        .filter(|e| !current.contains(e))
        .filter(|e| legal_for_each_target(g, spell, *e))
        .collect();
    if cands.len() > 1 {
        let items = cands.iter().map(|e| format!("{e:?}")).collect();
        if let crate::decision::Answer::Indices(order) = g.ask(
            controller,
            crate::decision::Decision::Order {
                prompt: "Order the copies (first is put onto the stack first)".into(),
                items,
            },
        ) {
            let mut sorted = order.clone();
            sorted.sort();
            if sorted == (0..cands.len()).collect::<Vec<_>>() {
                cands = order.iter().map(|i| cands[*i]).collect();
            }
        }
    }
    cands
        .into_iter()
        .filter_map(|e| copy_targeting(g, spell, controller, e))
        .collect()
}

/// CR 707.10e: copies `spell` with a specified new target: each of the copy's targets must
/// be that object or player; if it isn't legal for each, the copy isn't created. If
/// several were specified (e.g. an effect created more tokens than usual), the copy's
/// controller chooses one of the legal ones.
pub fn copy_with_target(
    g: &mut Game,
    spell: ObjectId,
    controller: PlayerId,
    targets: Vec<Entity>,
) -> Option<ObjectId> {
    if !g.is_live(spell) || g.obj(spell).zone != Zone::Stack {
        return None;
    }
    if g.dirty {
        g.recompute();
    }
    let legal: Vec<Entity> = targets
        .into_iter()
        .filter(|e| legal_for_each_target(g, spell, *e))
        .collect();
    let e = match legal.len() {
        0 => return None,
        1 => legal[0],
        _ => g
            .ask_entities(
                controller,
                Some(spell),
                "Choose the copy's target",
                legal.clone(),
                1,
                1,
            )
            .first()
            .copied()
            .filter(|e| legal.contains(e))
            .unwrap_or(legal[0]),
    };
    copy_targeting(g, spell, controller, e)
}

/// CR 707.8a: a token that's a copy of a double-faced permanent or card is a
/// double-faced token with both faces, entering with the face up that the original has
/// up. Returns the face it should enter with when `src` is such an object whose copiable
/// values are those of its card (no other copy effect applies to it): the token then takes
/// each face's characteristics from the card instead of a single set of copied values.
pub fn double_faced_copy_face(g: &Game, src: ObjectId) -> Option<FaceState> {
    let o = g.obj(src);
    let card = o.card.as_ref()?;
    let transforming = matches!(
        card.layout,
        crate::card::Layout::Transform
            | crate::card::Layout::ModalDfc
            | crate::card::Layout::DoubleFacedToken
            | crate::card::Layout::Battle
    );
    if !transforming || card.faces.len() < 2 || o.face_down {
        return None;
    }
    let face = match o.face {
        FaceState::Back => FaceState::Back,
        _ => FaceState::Front,
    };
    // Only if no copy effect changed its copiable values.
    let printed = card.characteristics(face);
    (o.copiable.name == printed.name && o.copiable.abilities.len() == printed.abilities.len())
        .then_some(face)
}

/// Creates a copy of a card as an object of kind `CardCopy` in `zone`.
fn new_card_copy(
    g: &mut Game,
    card: Option<std::sync::Arc<crate::card::CardDef>>,
    chars: Characteristics,
    controller: PlayerId,
    zone: Zone,
) -> Option<ObjectId> {
    let id = g.create_card_object(card?, controller, zone);
    {
        let o = &mut g.objects[id.0 as usize];
        o.kind = ObjKind::CardCopy;
        o.base = chars.clone();
        o.copiable = chars.clone();
        o.chars = chars;
        o.controller = controller;
        o.base_controller = controller;
    }
    if let Some(l) = g.zone_list_mut(zone) {
        l.push(id);
    }
    g.dirty = true;
    Some(id)
}

/// Performs [`Effect::CopyCard`]. Returns the copies created.
pub fn copy_cards(
    g: &mut Game,
    what: &Sel,
    named: &Option<NamedCopy>,
    ctx: &mut Ctx,
) -> Vec<ObjectId> {
    let controller = ctx.controller;
    let mut out = Vec::new();
    match named {
        None => {
            // CR 707.12: created in the zone the object is in.
            for o in g.resolve_objects(what, ctx) {
                if !g.is_live(o) {
                    continue;
                }
                let ob = g.obj(o).clone();
                out.extend(new_card_copy(
                    g,
                    ob.card.clone(),
                    ob.copiable.clone(),
                    controller,
                    ob.zone,
                ));
            }
        }
        Some(NamedCopy::LastKnown) => {
            // CR 707.14: the noted card as it last existed where it was noted.
            let noted: Vec<ObjectId> = match what {
                Sel::Var(v) => ctx
                    .vars
                    .get(v)
                    .map(|es| es.iter().filter_map(|e| e.object()).collect())
                    .unwrap_or_default(),
                other => g.resolve_objects(other, ctx),
            };
            for o in noted {
                let ob = g.obj(o).clone();
                let chars = ob.copiable.clone();
                out.extend(new_card_copy(
                    g,
                    ob.card.clone(),
                    chars,
                    controller,
                    Zone::Outside(controller),
                ));
            }
        }
        Some(NamedCopy::OneOf { names, unchosen }) => {
            // CR 707.13: a card defined by name, from the Oracle card reference, created
            // outside the game. The source keeps track of the names chosen for it.
            let src = ctx.source;
            let link = ctx.link;
            let chosen: Vec<SmolStr> = src
                .and_then(|s| g.obj(s).linked_choices.get(&link))
                .and_then(|c| c.text.clone())
                .map(|t| t.split('|').map(SmolStr::new).collect())
                .unwrap_or_default();
            let options: Vec<SmolStr> = names
                .iter()
                .filter(|n| !*unchosen || !chosen.contains(n))
                .cloned()
                .collect();
            if options.is_empty() {
                return out;
            }
            let k = g.ask_option(
                controller,
                src,
                "Choose a card name",
                options.iter().map(|n| n.to_string()).collect(),
            );
            let name = options[k.min(options.len() - 1)].clone();
            if let Some(s) = src {
                let mut all = chosen.clone();
                all.push(name.clone());
                let joined: Vec<&str> = all.iter().map(|n| n.as_str()).collect();
                g.objects[s.0 as usize]
                    .linked_choices
                    .entry(link)
                    .or_default()
                    .text = Some(SmolStr::new(joined.join("|")));
                g.objects[s.0 as usize].choices.card_name = Some(name.clone());
            }
            if let Some(card) = crate::card::CardDb::global().get(&name) {
                let chars = card.characteristics(FaceState::Front);
                out.extend(new_card_copy(
                    g,
                    Some(card),
                    chars,
                    controller,
                    Zone::Outside(controller),
                ));
            }
        }
    }
    g.recompute();
    ctx.set_var(
        vars::CREATED,
        out.iter().map(|o| Entity::Object(*o)).collect(),
    );
    out
}
