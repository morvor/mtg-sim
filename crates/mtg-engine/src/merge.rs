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
use std::collections::BTreeMap;
use std::sync::Arc;

/// Bookkeeping for merged permanents, in `Game::merges`.
#[derive(Clone, Debug, Default)]
pub struct MergeState {
    /// The objects a merged permanent became as it left the battlefield (CR 730.3), by the
    /// first of them (the one `Game::current` follows).
    pub left_together: BTreeMap<ObjectId, Vec<ObjectId>>,
}

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
    // A double-faced component keeps the face it has up; a flipped one stays flipped.
    if matches!(o.face, FaceState::Back | FaceState::Flipped) && c.kind == ObjKind::Card {
        c.face = o.face;
    }
    if o.face == FaceState::Melded {
        c.face = FaceState::Melded;
        c.base = o.base.clone();
        c.merged_with = o.merged_with.clone();
    }
    id
}

/// The characteristics a component contributes (its printed ones, CR 730.2a): those of
/// the face it has up — a double-faced component turned to its other face (CR 730.2i).
fn component_chars(g: &Game, c: ObjectId) -> Characteristics {
    let o = g.obj(c);
    match &o.card {
        Some(card) if o.kind == ObjKind::Card && o.face != FaceState::Melded => {
            let face = match o.face {
                FaceState::Back | FaceState::Flipped => o.face,
                _ => FaceState::Front,
            };
            card.characteristics(face)
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
    let (card, kind, face, face_down) = (t.card.clone(), t.kind, t.face, t.face_down);
    let o = &mut g.objects[id.0 as usize];
    o.base = base;
    o.card = card;
    o.kind = kind;
    // A merged permanent isn't a double-faced permanent (CR 730.2i): its components'
    // faces are kept on the components.
    o.face = match (face, o.face) {
        (FaceState::Melded, _) => face,
        // A flipped merged permanent stays flipped (CR 730.2h).
        (_, FaceState::Flipped) => FaceState::Flipped,
        _ => FaceState::Front,
    };
    // CR 730.2e: face up or face down as its topmost component is. A face-down permanent
    // that becomes face up this way isn't "turned face up".
    o.face_down = face_down;
    g.dirty = true;
}

/// Whether `id` contains a component represented by a double-faced card that can
/// transform (CR 730.2i, 730.2j).
fn has_double_faced_component(g: &Game, id: ObjectId) -> bool {
    physical_components(g, id).iter().any(|c| {
        let o = g.obj(*c);
        o.kind == ObjKind::Card
            && o.card.as_ref().is_some_and(|card| {
                card.layout.is_double_faced()
                    && card.layout != crate::card::Layout::Meld
                    && card.faces.len() > 1
            })
    })
}

/// `Effect::Custom` name: "flip [this permanent]" (CR 710).
pub const FLIP: &str = "flip this permanent";

/// Whether the component or object `c` is represented by a flip card (CR 710.1).
fn is_flip_card(g: &Game, c: ObjectId) -> bool {
    let o = g.obj(c);
    o.kind == ObjKind::Card
        && o.card
            .as_ref()
            .is_some_and(|card| card.layout == crate::card::Layout::Flip && card.faces.len() > 1)
}

/// Flips a permanent (CR 710.2): its alternative characteristics apply from now on;
/// flipping is a one-way process (CR 710.4). A merged permanent that's flipped uses the
/// alternative characteristics of each of its flip-card components (CR 730.2h). Returns
/// true if it flipped.
pub fn flip(g: &mut Game, id: ObjectId) -> bool {
    let o = g.obj(id);
    if !g.is_live(id) || o.zone != Zone::Battlefield || o.face_down || o.face == FaceState::Flipped
    {
        return false;
    }
    if is_merged(g, id) {
        let comps: Vec<ObjectId> = physical_components(g, id)
            .into_iter()
            .filter(|c| is_flip_card(g, *c))
            .collect();
        for c in &comps {
            g.objects[c.0 as usize].face = FaceState::Flipped;
        }
        refresh(g, id);
        g.objects[id.0 as usize].face = FaceState::Flipped;
        return !comps.is_empty();
    }
    if !is_flip_card(g, id) {
        return false;
    }
    let card = g.obj(id).card.clone().unwrap();
    let o = &mut g.objects[id.0 as usize];
    o.face = FaceState::Flipped;
    o.base = card.characteristics(FaceState::Flipped);
    g.dirty = true;
    g.log(|g| format!("{} flips", g.describe(id)));
    true
}

/// CR 730.2j: a face-up merged permanent that contains a double-faced component can't be
/// turned face down.
pub fn cant_turn_face_down(g: &Game, id: ObjectId) -> bool {
    is_merged(g, id) && !g.obj(id).face_down && has_double_faced_component(g, id)
}

/// CR 730.2g: a face-down merged permanent that contains an instant or sorcery card can't
/// be turned face up.
pub fn cant_turn_face_up(g: &Game, id: ObjectId) -> bool {
    is_merged(g, id)
        && physical_components(g, id).iter().any(|c| {
            let o = g.obj(*c);
            o.kind == ObjKind::Card
                && o.card.as_ref().is_some_and(|card| {
                    let f = card.characteristics(FaceState::Front);
                    f.is(crate::types::CardType::Instant) || f.is(crate::types::CardType::Sorcery)
                })
        })
}

/// CR 730.2f: a merged permanent turned face down has each of its face-up components
/// turned face down; turned face up, each face-down component turned face up.
pub fn turned_face(g: &mut Game, id: ObjectId, face_down: bool) {
    if !is_merged(g, id) {
        return;
    }
    for c in physical_components(g, id) {
        g.objects[c.0 as usize].face_down = face_down;
    }
    refresh(g, id);
}

/// CR 730.2i: transforming (or converting) a merged permanent turns each of its
/// double-faced components that can transform to its other face. Returns true if any
/// did.
pub fn transform_merged(g: &mut Game, id: ObjectId) -> bool {
    let o = g.obj(id);
    if o.zone != Zone::Battlefield || !g.is_live(id) || o.face_down {
        return false;
    }
    let mut any = false;
    for c in physical_components(g, id) {
        let co = g.obj(c);
        let Some(card) = co.card.clone() else {
            continue;
        };
        if co.kind != ObjKind::Card
            || !card.layout.is_double_faced()
            || card.layout == crate::card::Layout::Meld
            || card.faces.len() < 2
        {
            continue;
        }
        let (new_face, idx) = match co.face {
            FaceState::Back => (FaceState::Front, 0),
            _ => (FaceState::Back, 1),
        };
        // CR 701.27d: not into an instant or sorcery face.
        let types = &card.faces[idx].chars.card_types;
        if types.contains(crate::types::CardType::Instant)
            || types.contains(crate::types::CardType::Sorcery)
        {
            continue;
        }
        g.objects[c.0 as usize].face = new_face;
        any = true;
    }
    if !any {
        return false;
    }
    refresh(g, id);
    let ts = g.new_timestamp();
    g.objects[id.0 as usize].timestamp = ts;
    crate::transform_rules::record(g, id, ts);
    g.emit(Event::Transformed { obj: id });
    true
}

/// [`with_components`] for a selection of entities.
pub fn with_components_of(g: &Game, sel: Vec<Entity>) -> Vec<Entity> {
    if g.merges.left_together.is_empty()
        || !sel
            .iter()
            .any(|e| matches!(e, Entity::Object(o) if g.merges.left_together.contains_key(o)))
    {
        return sel;
    }
    let mut out = Vec::with_capacity(sel.len() + 1);
    for e in sel {
        match e {
            Entity::Object(o) => {
                for x in with_components(g, vec![o]) {
                    if !out.contains(&Entity::Object(x)) {
                        out.push(Entity::Object(x));
                    }
                }
            }
            p => out.push(p),
        }
    }
    out
}

/// The objects in `objs`, each followed by the other objects the merged permanent it came
/// from became as it left the battlefield, if they're still in that zone: an effect that
/// can find the new object a merged permanent became finds all of them (CR 730.3c).
pub fn with_components(g: &Game, objs: Vec<ObjectId>) -> Vec<ObjectId> {
    let mut out = Vec::with_capacity(objs.len());
    for o in objs {
        if !out.contains(&o) {
            out.push(o);
        }
        if let Some(sibs) = g.merges.left_together.get(&o) {
            let zone = g.obj(o).zone;
            for s in sibs {
                if !out.contains(s) && g.is_live(*s) && g.obj(*s).zone == zone {
                    out.push(*s);
                }
            }
        }
    }
    out
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
    // CR 730.2h: a flip card merged into a flipped permanent uses its alternative
    // characteristics.
    if g.obj(target).face == FaceState::Flipped && is_flip_card(g, comp) {
        g.objects[comp.0 as usize].face = FaceState::Flipped;
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
/// each of its other components is put into the same zone: a replacement effect applied
/// to the permanent applied to all of them (CR 730.3d). If the merged permanent is a
/// token, a replacement effect that applies only to cards moves its components that are
/// cards (CR 730.3e). Their owner arranges cards put into a graveyard or library
/// (CR 730.3a); the player who exiles it orders their timestamps (CR 730.3b). If the zone
/// is public and it had stickers, its owner chooses which of the objects it became keeps
/// them (CR 123.5c).
pub fn after_leaving(g: &mut Game, old: ObjectId, new: ObjectId, m: &MoveEv) {
    let phys = physical_components(g, old);
    if phys.len() < 2 {
        return;
    }
    let token_permanent = g.obj(old).kind == ObjKind::Token;
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
        let card = g.obj(*c).kind == ObjKind::Card;
        let moved = if token_permanent && card {
            // CR 730.3e: replacement effects that apply to cards (but not tokens) didn't
            // apply to the token permanent; they apply to its card components.
            g.move_object_ev(ev)
        } else {
            g.perform_move(ev, None)
        };
        if let Some(n) = moved {
            news.push(n);
        }
    }
    order_components(g, old, &news, m);
    if news.len() > 1 {
        g.merges.left_together.insert(new, news.clone());
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

/// CR 730.3a, 730.3b: the owner of a merged permanent put into their graveyard or library
/// arranges its cards in any order there; the player who exiles one determines the
/// relative timestamp order of its cards.
fn order_components(g: &mut Game, old: ObjectId, news: &[ObjectId], m: &MoveEv) {
    let zone = g.obj(news[0]).zone;
    let here: Vec<ObjectId> = news
        .iter()
        .copied()
        .filter(|n| g.is_live(*n) && g.obj(*n).zone == zone)
        .collect();
    if here.len() < 2 {
        return;
    }
    let names: Vec<String> = here
        .iter()
        .map(|n| {
            g.obj(*n)
                .card
                .as_ref()
                .map_or_else(|| g.describe(*n), |c| c.name.to_string())
        })
        .collect();
    match zone {
        Zone::Graveyard(p) | Zone::Library(p)
            if !matches!(
                m.pos,
                LibraryPosition::Shuffled | LibraryPosition::BottomRandom
            ) =>
        {
            let order = g.ask_order(
                p,
                "Arrange the cards of the merged permanent (in the order they're put there)",
                names,
            );
            let Some(list) = g.zone_list_mut(zone) else {
                return;
            };
            let mut slots: Vec<usize> = list
                .iter()
                .enumerate()
                .filter(|(_, x)| here.contains(x))
                .map(|(i, _)| i)
                .collect();
            slots.sort_unstable();
            for (k, slot) in slots.into_iter().enumerate() {
                list[slot] = here[order[k]];
            }
        }
        Zone::Exile => {
            // The player who exiled it: the one responsible for the move, or the
            // controller of the effect that exiled it.
            let who =
                m.by.or_else(|| m.source.map(|s| g.obj(s).controller))
                    .unwrap_or_else(|| g.obj(old).controller);
            let order = g.ask_order(
                who,
                "Choose the timestamp order of the exiled cards (earliest first)",
                names,
            );
            let mut stamps: Vec<Timestamp> = here.iter().map(|n| g.obj(*n).timestamp).collect();
            stamps.sort_unstable();
            for (k, ts) in stamps.into_iter().enumerate() {
                g.objects[here[order[k]].0 as usize].timestamp = ts;
            }
        }
        _ => {}
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

/// Custom effects for melding and flipping. Returns true if handled.
pub fn custom_effect(g: &mut Game, name: &str, ctx: &Ctx) -> bool {
    if name == FLIP {
        if let Some(src) = ctx.source {
            flip(g, src);
        }
        return true;
    }
    match name.strip_prefix(MELD_EFFECT) {
        Some(result) => {
            meld_effect(g, result, ctx);
            true
        }
        None => false,
    }
}
