//! Putting cards from a hand onto the battlefield: "You may put a land card from your hand
//! onto the battlefield.", "put up to two creature cards from your hand onto the
//! battlefield", "that player may put an artifact, creature, or land card from their hand
//! onto the battlefield", "... onto the battlefield tapped [and attacking]".
//!
//! The player chooses the cards as the instruction is performed; they enter under that
//! player's control (CR 110.2a). Putting a land onto the battlefield this way isn't
//! playing it (CR 305.4). "That creature" afterwards is the permanent the card became
//! (CR 400.7).

use super::card_flow_search::card_filter;
use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{player_ref, Builder};
use crate::oracle::phrases::*;

/// The permanents the cards put onto the battlefield became.
const PUT: Var = vars::USER + 1400;

inventory::submit! {
    EffectPattern { name: "sweep: put cards from your hand onto the battlefield", priority: 95, parse: put_from_hand }
}
inventory::submit! {
    EffectPattern { name: "sweep: [player] may put cards from their hand onto the battlefield", priority: 95, parse: player_may_put_from_hand }
}
inventory::submit! {
    FollowupPattern { name: "sweep: delayed removal after putting from hand", priority: 45, apply: delayed_after_put }
}

/// "a [card]", "up to two [cards]", "any number of [cards]": (count, up to, description).
fn hand_count<'a>(r: &'a str, who: &PlayerRef) -> Option<(Value, bool, &'a str)> {
    if let Some(x) = r.strip_prefix("any number of ") {
        return Some((Value::HandSize(who.clone()), true, x));
    }
    if let Some(x) = r.strip_prefix("up to ") {
        let (n, x) = parse_number(x)?;
        n.as_const()?;
        return Some((n, true, x.trim_start()));
    }
    let x = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    Some((Value::Const(1), false, x))
}

/// The hand a player's cards come from, as a filter.
fn owner_rel(who: &PlayerRef) -> Option<PlayerRel> {
    Some(match who {
        PlayerRef::You => PlayerRel::You,
        PlayerRef::Target(s) => PlayerRel::Target(*s),
        PlayerRef::TriggerPlayer => PlayerRel::TriggerPlayer,
        PlayerRef::DefendingPlayer => PlayerRel::Defending,
        _ => return None,
    })
}

/// "[count] [cards] from [your/their] hand onto the battlefield [tapped] [and attacking]",
/// for the player `who` (who chooses them and controls the permanents).
fn from_hand(r: &str, hand: &str, who: PlayerRef, b: &mut Builder) -> Option<Effect> {
    let rel = owner_rel(&who)?;
    let (count, up_to, r) = hand_count(r, &who)?;
    let (desc, tail) = r.split_once(&format!(" from {hand} hand onto the battlefield"))?;
    // "a creature card and/or a land card" (one of each) is a different instruction.
    if desc.contains(" and/or ") {
        return None;
    }
    let filter = card_filter(desc, b)?;
    let mut to = Destination::battlefield();
    to.controller = Some(who.clone());
    match tail.trim() {
        "" => {}
        "tapped" => to.tapped = true,
        "tapped and attacking" => {
            to.tapped = true;
            to.attacking = true;
        }
        _ => return None,
    }
    // "That creature gains haste.": the permanent it became.
    // Kept in a variable of its own, so later instructions ("That creature gains haste.")
    // don't change what it refers to.
    b.it = Sel::Var(PUT);
    Some(Effect::seq(vec![
        Effect::Move {
            what: Sel::Choose {
                chooser: who,
                filter: Filter::and(vec![
                    filter,
                    Filter::Card,
                    Filter::InZone(ZoneKind::Hand),
                    Filter::OwnedBy(rel),
                ]),
                count,
                up_to,
                store: None,
            },
            to,
        },
        Effect::Store {
            var: PUT,
            sel: Sel::Var(vars::IT),
        },
    ]))
}

/// "Sacrifice it at the beginning of the next end step.", "At the beginning of the next
/// end step, sacrifice that creature." after putting a card onto the battlefield from a
/// hand (Sneak Attack): a delayed triggered ability (CR 603.7) that refers to the
/// permanent the card became (CR 400.7). If no card was put onto the battlefield, the
/// delayed ability does nothing.
fn delayed_after_put(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    // "It" still names what an earlier sentence put onto the battlefield.
    if !matches!(b.it, Sel::Var(PUT)) {
        return false;
    }
    let Some((verb, r, step)) = super::damage_removal::delayed_parts(l) else {
        return false;
    };
    let Some(tail) = [
        "it",
        "that creature",
        "the creature",
        "that artifact",
        "that permanent",
    ]
    .iter()
    .find_map(|p| {
        let x = r.strip_prefix(p)?;
        (x.is_empty() || x.starts_with(' ')).then_some(x)
    }) else {
        return false;
    };
    let Some(e) = super::damage_removal::delayed_removal(verb, Sel::Var(PUT), tail.trim(), step)
    else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

/// "put a land card from your hand onto the battlefield"; also "you may put ..." as the
/// second half of a clause ("draw a card, then you may put a land card from your hand
/// onto the battlefield"), where the sentence parser doesn't see the "you may".
fn put_from_hand(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if let Some(r) = l.strip_prefix("you may put ") {
        return Some(Effect::May {
            who: PlayerRef::You,
            effect: Box::new(from_hand(r, "your", PlayerRef::You, b)?),
        });
    }
    let r = l.strip_prefix("put ")?;
    from_hand(r, "your", PlayerRef::You, b)
}

/// "that player may put an artifact, creature, or land card from their hand onto the
/// battlefield".
fn player_may_put_from_hand(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if l.starts_with("you ") || l.starts_with("each ") {
        return None;
    }
    let saved = b.targets.len();
    let (who, rest) = player_ref(l, b)?;
    let Some(r) = rest.trim_start().strip_prefix("may put ") else {
        b.targets.truncate(saved);
        return None;
    };
    let Some(e) = from_hand(r, "their", who.clone(), b) else {
        b.targets.truncate(saved);
        return None;
    };
    Some(Effect::May {
        who,
        effect: Box::new(e),
    })
}
