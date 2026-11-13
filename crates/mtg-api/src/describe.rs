//! Readable descriptions of objects, players, actions and events for a viewer, never
//! naming what the viewer can't see.

use crate::view::zone_name;
use crate::visibility::{can_see, Viewer};
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::{Entity, Game, ObjectId, PlayerId};

/// A player's short name in descriptions: "P0", "P1", ...
pub fn player_name(p: PlayerId) -> String {
    format!("P{}", p.0)
}

fn known_name(g: &Game, id: ObjectId) -> String {
    let o = g.obj(id);
    let name = if o.face_down {
        mtg_engine::facedown::revealed_characteristics(g, id)
            .name
            .to_string()
    } else {
        o.chars.name.to_string()
    };
    if name.is_empty() {
        let kind = o
            .chars
            .card_types
            .iter()
            .next()
            .map(|t| t.word())
            .unwrap_or("object");
        format!("{kind} #{}", id.0)
    } else if o.face_down {
        format!("face-down {name} #{}", id.0)
    } else {
        format!("{name} #{}", id.0)
    }
}

/// The name of the object `id` as `viewer` may refer to it: "Grizzly Bears #12"; a
/// face-down permanent or spell the viewer can't look at is "face-down creature #12"; a
/// card in a hidden zone the viewer can't see is "a hidden card" (without its id).
pub fn object_name(g: &Game, viewer: Viewer, id: ObjectId) -> String {
    if (id.0 as usize) >= g.objects.len() {
        return format!("object #{}", id.0);
    }
    if can_see(g, viewer, id) {
        return known_name(g, id);
    }
    let o = g.obj(id);
    match o.zone {
        Zone::Battlefield if o.face_down => format!("face-down creature #{}", id.0),
        Zone::Stack if o.face_down => format!("face-down spell #{}", id.0),
        Zone::Exile | Zone::Command if o.face_down => format!("face-down card #{}", id.0),
        _ => "a hidden card".to_string(),
    }
}

/// The real name of an object everyone was shown (a revealed card).
pub fn revealed_name(g: &Game, id: ObjectId) -> String {
    if (id.0 as usize) >= g.objects.len() {
        return format!("object #{}", id.0);
    }
    known_name(g, id)
}

/// The id of `id` as the viewer may know it: `None` for a card hidden from them in a
/// library, hand, or outside the game.
pub fn visible_id(g: &Game, viewer: Viewer, id: ObjectId) -> Option<u32> {
    if (id.0 as usize) >= g.objects.len() {
        return None;
    }
    let o = g.obj(id);
    let hidden_zone = matches!(o.zone, Zone::Library(_) | Zone::Hand(_) | Zone::Outside(_));
    (!hidden_zone || can_see(g, viewer, id)).then_some(id.0)
}

pub fn entity_name(g: &Game, viewer: Viewer, e: Entity) -> String {
    match e {
        Entity::Player(p) => player_name(p),
        Entity::Object(o) => object_name(g, viewer, o),
    }
}

/// The text of the ability with this uid, searched on `source` first, then on every
/// object (static abilities granting alternative costs and special actions).
pub fn ability_text(g: &Game, source: Option<ObjectId>, uid: u64) -> Option<String> {
    let find = |id: ObjectId| {
        g.obj(id)
            .chars
            .abilities
            .iter()
            .find(|a| a.uid == uid)
            .map(|a| a.text.clone())
    };
    if let Some(s) = source.filter(|s| (s.0 as usize) < g.objects.len()) {
        if let Some(t) = find(s) {
            return Some(t);
        }
    }
    (0..g.objects.len() as u32).map(ObjectId).find_map(find)
}

/// How a spell is being cast, in words ("with flashback", "face down", ...); empty for
/// a normal cast.
pub fn cast_method_text(g: &Game, card: ObjectId, m: &CastMethod) -> String {
    match m {
        CastMethod::Normal => String::new(),
        CastMethod::Free => "without paying its mana cost".into(),
        CastMethod::Keyword(k) => format!("with {}", k.name().to_lowercase()),
        CastMethod::FaceDown(k) => format!("face down ({})", k.name().to_lowercase()),
        CastMethod::Alternative(uid) => match ability_text(g, Some(card), *uid) {
            Some(t) if !t.is_empty() => format!("using \"{t}\""),
            _ => "using an alternative cost".into(),
        },
        CastMethod::Half(i) => {
            let face = g
                .obj(card)
                .card
                .as_ref()
                .and_then(|c| c.faces.get(*i as usize))
                .map(|f| f.chars.name.to_string());
            match face {
                Some(n) if !n.is_empty() => format!("as {n}"),
                _ => format!("as face {i}"),
            }
        }
    }
}

/// A short machine-friendly name of a cast method: "normal", "free", "flashback",
/// "face_down:morph", "alternative:<uid>", "half:<i>".
pub fn cast_method_code(m: &CastMethod) -> String {
    match m {
        CastMethod::Normal => "normal".into(),
        CastMethod::Free => "free".into(),
        CastMethod::Keyword(k) => k.name().to_lowercase().replace(' ', "_"),
        CastMethod::FaceDown(k) => format!("face_down:{}", k.name().to_lowercase()),
        CastMethod::Alternative(uid) => format!("alternative:{uid}"),
        CastMethod::Half(i) => format!("half:{i}"),
    }
}

/// A one-line description of a priority action.
pub fn action_text(g: &Game, viewer: Viewer, a: &Action) -> String {
    let name = |id: ObjectId| object_name(g, viewer, id);
    let from = |id: ObjectId| match g.obj(id).zone {
        Zone::Hand(_) => String::new(),
        z => format!(" from {}", zone_name(z)),
    };
    match a {
        Action::Pass => "Pass priority".into(),
        Action::Concede => "Concede the game".into(),
        Action::PlayLand { card } => format!("Play {}{}", name(*card), from(*card)),
        Action::Cast { card, method } => {
            let o = g.obj(*card);
            let cost = o
                .chars
                .mana_cost
                .as_ref()
                .map(|m| format!(" ({m})"))
                .unwrap_or_default();
            let how = cast_method_text(g, *card, method);
            let how = if how.is_empty() {
                String::new()
            } else {
                format!(" {how}")
            };
            format!("Cast {}{}{}{}", name(*card), cost, from(*card), how)
        }
        Action::Activate { source, ability } => {
            let text = ability_text(g, Some(*source), *ability).unwrap_or_default();
            let mana = g
                .obj(*source)
                .chars
                .abilities
                .iter()
                .any(|x| x.uid == *ability && x.is_mana_ability());
            format!(
                "Activate {}{}: {}",
                if mana { "mana ability of " } else { "" },
                name(*source),
                if text.is_empty() { "ability" } else { &text }
            )
        }
        Action::Special(s) => match s {
            SpecialAction::TurnFaceUp { obj } => format!("Turn {} face up", name(*obj)),
            SpecialAction::Suspend { card } => format!("Suspend {}", name(*card)),
            SpecialAction::Foretell { card } => format!("Foretell {}", name(*card)),
            SpecialAction::Plot { card } => format!("Plot {}", name(*card)),
            SpecialAction::CompanionToHand { card } => {
                format!("Put companion {} into your hand", name(*card))
            }
            SpecialAction::Static { source, ability } => {
                let text = ability_text(g, Some(*source), *ability).unwrap_or_default();
                format!("Special action of {}: {}", name(*source), text)
            }
            SpecialAction::Offer { id } => {
                let src = g
                    .special
                    .offers
                    .iter()
                    .find(|o| o.id == *id)
                    .and_then(|o| o.ctx.source);
                match src {
                    Some(s) => format!("Special action allowed by {}", name(s)),
                    None => format!("Special action #{id}"),
                }
            }
            SpecialAction::RollPlanarDie => "Roll the planar die".into(),
            SpecialAction::Other { name: n, obj } => match obj {
                Some(o) => format!("{n}: {}", name(*o)),
                None => n.clone(),
            },
        },
    }
}
