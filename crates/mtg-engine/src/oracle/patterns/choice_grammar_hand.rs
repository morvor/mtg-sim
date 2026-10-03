//! Choosing cards from another player's revealed (or looked-at) hand, and from their
//! graveyard (CR 701.9b, 701.20a, 608.2d), forms the core hand-disruption grammar
//! (`card_flow_reveal_hand`) doesn't read:
//!
//! - "you choose up to X nonland cards from it and exile them";
//! - "you choose from it a nonland card with mana value 3 or less and a card with mana
//!   value 4 or greater" (several choices; "those cards" are all of them);
//! - "you choose a nonland card from that player's graveyard or hand and exile it";
//! - "you choose a nonland card from it or a card from their graveyard";
//! - "you choose an artifact or creature card from it, then choose an artifact or creature
//!   card from their graveyard";
//! - "look at target player's hand and choose up to two cards from it".
//!
//! The cards chosen are `card_flow_reveal_hand::CHOSEN` ("that card", "those cards", "the
//! chosen cards").

use super::card_flow_reveal_hand::CHOSEN;
use super::card_flow_search::card_filter;
use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{player_ref, Builder};
use crate::oracle::phrases::*;

/// The player whose hand the effect revealed or looked at last.
fn hand_owner(e: &Effect) -> Option<PlayerRef> {
    match e {
        Effect::RevealHand { who } | Effect::LookAtHand { who } => Some(who.clone()),
        Effect::Seq(v) => v.last().and_then(hand_owner),
        _ => None,
    }
}

/// Cards of the player's in a zone.
fn in_zone(f: Filter, zone: ZoneKind, owner: &PlayerRef) -> Filter {
    Filter::and(vec![
        f,
        Filter::InZone(zone),
        Filter::OwnedByPlayer(Box::new(owner.clone())),
    ])
}

/// "a nonland card", "up to x nonland cards", "an artifact or creature card": the count,
/// whether it's "up to", and the card filter.
fn counted_cards(s: &str, b: &Builder) -> Option<(Value, bool, Filter)> {
    let s = s.trim();
    let (up_to, s) = match s.strip_prefix("up to ") {
        Some(r) => (true, r),
        None => (false, s),
    };
    let (n, r) = parse_number(s)?;
    let f = card_filter(r, b)?;
    Some((n, up_to, f))
}

fn choose(filter: Filter, count: Value, up_to: bool) -> Sel {
    Sel::Choose {
        chooser: PlayerRef::You,
        filter,
        count,
        up_to,
        store: None,
    }
}

/// What "you choose ..." chooses after the hand of `owner` was revealed: the selection
/// and what follows ("and exile them").
fn hand_choice(r: &str, owner: &PlayerRef, b: &mut Builder) -> Option<(Sel, String)> {
    let hand = |f: Filter| in_zone(f, ZoneKind::Hand, owner);
    let grave = |f: Filter| in_zone(f, ZoneKind::Graveyard, owner);
    // "from it a nonland card with mana value 3 or less and a card with mana value 4 or
    // greater": one choice for each.
    if let Some(list) = r.strip_prefix("from it ") {
        let (list, tail) = split_tail(list);
        let mut sels = Vec::new();
        for part in list.split(" and ") {
            let (n, up_to, f) = counted_cards(part, b)?;
            sels.push(choose(hand(f), n, up_to));
        }
        if sels.len() < 2 {
            return None;
        }
        return Some((Sel::Union(sels), tail.to_string()));
    }
    // "a nonland card from that player's graveyard or hand"
    for zones in [
        " from that player's graveyard or hand",
        " from that player's hand or graveyard",
        " from their graveyard or hand",
        " from their hand or graveyard",
    ] {
        if let Some((desc, tail)) = r.split_once(zones) {
            let (n, up_to, f) = counted_cards(desc, b)?;
            let both = Filter::and(vec![
                f,
                Filter::Or(vec![
                    Filter::InZone(ZoneKind::Hand),
                    Filter::InZone(ZoneKind::Graveyard),
                ]),
                Filter::OwnedByPlayer(Box::new(owner.clone())),
            ]);
            return Some((choose(both, n, up_to), tail.to_string()));
        }
    }
    let (desc, after) = r.split_once(" from it")?;
    let (n, up_to, f) = counted_cards(desc, b)?;
    let after = after.trim_start();
    // "... from it or a card from their graveyard": one card from either.
    if let Some(x) = after.strip_prefix("or ") {
        let (desc2, tail) = x.split_once(" from their graveyard")?;
        let (n2, _, f2) = counted_cards(desc2, b)?;
        if n.as_const() != Some(1) || n2.as_const() != Some(1) || up_to {
            return None;
        }
        let either = Filter::Or(vec![hand(f), grave(f2)]);
        return Some((choose(either, n, false), tail.to_string()));
    }
    // "..., then choose an artifact or creature card from their graveyard"
    if let Some(x) = after.strip_prefix(", then choose ") {
        let (desc2, tail) = x.split_once(" from their graveyard")?;
        let (n2, up2, f2) = counted_cards(desc2, b)?;
        return Some((
            Sel::Union(vec![choose(hand(f), n, up_to), choose(grave(f2), n2, up2)]),
            tail.to_string(),
        ));
    }
    Some((choose(hand(f), n, up_to), after.to_string()))
}

/// A list followed by " and exile them" / " and exile it" / " and exile that card".
fn split_tail(s: &str) -> (&str, &str) {
    for t in [" and exile them", " and exile it", " and exile that card"] {
        if let Some(x) = s.strip_suffix(t) {
            return (x, t);
        }
    }
    (s, "")
}

/// After the choice: nothing, or "and exile them/it/that card".
fn after_choice(tail: &str) -> Option<bool> {
    match end(tail) {
        "" => Some(false),
        "and exile them" | "and exile it" | "and exile that card" => Some(true),
        _ => None,
    }
}

fn with_choice(prev: Effect, sel: Sel, exile: bool) -> Effect {
    let mut v = vec![
        prev,
        Effect::Store {
            var: CHOSEN,
            sel,
        },
    ];
    if exile {
        v.push(Effect::Exile {
            what: Sel::Var(CHOSEN),
            face_down: false,
            link: false,
        });
    }
    Effect::seq(v)
}

/// "You choose [cards] from it ..." after a hand was revealed (the forms the core reads
/// are left to it).
fn choose_from_hand(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = l.strip_prefix("you choose ") else {
        return false;
    };
    let Some(owner) = hand_owner(prev) else {
        return false;
    };
    let saved = b.targets.len();
    let Some((sel, tail)) = hand_choice(r, &owner, b) else {
        b.targets.truncate(saved);
        return false;
    };
    let Some(exile) = after_choice(&tail) else {
        b.targets.truncate(saved);
        return false;
    };
    let old = std::mem::take(prev);
    *prev = with_choice(old, sel, exile);
    b.it = Sel::Var(CHOSEN);
    b.named
        .push(("the chosen cards".into(), Sel::Var(CHOSEN)));
    b.named.push(("those cards".into(), Sel::Var(CHOSEN)));
    b.it_player = owner;
    true
}

inventory::submit! { FollowupPattern { name: "choice grammar: you choose cards from a revealed hand", priority: 95, apply: choose_from_hand } }

/// "look at target player's hand and choose up to two cards from it".
fn look_and_choose(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("look at ")?;
    let (who_text, rest) = r.split_once("'s hand and choose ")?;
    let saved = (b.targets.len(), b.it_player.clone());
    let parsed = (|| {
        let (who, extra) = player_ref(who_text, b)?;
        if !extra.trim().is_empty() || !matches!(who, PlayerRef::Target(_)) {
            return None;
        }
        let (sel, tail) = hand_choice(rest, &who, b)?;
        let exile = after_choice(&tail)?;
        b.it_player = who.clone();
        Some(with_choice(Effect::LookAtHand { who }, sel, exile))
    })();
    match parsed {
        Some(e) => {
            b.it = Sel::Var(CHOSEN);
            b.named.push(("those cards".into(), Sel::Var(CHOSEN)));
            Some(e)
        }
        None => {
            b.targets.truncate(saved.0);
            b.it_player = saved.1;
            None
        }
    }
}

inventory::submit! { EffectPattern { name: "choice grammar: look at a hand and choose cards from it", priority: 90, parse: look_and_choose } }

/// Whether the effect ends with a choice of cards from a hand (see [`with_choice`] and
/// `card_flow_reveal_hand`).
fn ends_with_hand_choice(e: &Effect) -> bool {
    match e {
        Effect::Store { var, .. } => *var == CHOSEN,
        Effect::Seq(v) => v.last().is_some_and(ends_with_hand_choice),
        _ => false,
    }
}

/// "When you do, that player reveals their hand and you choose a nonland card from it.
/// Exile that card." (Biting-Palm Ninja): a sentence about the card chosen in a reflexive
/// triggered ability continues that ability (CR 603.12).
fn reflexive_choice_continues(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !l.contains("that card") || l.contains("target") {
        return false;
    }
    let last = match prev {
        Effect::Seq(v) => match v.last_mut() {
            Some(x) => x,
            None => return false,
        },
        e => e,
    };
    let Effect::If {
        cond: Condition::PrevHappened,
        then,
        otherwise,
    } = last
    else {
        return false;
    };
    if !matches!(**otherwise, Effect::Noop) {
        return false;
    }
    let Effect::Reflexive { body } = &mut **then else {
        return false;
    };
    if !ends_with_hand_choice(&body.effect) {
        return false;
    }
    let mut sub = Builder::new(b.ctx);
    sub.in_trigger = true;
    sub.it = Sel::Var(CHOSEN);
    sub.it_player = b.it_player.clone();
    sub.sentences = 1;
    let Some(e) = crate::oracle::effects::parse_sentence(l, &mut sub) else {
        return false;
    };
    if !sub.targets.is_empty() {
        return false;
    }
    let old = std::mem::take(&mut body.effect);
    body.effect = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "choice grammar: a reflexive ability's chosen card", priority: 85, apply: reflexive_choice_continues } }
