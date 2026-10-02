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
    let l = end(l);
    if l.matches(" face down").count() != 1 || l.contains("face-down") || l.contains("face up") {
        return None;
    }
    // "Look at the top card of that player's library and exile it face down": having
    // looked at it, the player may go on looking at it (`r406_exile.rs`).
    if l.contains("look at ") {
        return None;
    }
    let (head, tail) = l.split_once(" face down")?;
    if exile_verbs(l) != 1 || exile_verbs(head) != 1 {
        return None;
    }
    if !(tail.is_empty() || tail.starts_with([' ', ','])) {
        return None;
    }
    // "Face down" says how the exiled objects are exiled: it follows them directly, not
    // another instruction ("exile ~, then return it to the battlefield face down").
    let verb_at = ["exiles ", "exile "]
        .iter()
        .filter_map(|v| head.rfind(v))
        .max()?;
    let objects = &head[verb_at..];
    if objects.contains(", ") || objects.contains(" and ") || objects.contains("battlefield") {
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
        // Something was done with the cards.
        looked_at(&mut e).is_none().then_some(e)
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

// ---------------------------------------------------------------------------
// Transforming: "transform up to one target Werewolf you control", "transform all
// Humans", "transform any number of Human Werewolves you control" (CR 701.27).
// ---------------------------------------------------------------------------

/// "transform [target phrase / all objects / any number of objects]": each one that can
/// transform does; anything else doesn't (CR 701.27c).
fn p_transform_objects(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("transform ")?;
    let on_battlefield = |f: Filter| {
        if f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
            return None;
        }
        Some(Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]))
    };
    if let Some(x) = r.strip_prefix("all ") {
        let (f, plural, rest) = parse_object_phrase(x)?;
        if !plural || !end(rest).is_empty() {
            return None;
        }
        let f = super::filters_relational::resolve_referent(f, b)?;
        return Some(Effect::Transform {
            what: Sel::All(on_battlefield(f)?),
        });
    }
    if let Some(x) = r.strip_prefix("any number of ") {
        if x.starts_with("target ") {
            return None;
        }
        let (f, _, rest) = parse_object_phrase(x)?;
        if !end(rest).is_empty() {
            return None;
        }
        let f = super::filters_relational::resolve_referent(f, b)?;
        return Some(Effect::Transform {
            what: Sel::Choose {
                chooser: PlayerRef::You,
                filter: on_battlefield(f)?,
                count: Value::c(99),
                up_to: true,
                store: None,
            },
        });
    }
    let targeted = r.starts_with("target ")
        || r.starts_with("up to one target ")
        || r.starts_with("up to one other target ")
        || r.starts_with("another target ");
    if !targeted {
        return None;
    }
    let (spec, tail) = parse_target(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    match &spec.what {
        TargetKind::Object(f) if f.zone().is_none_or(|z| z == ZoneKind::Battlefield) => {}
        _ => return None,
    }
    let text = r[..r.len() - tail.len()].trim().to_string();
    let slot = b.add_target(spec, &text);
    Some(Effect::Transform {
        what: Sel::Target(slot),
    })
}

inventory::submit! { EffectPattern { name: "face grammar: transform [targets / all objects / any number of objects]", priority: 110, parse: p_transform_objects } }

// ---------------------------------------------------------------------------
// Entering face down: "return it to the battlefield face down under your control. It's a
// 2/2 Cyberman artifact creature." (CR 708.2a, 708.3)
// ---------------------------------------------------------------------------

/// Sets `mods` as the listed characteristics of the one face-down battlefield destination
/// in `e`; false unless there's exactly one, with none listed yet.
fn list_face_down_characteristics(e: &mut Effect, mods: &[Modification]) -> bool {
    use serde_json::Value as J;
    fn walk(v: &mut J, mods: &J, n: &mut usize) {
        match v {
            J::Object(m) => {
                let face_down_entry = m.get("zone").and_then(|z| z.as_str()) == Some("Battlefield")
                    && m.get("face_down") == Some(&J::Bool(true))
                    && m
                        .get("with_mods")
                        .is_some_and(|w| w.as_array().is_some_and(|a| a.is_empty()));
                if face_down_entry {
                    m.insert("with_mods".into(), mods.clone());
                    *n += 1;
                }
                for x in m.values_mut() {
                    walk(x, mods, n);
                }
            }
            J::Array(a) => a.iter_mut().for_each(|x| walk(x, mods, n)),
            _ => {}
        }
    }
    let (Ok(mut json), Ok(mods)) = (serde_json::to_value(&*e), serde_json::to_value(mods)) else {
        return false;
    };
    let mut n = 0;
    walk(&mut json, &mods, &mut n);
    if n != 1 {
        return false;
    }
    match serde_json::from_value(json) {
        Ok(x) => {
            *e = x;
            true
        }
        Err(_) => false,
    }
}

/// "It's a 2/2 Cyberman artifact creature.", "They're 5/5 artifact creatures.", "It's a
/// Forest land." after putting cards onto the battlefield face down: the characteristics
/// they have face down instead of a 2/2 creature's (CR 708.2a).
fn f_face_down_entry_listed(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let clause = if let Some(r) = l.strip_prefix("it's ") {
        format!("~ becomes {r}")
    } else if let Some(r) = l.strip_prefix("they're ") {
        format!("~ becomes {r}")
    } else {
        return false;
    };
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let parsed = parse_clause(&clause, b);
    b.targets.truncate(saved.0);
    (b.it, b.it_player) = (saved.1, saved.2);
    let Some(Effect::Modify {
        what: Sel::This,
        mods,
        duration: Duration::Permanent,
    }) = parsed
    else {
        return false;
    };
    list_face_down_characteristics(prev, &mods)
}

inventory::submit! { FollowupPattern { name: "face grammar: face-down entry characteristics listed", priority: 95, apply: f_face_down_entry_listed } }

/// "[put/return objects onto/to the battlefield] face up or face down": the controller of
/// the effect chooses which as it's performed.
fn p_face_up_or_down(l: &str, b: &mut Builder) -> Option<Effect> {
    let head = end(l).strip_suffix(" face up or face down")?;
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let up = parse_clause(head, b);
    let parsed = up.and_then(|up| {
        // Read again from the same starting point (the same target slots).
        let mut again = Builder {
            targets: b.targets[..saved.0].to_vec(),
            it: saved.1.clone(),
            it_player: saved.2.clone(),
            in_trigger: b.in_trigger,
            sentences: b.sentences,
            chosen_creature: b.chosen_creature.clone(),
            group: b.group.clone(),
            named: b.named.clone(),
            ctx: b.ctx,
        };
        let down = parse_clause(&format!("{head} face down"), &mut again)?;
        if again.targets.len() != b.targets.len() {
            return None;
        }
        // The same instruction, entering face down.
        let s = |e: &Effect| serde_json::to_string(e).unwrap_or_default();
        if s(&up) == s(&down) || !s(&down).contains("\"face_down\":true") {
            return None;
        }
        Some(Effect::ChooseOne {
            who: PlayerRef::You,
            options: vec![("Face up".into(), up), ("Face down".into(), down)],
        })
    });
    if parsed.is_none() {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
    }
    parsed
}

inventory::submit! { EffectPattern { name: "face grammar: [enter the battlefield] face up or face down", priority: 110, parse: p_face_up_or_down } }

// ---------------------------------------------------------------------------
// Manifesting and cloaking particular cards (CR 701.40, 701.58): "you manifest the top
// card of that player's library", "cloak a card from your hand", "manifest a number of
// cards from the top of your library equal to ..."
// ---------------------------------------------------------------------------

/// "[you] manifest|cloak [cards]" for cards the core keyword-action grammar doesn't name:
/// the top card of another player's library (the manifested permanent is yours, CR
/// 701.40a), a card from your hand (chosen as the instruction is performed), or a number
/// of cards from the top of your library given by a value.
fn p_manifest_cards(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("you ").unwrap_or(l);
    let (action, r) = if let Some(r) = l
        .strip_prefix("manifest ")
        .or_else(|| l.strip_prefix("manifests "))
    {
        (KeywordAction::Manifest, r)
    } else {
        let r = l.strip_prefix("cloak ").or_else(|| l.strip_prefix("cloaks "))?;
        (KeywordAction::Cloak, r)
    };
    let (what, n) = if let Some(whose) = r.strip_prefix("the top card of ") {
        let who = match whose {
            "that player's library" | "their library" => b.it_player.clone(),
            "target player's library" | "target opponent's library" => {
                let (who, rest) = crate::oracle::effects::player_ref(
                    whose.strip_suffix("'s library")?,
                    b,
                )?;
                if !end(&rest).is_empty() {
                    return None;
                }
                who
            }
            _ => return None,
        };
        if super::oracle_hardening_referents::is_no_player_referent(&who)
            || matches!(who, PlayerRef::You)
        {
            return None;
        }
        (Sel::TopOfLibrary(who, Value::c(1)), Value::c(1))
    } else if r == "a card from your hand" {
        (
            Sel::Choose {
                chooser: PlayerRef::You,
                filter: Filter::and(vec![
                    Filter::Card,
                    Filter::InZone(ZoneKind::Hand),
                    Filter::OwnedBy(PlayerRel::You),
                ]),
                count: Value::c(1),
                up_to: false,
                store: None,
            },
            Value::c(1),
        )
    } else if let Some(x) = r.strip_prefix("a number of cards from the top of your library equal to ")
    {
        let (n, rest) = super::value_grammar::parse_value(x, b)?;
        if !end(&rest).is_empty() {
            return None;
        }
        (Sel::None, n)
    } else {
        return None;
    };
    b.it = Sel::Var(crate::kwa::kvars::MANIFESTED);
    Some(Effect::KeywordAction {
        action,
        who: PlayerRef::You,
        what,
        n,
    })
}

inventory::submit! { EffectPattern { name: "face grammar: manifest / cloak [particular cards]", priority: 110, parse: p_manifest_cards } }

// ---------------------------------------------------------------------------
// Looking at face-down permanents: "look at target face-down creature", "you may look at
// each face-down creature that's attacking or blocking" (CR 708.5: otherwise only their
// controllers may).
// ---------------------------------------------------------------------------

fn p_look_at_face_down(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("you may ").unwrap_or(l);
    let r = l.strip_prefix("look at ")?;
    let what = if r.starts_with("target ") {
        let (spec, tail) = parse_target(r)?;
        if !end(tail).is_empty() {
            return None;
        }
        match &spec.what {
            TargetKind::Object(f) if names_face_down(f) => {}
            _ => return None,
        }
        let text = r[..r.len() - tail.len()].trim().to_string();
        Sel::Target(b.add_target(spec, &text))
    } else {
        let x = r.strip_prefix("each ").or_else(|| r.strip_prefix("all "))?;
        // "... that's attacking or blocking": in combat.
        let (x, in_combat) = match x
            .strip_suffix(" that's attacking or blocking")
            .or_else(|| x.strip_suffix(" that are attacking or blocking"))
        {
            Some(h) => (h, true),
            None => (x, false),
        };
        let (mut f, _, tail) = parse_object_phrase(x)?;
        if !end(tail).is_empty() || !names_face_down(&f) || f.zone().is_some() {
            return None;
        }
        if in_combat {
            f = Filter::and(vec![
                f,
                Filter::Or(vec![Filter::Attacking, Filter::Blocking]),
            ]);
        }
        Sel::All(Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]))
    };
    Some(Effect::Seq(vec![
        Effect::Store {
            var: vars::IT,
            sel: what,
        },
        Effect::Custom(crate::zones::MAY_LOOK_AT_EXILED.into()),
    ]))
}

/// Whether the filter says its objects are face down.
fn names_face_down(f: &Filter) -> bool {
    match f {
        Filter::FaceDown => true,
        Filter::And(v) => v.iter().any(names_face_down),
        _ => false,
    }
}

inventory::submit! { EffectPattern { name: "face grammar: look at face-down permanents", priority: 110, parse: p_look_at_face_down } }

/// "exile a card from the top of your library for each opponent you have" (Wall of
/// Mourning; "face down" is read by [`p_exile_face_down`]): the top N cards.
fn p_exile_top_for_each(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("exile a card from the top of your library for each ")?;
    let n = super::statics::parse_for_each(r, None)?;
    Some(Effect::Exile {
        what: Sel::TopOfLibrary(PlayerRef::You, n),
        face_down: false,
        link: false,
    })
}

inventory::submit! { EffectPattern { name: "face grammar: exile a card from the top of your library for each [...]", priority: 110, parse: p_exile_top_for_each } }

/// "If you do, put all other cards you own exiled with ~ into your hand." (Duplicity)
/// after exiling cards with the source: the cards exiled with it before, not the ones just
/// exiled (CR 607.2a).
fn p_other_exiled_with(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (verb, r) = l.split_once(" all other cards ")?;
    if !matches!(verb, "put" | "return") || !r.contains(" exiled with ~") {
        return None;
    }
    let e = parse_clause(&format!("{verb} all cards {r}"), b)?;
    let Effect::Move {
        what: Sel::All(f),
        to,
    } = e
    else {
        return None;
    };
    Some(Effect::Move {
        what: Sel::All(Filter::and(vec![
            f,
            Filter::not(Filter::In(Box::new(Sel::Var(vars::IT)))),
        ])),
        to,
    })
}

inventory::submit! { EffectPattern { name: "face grammar: put all other cards exiled with ~ ...", priority: 110, parse: p_other_exiled_with } }

/// "turn that creature face up or put a +1/+1 counter on it" (Experimental Lab): the
/// controller of the effect chooses one instruction as it's performed.
fn p_face_up_or(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.starts_with("turn ") {
        return None;
    }
    let (first, second) = l.split_once(" face up or ")?;
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let parsed = (|| {
        let a = p_turn_face_up(&format!("{first} face up"), b)?;
        let c = parse_clause(second, b)?;
        Some(Effect::ChooseOne {
            who: PlayerRef::You,
            options: vec![("Turn it face up".into(), a), ("The other".into(), c)],
        })
    })();
    if parsed.is_none() || b.targets.len() != saved.0 {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    }
    parsed
}

inventory::submit! { EffectPattern { name: "face grammar: turn [object] face up or [instruction]", priority: 110, parse: p_face_up_or } }

/// "manifest one of those cards, then put the other on the top or bottom of your library"
/// (Write into Being), "cloak two of them, and put the rest on the bottom of your library
/// in a random order" (Hide in Plain Sight) after looking at the top cards of your
/// library: the cards are chosen among those looked at, then manifested or cloaked one at
/// a time (CR 701.40, 701.58), and the rest go where the text says.
fn f_manifest_some(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let (action, r) = if let Some(r) = l.strip_prefix("manifest ") {
        (KeywordAction::Manifest, r)
    } else if let Some(r) = l.strip_prefix("cloak ") {
        (KeywordAction::Cloak, r)
    } else {
        return false;
    };
    let Some((k, r)) = parse_number(r) else {
        return false;
    };
    let Some(k) = k.as_const().filter(|k| *k > 0) else {
        return false;
    };
    let Some(r) = r
        .trim_start()
        .strip_prefix("of them")
        .or_else(|| r.trim_start().strip_prefix("of those cards"))
    else {
        return false;
    };
    if !own_library(prev) {
        return false;
    }
    let Some(Effect::Dig { n, .. }) = looked_at(prev) else {
        return false;
    };
    let n = n.as_const();
    let rest = [", then put the other ", ", and put the other ", " and put the other "]
        .iter()
        .find_map(|p| r.strip_prefix(p))
        .map(|x| (true, x))
        .or_else(|| {
            [", then put the rest ", ", and put the rest ", " and put the rest "]
                .iter()
                .find_map(|p| r.strip_prefix(p))
                .map(|x| (false, x))
        });
    let Some((single, x)) = rest else {
        return false;
    };
    if single && n != Some(k + 1) {
        return false;
    }
    let Some(n) = n else {
        return false;
    };
    // The rest are still the top cards of the library: they're put in place as the rest
    // of a look at those cards would be (not moved to another zone).
    let rest_of = |rest_to: Destination| Effect::Dig {
        who: PlayerRef::You,
        n: Value::c(n - k),
        reveal: false,
        filter: Filter::Any,
        take: Value::c(0),
        take_up_to: true,
        take_to: Destination::zone(ZoneKind::Hand),
        rest_to,
    };
    let place_rest = if single && x == "on the top or bottom of your library" {
        Effect::ChooseOne {
            who: PlayerRef::You,
            options: vec![
                ("Top of library".into(), Effect::Noop),
                ("Bottom of library".into(), rest_of(Destination::library_bottom())),
            ],
        }
    } else {
        match super::card_flow_dig::rest_destination(x, single) {
            Some(d) if d.zone == ZoneKind::Library => rest_of(d),
            _ => return false,
        }
    };
    let looked = Filter::and(vec![
        Filter::In(Box::new(Sel::Var(vars::REVEALED))),
        Filter::InZone(ZoneKind::Library),
    ]);
    let dig = std::mem::take(prev);
    *prev = Effect::Seq(vec![
        dig,
        Effect::KeywordAction {
            action,
            who: PlayerRef::You,
            what: Sel::Choose {
                chooser: PlayerRef::You,
                filter: looked,
                count: Value::c(k),
                up_to: false,
                store: None,
            },
            n: Value::c(1),
        },
        place_rest,
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "face grammar: manifest / cloak N of them, then put the rest ...", priority: 95, apply: f_manifest_some } }

/// "the exiled card's owner manifests dread" (Unidentified Hovership): the owner of the
/// card exiled with the source (CR 607.2a, 701.62a).
fn p_exiled_cards_owner_manifests(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("the exiled card's owner ")?;
    let action = match r {
        "manifests dread" => KeywordAction::ManifestDread,
        _ => return None,
    };
    Some(Effect::KeywordAction {
        action,
        who: PlayerRef::OwnerOf(Box::new(Sel::All(exiled_with_source()))),
        what: Sel::None,
        n: Value::c(1),
    })
}

inventory::submit! { EffectPattern { name: "face grammar: the exiled card's owner manifests dread", priority: 110, parse: p_exiled_cards_owner_manifests } }

/// "Put the rest of those cards on the bottom of your library in a random order." after
/// taking some of the looked-at cards (and perhaps doing something with them).
fn f_rest_of_those_cards(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("put the rest of those cards ") else {
        return false;
    };
    let Some(d) = super::card_flow_dig::rest_destination(r, false) else {
        return false;
    };
    // The look itself: the effect, or the first instruction of it.
    let dig = match prev {
        Effect::Seq(v) => v.first_mut(),
        e => Some(e),
    };
    match dig {
        Some(Effect::Dig {
            who: PlayerRef::You,
            take,
            take_to,
            rest_to,
            ..
        }) if super::card_flow_dig::is_in_place(rest_to)
            && !matches!(take, Value::Const(0))
            && take_to.zone != d.zone =>
        {
            *rest_to = d;
            true
        }
        _ => false,
    }
}

inventory::submit! { FollowupPattern { name: "face grammar: put the rest of those cards ...", priority: 95, apply: f_rest_of_those_cards } }

/// "You may look at face-down creatures your opponents control any time." (Found Footage):
/// a static ability letting its controller look at them (CR 708.5 otherwise allows only
/// their controllers).
fn s_look_at_opponents_face_down(
    l: &str,
    text: &str,
    _ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    if end(l) != "you may look at face-down creatures your opponents control any time" {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(
            crate::facedown::LOOK_AT_OPPONENTS_FACE_DOWN.into(),
        ))),
        text,
    )])
}

inventory::submit! { super::StaticPattern { name: "face grammar: you may look at face-down creatures your opponents control", priority: 110, parse: s_look_at_opponents_face_down } }

// ---------------------------------------------------------------------------
// Transforming itself after counting: "Then if there are three or more cards exiled with
// ~, transform it." (Profane Procession), "Then if there are five or more hatchling
// counters on it, remove all of them and transform it." (Ludevic's Test Subject).
// ---------------------------------------------------------------------------

/// "[then] if there are N or more cards exiled with ~, transform it": "it" is the source
/// (a card in exile can't transform, CR 701.27c), not the card just exiled. "[then] if
/// there are N or more [kind] counters on it, remove all of them and transform it": "all
/// of them" are those counters. The sentence, with those words made explicit.
fn if_counted_transform_it(l: &str) -> Option<String> {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let r = l.strip_prefix("if there are ")?;
    let (cond, instr) = r.split_once(", ")?;
    let instr = match instr {
        "transform it" if cond.ends_with(" cards exiled with ~") => "transform ~".to_string(),
        "remove all of them and transform it" => {
            let (_, kind_on) = cond.split_once(" or more ")?;
            let kind = kind_on.strip_suffix(" counters on it")?;
            if kind.contains(' ') {
                return None;
            }
            format!("remove all {kind} counters from it and transform it")
        }
        _ => return None,
    };
    Some(format!("then if there are {cond}, {instr}."))
}

/// The sentence after the instruction that exiled the card or put the counter on.
fn f_if_counted_transform_it(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(s) = if_counted_transform_it(l) else {
        return false;
    };
    // The rewritten sentence as a sentence following `prev` (counters on it), or on its
    // own.
    if crate::oracle_ext::apply_followup_ext(&s, prev, b) {
        return true;
    }
    let Some(e) = crate::oracle::effects::parse_sentence(&s, b) else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "face grammar: if there are N or more [cards exiled with ~ / counters on it], transform it", priority: 110, apply: f_if_counted_transform_it } }

/// "creatures other than Werewolves and Wolves" (Moonmist), "creatures other than
/// Phyrexians" (That's No Moonmist): of none of those subtypes.
fn other_than_subtypes<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let r = t.strip_prefix("other than ")?;
    let stop = r.find([',', '.']).unwrap_or(r.len());
    let (f, plural, tail) = parse_object_phrase(&r[..stop])?;
    let subtypes = match &f {
        Filter::Subtype(_) => true,
        Filter::Or(v) => v.iter().all(|x| matches!(x, Filter::Subtype(_))),
        _ => false,
    };
    if !plural || !subtypes {
        return None;
    }
    let used = stop - tail.len();
    Some((Filter::not(f), &r[used..]))
}

inventory::submit! { super::FilterSuffixPattern { name: "face grammar: other than [subtypes]", priority: 100, parse: other_than_subtypes } }
