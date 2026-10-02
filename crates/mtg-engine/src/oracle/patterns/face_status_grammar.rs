//! Face-down and face-up grammar (CR 406.3, 701.40, 708), read compositionally.
//!
//! ```text
//! exile-face-down := instruction-with-one-exile ["face down" after what's exiled]
//!                    ("exile all cards from your hand face down", "target player exiles
//!                    all cards from their hand face down, then draws that many cards",
//!                    "search your library for a card, exile it face down, then shuffle")
//! look-and-take   := "look at the top N cards of" library "," take-part ["," "then"]
//!                    rest-part
//! take-part       := "exile" count ["of them"] ["face down"]
//! rest-part       := "put the rest" / "the other" destination
//! ```
//!
//! An instruction to exile cards face down is the same instruction with the cards exiled
//! face down: no player may look at them unless an instruction allows it (CR 406.3).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;

/// Whether `w` is a form of the verb "exile".
fn is_exile_verb(w: &str) -> bool {
    matches!(w, "exile" | "exiles")
}

/// How many times the verb "exile" appears in `s`.
fn exile_verbs(s: &str) -> usize {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|w| is_exile_verb(w))
        .count()
}

/// Marks every exile in `e` as face down; returns `None` unless exactly one instruction
/// exiles something (the one the text says is face down).
fn exiled_face_down(e: Effect) -> Option<Effect> {
    use serde_json::Value as J;
    fn walk(v: &mut J, n: &mut usize) {
        match v {
            J::Object(m) => {
                // `Effect::Exile { face_down, .. }`.
                if let Some(J::Object(x)) = m.get_mut("Exile") {
                    if let Some(f) = x.get_mut("face_down") {
                        *f = J::Bool(true);
                        *n += 1;
                    }
                }
                // A `Destination` in exile.
                if m.get("zone").and_then(|z| z.as_str()) == Some("Exile") {
                    if let Some(f) = m.get_mut("face_down") {
                        *f = J::Bool(true);
                        *n += 1;
                    }
                }
                for x in m.values_mut() {
                    walk(x, n);
                }
            }
            J::Array(a) => a.iter_mut().for_each(|x| walk(x, n)),
            _ => {}
        }
    }
    let mut json = serde_json::to_value(&e).ok()?;
    let mut n = 0;
    walk(&mut json, &mut n);
    (n == 1).then(|| serde_json::from_value(json).ok()).flatten()
}

/// "[instruction exiling something] face down": the instruction, with the cards exiled
/// face down (CR 406.3). Only for text with one exile instruction, where "face down"
/// follows it.
fn p_exile_face_down(l: &str, b: &mut Builder) -> Option<Effect> {
    if super::zz_probe_ps::disabled() {
        return None;
    }
    let l = end(l);
    if l.matches(" face down").count() != 1 || l.contains("face-down") || l.contains("face up") {
        return None;
    }
    let (head, tail) = l.split_once(" face down")?;
    if exile_verbs(l) != 1 || exile_verbs(head) != 1 {
        return None;
    }
    if !(tail.is_empty() || tail.starts_with([' ', ','])) {
        return None;
    }
    let text = format!("{head}{tail}");
    let saved = (
        b.targets.len(),
        b.it.clone(),
        b.it_player.clone(),
        b.group.clone(),
    );
    let parsed = parse_clause(&text, b).and_then(exiled_face_down);
    if parsed.is_none() {
        b.targets.truncate(saved.0);
        (b.it, b.it_player, b.group) = (saved.1, saved.2, saved.3);
    }
    parsed
}

inventory::submit! { EffectPattern { name: "face grammar: [instruction] exiling cards face down", priority: 110, parse: p_exile_face_down } }

// ---------------------------------------------------------------------------
// Exiling some of the looked-at cards: "Look at the top four cards of your library.
// Exile one of them face down and put the rest on the bottom of your library in any
// order."
// ---------------------------------------------------------------------------

/// A look at the top cards of a library with nothing taken yet (see `card_flow_dig`).
fn looked_at(e: &mut Effect) -> Option<&mut Effect> {
    match e {
        Effect::Dig {
            take: Value::Const(0),
            rest_to,
            ..
        } if super::card_flow_dig::is_in_place(rest_to) => Some(e),
        _ => None,
    }
}

/// Whether the looked-at library is the controller's own.
fn own_library(e: &Effect) -> bool {
    matches!(
        e,
        Effect::Dig {
            who: PlayerRef::You,
            ..
        }
    )
}

/// Where the rest of the looked-at cards go. Another player's library: only where no
/// order is chosen ("in a random order", one card, a graveyard).
fn rest_to(s: &str, single: bool, own: bool) -> Option<Destination> {
    if own {
        return super::card_flow_dig::rest_destination(s, single);
    }
    let s = s.trim();
    let bottom = |random: bool| {
        let mut d = Destination::library_bottom();
        if random {
            d.position = LibraryPosition::BottomRandom;
        }
        d
    };
    Some(match s {
        "on the bottom of that library" | "on the bottom of their library" if single => {
            bottom(false)
        }
        "on the bottom of that library in a random order"
        | "on the bottom of their library in a random order" => bottom(true),
        "into their graveyard" | "into that player's graveyard" => {
            Destination::zone(ZoneKind::Graveyard)
        }
        _ => return None,
    })
}

/// "exile one [of them] [face down]", "exile two of them face down": how many, whether
/// face down, and the rest.
fn exile_some(l: &str) -> Option<(u32, bool, &str)> {
    let r = l.strip_prefix("exile ")?;
    let (n, r) = parse_number(r)?;
    let n = n.as_const()?;
    if n < 1 {
        return None;
    }
    let r = r.trim_start();
    let r = r
        .strip_prefix("of them")
        .or_else(|| r.strip_prefix("of those cards"))
        .map(str::trim_start)
        .unwrap_or(r);
    // "Exile one" alone only for a single card ("exile one face down").
    let (face_down, r) = match r.strip_prefix("face down") {
        Some(x) => (true, x),
        None => (false, r),
    };
    Some((n as u32, face_down, r))
}

/// "exile one of them face down [and put the rest|other <where>]" after a look at the
/// top cards of a library: the player performing it chooses the cards exiled (CR 406.3:
/// no one may look at them afterwards unless an instruction says so).
fn f_exile_some(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if super::zz_probe_ps::disabled() {
        return false;
    }
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let Some((k, face_down, r)) = exile_some(l) else {
        return false;
    };
    let own = own_library(prev);
    let Some(Effect::Dig {
        n,
        take,
        take_up_to,
        take_to,
        rest_to: rest,
        ..
    }) = looked_at(prev)
    else {
        return false;
    };
    let r = r.trim_start();
    let rest_dest = if r.is_empty() {
        None
    } else {
        let (single, x) = if let Some(x) = r.strip_prefix("and put the other ") {
            (true, x)
        } else if let Some(x) = r
            .strip_prefix("and put the rest ")
            .or_else(|| r.strip_prefix(", then put the rest "))
        {
            (false, x)
        } else {
            return false;
        };
        if single && n.as_const() != Some(k as i32 + 1) {
            return false;
        }
        match rest_to(x, single, own) {
            Some(d) => Some(d),
            None => return false,
        }
    };
    let mut to = Destination::zone(ZoneKind::Exile);
    to.face_down = face_down;
    *take = Value::c(k as i32);
    *take_up_to = false;
    *take_to = to;
    if let Some(d) = rest_dest {
        *rest = d;
    }
    // "You may cast that card for as long as it remains exiled": the exiled card.
    b.it = Sel::Var(vars::IT);
    true
}

inventory::submit! { FollowupPattern { name: "face grammar: exile N of them [face down] [and put the rest ...]", priority: 95, apply: f_exile_some } }

/// "put the rest on the bottom of that library in a random order", "put the rest into
/// their graveyard" after exiling some of the cards looked at in another player's
/// library.
fn f_rest_of_other_library(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if super::zz_probe_ps::disabled() {
        return false;
    }
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let Some(r) = l.strip_prefix("put the rest ") else {
        return false;
    };
    let Some(d) = rest_to(r, false, false) else {
        return false;
    };
    match prev {
        Effect::Dig {
            who,
            take,
            take_to,
            rest_to,
            ..
        } if !matches!(who, PlayerRef::You)
            && super::card_flow_dig::is_in_place(rest_to)
            && !matches!(take, Value::Const(0))
            && take_to.zone == ZoneKind::Exile =>
        {
            *rest_to = d;
            true
        }
        _ => false,
    }
}

inventory::submit! { FollowupPattern { name: "face grammar: put the rest (another player's library)", priority: 95, apply: f_rest_of_other_library } }

/// "look at the top N cards of [library], [instruction][, then instruction]": the look,
/// then what's done with the cards, read as the sentences that would follow it.
fn p_look_then(l: &str, b: &mut Builder) -> Option<Effect> {
    if super::zz_probe_ps::disabled() {
        return None;
    }
    let l = end(l);
    if !l.starts_with("look at the top ") {
        return None;
    }
    let (look, rest) = l.split_once(", ")?;
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let parsed = (|| {
        let mut e = crate::oracle::effects::parse_simple(look, b)?;
        looked_at(&mut e)?;
        let parts: Vec<&str> = match rest.split_once(", then ") {
            Some((a, c)) => vec![a, c],
            None => vec![rest],
        };
        for p in parts {
            if !crate::oracle_ext::apply_followup_ext(p, &mut e, b) {
                return None;
            }
        }
        // Something was taken.
        matches!(&e, Effect::Dig { take, .. } if !matches!(take, Value::Const(0)))
            .then_some(e)
    })();
    if parsed.is_none() {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
    }
    parsed
}

inventory::submit! { EffectPattern { name: "face grammar: look at the top N cards, [instruction], then [instruction]", priority: 110, parse: p_look_then } }

/// "look at the top N cards of that player's library" (the player the text refers to).
fn p_look_at_that_players_library(l: &str, b: &mut Builder) -> Option<Effect> {
    if super::zz_probe_ps::disabled() {
        return None;
    }
    let r = end(l).strip_prefix("look at the top ")?;
    let (n, r) = match r.strip_prefix("card of ") {
        Some(r) => (Value::c(1), r),
        None => {
            let (n, r) = parse_number(r)?;
            n.as_const()?;
            (n, r.strip_prefix("cards of ")?)
        }
    };
    if r != "that player's library" {
        return None;
    }
    let who = b.it_player.clone();
    if super::oracle_hardening_referents::is_no_player_referent(&who)
        || matches!(who, PlayerRef::You)
    {
        return None;
    }
    let mut rest_to = Destination::library_top();
    rest_to.position = LibraryPosition::FromTop(0);
    b.it = Sel::Var(vars::IT);
    Some(Effect::Dig {
        who,
        n,
        reveal: false,
        filter: Filter::Any,
        take: Value::c(0),
        take_up_to: true,
        take_to: Destination::zone(ZoneKind::Hand),
        rest_to,
    })
}

inventory::submit! { EffectPattern { name: "face grammar: look at the top N cards of that player's library", priority: 110, parse: p_look_at_that_players_library } }

// ---------------------------------------------------------------------------
// Turning face up: "turn the exiled card face up", "turn all cards exiled with ~ face
// up", "turn it face up if it's face down", "you may turn a creature you control face
// up".
// ---------------------------------------------------------------------------

/// The cards exiled with the source (CR 607.2a).
fn exiled_with_source() -> Filter {
    Filter::and(vec![
        Filter::In(Box::new(Sel::Linked)),
        Filter::InZone(ZoneKind::Exile),
    ])
}

/// What's turned face up: the source's exiled cards, a referent or target, or a
/// face-down permanent chosen as the instruction is performed ("a creature you control").
fn face_up_objects(s: &str, b: &mut Builder) -> Option<Sel> {
    let s = s.trim();
    match s {
        "the exiled card" | "the exiled cards" | "all cards exiled with ~"
        | "each card exiled with ~" => return Some(Sel::All(exiled_with_source())),
        _ => {}
    }
    if let Some(r) = s.strip_prefix("a ").or_else(|| s.strip_prefix("an ")) {
        let (f, plural, rest) = parse_object_phrase(r)?;
        if plural || !rest.trim().is_empty() || f.zone().is_some() {
            return None;
        }
        // Only a face-down one can be turned face up (CR 708.8); choosing one that's
        // face up would do nothing.
        return Some(Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![
                f,
                Filter::FaceDown,
                Filter::InZone(ZoneKind::Battlefield),
            ]),
            count: Value::c(1),
            up_to: false,
            store: None,
        });
    }
    let (sel, rest) = crate::oracle::effects::object_ref(s, b)?;
    if !rest.trim().is_empty()
        || super::oracle_hardening_referents::is_no_referent(&sel)
        || matches!(sel, Sel::None | Sel::Players(_))
    {
        return None;
    }
    if let Sel::All(f) = &sel {
        if f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
            return None;
        }
    }
    Some(sel)
}

/// "turn [objects] face up [if it's face down]": each face-down permanent is turned face
/// up (CR 708.8; turning a face-up one face up does nothing), and a card exiled face down
/// becomes a face-up exiled card (CR 406.3).
fn p_turn_face_up(l: &str, b: &mut Builder) -> Option<Effect> {
    if super::zz_probe_ps::disabled() {
        return None;
    }
    let l = end(l);
    let r = l.strip_prefix("turn ")?;
    let r = r
        .strip_suffix(" face up if it's face down")
        .or_else(|| r.strip_suffix(" face up"))?;
    if r.starts_with("target ") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone());
    let Some(what) = face_up_objects(r, b) else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    };
    // "If it's a creature card, put it onto the battlefield": the card turned face up.
    if let Sel::All(_) = &what {
        b.it = what.clone();
    }
    Some(Effect::TurnFaceUp { what })
}

inventory::submit! { EffectPattern { name: "face grammar: turn [objects] face up", priority: 110, parse: p_turn_face_up } }
