//! Permanents represented by more than one card or token: merged permanents (CR 730,
//! from mutate, CR 702.140) and melded permanents (CR 701.42, 712.4).
//!
//! Such a permanent keeps its components in `merged_with`, topmost first: objects outside
//! every zone that carry each card or token. A merged permanent has the characteristics
//! of its topmost component and the abilities of all of them (CR 730.2a, 702.140e); a
//! melded permanent (face `Melded`) has those of the meld result's combined back face
//! (CR 712.4a). When it leaves the battlefield, one permanent leaves and each component
//! is put into the new zone (CR 730.3); stickers on it stay with only one of the objects
//! it becomes, chosen by its owner (CR 123.5c).

use crate::ability::LibraryPosition;
use crate::card::{CardDb, CardDef};
use crate::eval::Ctx;
use crate::events::{Event, MoveCause};
use crate::game::Game;
use crate::object::{Characteristics, EventInfo, FaceState, ObjKind, Zone};
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::*;
use smol_str::SmolStr;
use std::sync::Arc;

/// `Event::Custom` name: a spell merged with a creature as a resolving mutating creature
/// spell (CR 702.140d). `obj` is the mutated permanent.
pub const MUTATES: &str = "mutates";
/// `TriggerCond::Custom` name of "Whenever this creature mutates".
pub const MUTATES_SELF: &str = "mutates:self";
/// `Effect::Custom` name prefix of "exile them, then meld them into [result]"; the
/// result's name follows (empty: the source's meld result).
pub const MELD_EFFECT: &str = "meld into:";
/// `Condition::Custom` name: "you both own and control [this] and its meld partner".
pub const MELD_PAIR_CONDITION: &str = "own and control meld pair";

/// The cards and tokens that represent `id`, topmost first. A melded component counts as
/// its two cards. Empty for an object represented by a single card or token.
pub fn physical_components(g: &Game, id: ObjectId) -> Vec<ObjectId> {
    let mut out = Vec::new();
    for c in &g.obj(id).merged_with {
        if g.obj(*c).merged_with.is_empty() {
            out.push(*c);
        } else {
            out.extend(physical_components(g, *c));
        }
    }
    out
}

/// Whether `id` is a merged permanent (represented by more than one card or token).
pub fn is_merged(g: &Game, id: ObjectId) -> bool {
    physical_components(g, id).len() > 1
}

/// A new component representing the card or token `obj` is (outside every zone). A
/// melded permanent becoming a component keeps its own components.
fn new_component(g: &mut Game, obj: ObjectId) -> ObjectId {
    let o = g.obj(obj).clone();
    let id = match (&o.card, o.kind) {
        (Some(card), ObjKind::Card) => g.create_card_object(card.clone(), o.owner, Zone::Nowhere),
        // Tokens, and copies of permanent spells (which become tokens, CR 608.3f).
        (_, ObjKind::Token) => g.create_token_object(o.base.clone(), o.owner),
        _ => g.create_token_object(o.copiable.clone(), o.owner),
    };
    let c = &mut g.objects[id.0 as usize];
    c.face_down = o.face_down;
    if o.face == FaceState::Melded {
        c.face = FaceState::Melded;
        c.base = o.base.clone();
        c.merged_with = o.merged_with.clone();
    }
    id
}

/// The characteristics a component contributes (its printed ones, CR 730.2a).
fn component_chars(g: &Game, c: ObjectId) -> Characteristics {
    let o = g.obj(c);
    match &o.card {
        Some(card) if o.kind == ObjKind::Card && o.face != FaceState::Melded => {
            card.characteristics(FaceState::Front)
        }
        _ => o.base.clone(),
    }
}

/// CR 730.2a, 702.140e: a merged permanent has only the characteristics of its topmost
/// component, and the abilities of each of its components. It's a token only if the
/// topmost component is (CR 730.2d).
fn refresh(g: &mut Game, id: ObjectId) {
    let comps = g.obj(id).merged_with.clone();
    let Some(top) = comps.first().copied() else {
        return;
    };
    let mut base = component_chars(g, top);
    base.abilities = comps
        .iter()
        .flat_map(|c| component_chars(g, *c).abilities)
        .collect();
    let t = g.obj(top);
    let (card, kind, face) = (t.card.clone(), t.kind, t.face);
    let o = &mut g.objects[id.0 as usize];
    o.base = base;
    o.card = card;
    o.kind = kind;
    o.face = face;
    g.dirty = true;
}

/// CR 730.2: merges `obj` with the permanent `target`, putting it on top of or under it.
/// The permanent stays the same object (CR 730.2c) and isn't considered to have entered
/// the battlefield (CR 730.2b). Stickers on `obj` are on the merged permanent (CR 123.5b).
pub fn merge(g: &mut Game, obj: ObjectId, target: ObjectId, on_top: bool) {
    let comp = new_component(g, obj);
    let t = g.obj(target);
    if t.merged_with.is_empty() || t.face == FaceState::Melded {
        let own = new_component(g, target);
        g.objects[target.0 as usize].merged_with = vec![own];
    }
    let list = &mut g.objects[target.0 as usize].merged_with;
    if on_top {
        list.insert(0, comp);
    } else {
        list.push(comp);
    }
    crate::stickers::merge_into(g, obj, target);
    refresh(g, target);
    g.recompute();
}

/// Called as the object `new` is created for `old` (CR 400.7). A merged or melded
/// permanent leaving the battlefield becomes its first component (the others follow in
/// [`after_leaving`]); a melded permanent entering keeps its components.
pub fn incarnation(g: &mut Game, old: ObjectId, new: ObjectId) {
    let o = g.obj(old);
    if o.merged_with.is_empty() {
        return;
    }
    if o.zone == Zone::Battlefield {
        let Some(first) = physical_components(g, old).first().copied() else {
            return;
        };
        let f = g.obj(first).clone();
        let n = &mut g.objects[new.0 as usize];
        n.card = f.card.clone();
        n.kind = f.kind;
        n.owner = f.owner;
        n.face = FaceState::Front;
        n.base = match &f.card {
            Some(card) if f.kind == ObjKind::Card => card.characteristics(FaceState::Front),
            _ => f.base.clone(),
        };
    } else {
        let (comps, face) = (o.merged_with.clone(), o.face);
        let n = &mut g.objects[new.0 as usize];
        n.merged_with = comps;
        n.face = face;
    }
}

/// CR 730.3: after the merged or melded permanent `old` left the battlefield as `new`,
/// each of its other components is put into the same zone. If the zone is public and it
/// had stickers, its owner chooses which of the objects it became keeps them
/// (CR 123.5c).
pub fn after_leaving(g: &mut Game, old: ObjectId, new: ObjectId, m: &MoveEv) {
    let phys = physical_components(g, old);
    if phys.len() < 2 {
        return;
    }
    let mut news = vec![new];
    for c in &phys[1..] {
        let owner = g.obj(*c).owner;
        let to = match m.to {
            Zone::Graveyard(_) => Zone::Graveyard(owner),
            Zone::Hand(_) => Zone::Hand(owner),
            Zone::Library(_) => Zone::Library(owner),
            z => z,
        };
        let ev = MoveEv {
            obj: *c,
            to,
            pos: m.pos,
            cause: m.cause,
            by: m.by,
            etb: EtbInfo {
                face_down: m.etb.face_down,
                ..Default::default()
            },
            source: m.source,
        };
        if let Some(n) = g.perform_move(ev, None) {
            news.push(n);
        }
    }
    if m.to.is_public() && crate::stickers::is_stickered(g, new) && news.len() > 1 {
        let owner = g.obj(old).owner;
        let options = news.iter().map(|n| g.describe(*n)).collect();
        let pick = g.ask_option(owner, Some(new), "Which card keeps the stickers?", options);
        if let Some(to) = news.get(pick).copied().filter(|to| *to != new) {
            crate::stickers::transfer(g, new, to);
        }
    }
}

/// Merges a resolving mutating creature spell with its target (CR 702.140c): its
/// controller chooses whether it goes on top or under; the spell leaves the stack and
/// effects referring to it refer to the mutated permanent (CR 702.140f).
pub fn mutate(g: &mut Game, spell: ObjectId, target: ObjectId) {
    let controller = g.obj(spell).controller;
    let on_top = g.ask_option(
        controller,
        Some(spell),
        "Put the mutating spell on top of the creature or under it?",
        vec!["On top".into(), "Under".into()],
    ) == 0;
    merge(g, spell, target, on_top);
    g.remove_from_stack(spell);
    let s = &mut g.objects[spell.0 as usize];
    s.zone = Zone::Nowhere;
    s.next = Some(target);
    g.log(|g| format!("{} mutates", g.describe(target)));
    g.emit(Event::Custom {
        name: SmolStr::new(MUTATES),
        player: Some(controller),
        obj: Some(target),
        amount: 0,
    });
    g.emit(Event::SpellResolved { spell });
}

/// "Whenever this creature mutates" (CR 702.140d).
pub fn custom_trigger(name: &str, src: ObjectId, ev: &Event) -> Option<Vec<EventInfo>> {
    if name != MUTATES_SELF {
        return None;
    }
    Some(match ev {
        Event::Custom {
            name: n,
            player,
            obj: Some(o),
            ..
        } if n == MUTATES && *o == src => vec![EventInfo {
            object: Some(src),
            player: *player,
            ..Default::default()
        }],
        _ => vec![],
    })
}

/// Whether the card named `name` is one of the two meld cards of `result`.
fn is_meld_part(result: &CardDef, name: &str) -> bool {
    result
        .related
        .iter()
        .any(|(k, n)| k == "meld_part" && n.eq_ignore_ascii_case(name))
}

fn card_name(g: &Game, id: ObjectId) -> Option<SmolStr> {
    g.obj(id).card.as_ref().map(|c| c.name.clone())
}

/// CR 701.42a: melds two cards of a meld pair: they're put onto the battlefield as one
/// permanent with their back faces up and combined, under `controller`'s control. Only
/// two cards of the same meld pair can be melded (CR 701.42b); otherwise they stay where
/// they are (CR 701.42c). Stickers on them are on the melded permanent, keeping their
/// relative timestamp order (CR 123.5a). Returns the melded permanent.
pub fn meld(
    g: &mut Game,
    a: ObjectId,
    b: ObjectId,
    result: Arc<CardDef>,
    controller: PlayerId,
) -> Option<ObjectId> {
    let part = |g: &Game, x: ObjectId| {
        g.is_live(x)
            && g.obj(x).kind == ObjKind::Card
            && card_name(g, x).is_some_and(|n| is_meld_part(&result, &n))
    };
    if a == b
        || !part(g, a)
        || !part(g, b)
        || card_name(g, a) == card_name(g, b)
        || g.obj(a).owner != g.obj(b).owner
    {
        return None;
    }
    let owner = g.obj(a).owner;
    let ca = new_component(g, a);
    let cb = new_component(g, b);
    let m = g.create_card_object(result.clone(), owner, Zone::Nowhere);
    {
        let o = &mut g.objects[m.0 as usize];
        o.face = FaceState::Melded;
        o.base = result.characteristics(FaceState::Melded);
        o.merged_with = vec![ca, cb];
    }
    crate::stickers::redirect(g, &[a, b], m);
    let new = g.move_object_ev(MoveEv {
        obj: m,
        to: Zone::Battlefield,
        pos: LibraryPosition::Top,
        cause: MoveCause::Effect,
        by: Some(controller),
        etb: EtbInfo {
            controller: Some(controller),
            ..Default::default()
        },
        source: None,
    })?;
    // The two cards are now the melded permanent.
    for x in [a, b] {
        let zone = g.obj(x).zone;
        if let Some(list) = g.zone_list_mut(zone) {
            list.retain(|y| *y != x);
        }
        let o = &mut g.objects[x.0 as usize];
        o.zone = Zone::Nowhere;
        o.next = Some(new);
    }
    g.dirty = true;
    Some(new)
}

/// The meld result of the card `src` is, and its meld partner on the battlefield that
/// `p` both owns and controls, if `src` is also a permanent `p` owns and controls.
fn meld_pair(
    g: &Game,
    src: ObjectId,
    p: PlayerId,
    result_name: &str,
) -> Option<(Arc<CardDef>, ObjectId)> {
    let owned_and_controlled =
        |x: ObjectId| g.is_live(x) && g.obj(x).owner == p && g.obj(x).controller == p;
    if !owned_and_controlled(src) || g.obj(src).zone != Zone::Battlefield {
        return None;
    }
    let card = g.obj(src).card.clone()?;
    let result = card
        .related
        .iter()
        .find(|(k, _)| k == "meld_result")
        .and_then(|(_, n)| CardDb::global().get(n))
        .or_else(|| CardDb::global().get(result_name))?;
    if !is_meld_part(&result, &card.name) {
        return None;
    }
    let partner = g.battlefield.iter().copied().find(|x| {
        *x != src
            && owned_and_controlled(*x)
            && card_name(g, *x).is_some_and(|n| n != card.name && is_meld_part(&result, &n))
    })?;
    Some((result, partner))
}

/// "If you both own and control [this] and [its meld partner], exile them, then meld
/// them into [result]" (CR 701.42a).
pub fn meld_effect(g: &mut Game, result_name: &str, ctx: &Ctx) {
    let Some(src) = ctx.source else {
        return;
    };
    let p = ctx.controller;
    let Some((result, partner)) = meld_pair(g, src, p, result_name) else {
        return;
    };
    let a = g.move_object(src, Zone::Exile, MoveCause::Effect, Some(p));
    let b = g.move_object(partner, Zone::Exile, MoveCause::Effect, Some(p));
    if let (Some(a), Some(b)) = (a, b) {
        meld(g, a, b, result, p);
    }
}

/// "you both own and control [this] and its meld partner".
pub fn custom_condition(g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
    (name == MELD_PAIR_CONDITION).then(|| {
        ctx.source
            .is_some_and(|s| meld_pair(g, s, ctx.controller, "").is_some())
    })
}

/// Custom effects for melding. Returns true if handled.
pub fn custom_effect(g: &mut Game, name: &str, ctx: &Ctx) -> bool {
    match name.strip_prefix(MELD_EFFECT) {
        Some(result) => {
            meld_effect(g, result, ctx);
            true
        }
        None => false,
    }
}
