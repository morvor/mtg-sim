//! "Each opponent loses 1 life for each creature card in their graveyard", "~ deals
//! damage to each player equal to the number of lands they control", "Each player draws
//! a card for each creature card in their graveyard": an instruction for each of several
//! players whose amount depends on that player ("they", "their", "that player"). The
//! instruction is performed for each of them in turn (APNAP order, CR 101.4), with the
//! amount determined for that player.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};

/// Whether the text after the players mentions one of them in an amount.
fn mentions_them_in_amount(rest: &str) -> bool {
    let amount = [" for each ", " equal to ", "where x is "]
        .iter()
        .filter_map(|p| rest.find(p).map(|i| &rest[i..]))
        .next();
    let Some(amount) = amount else {
        return false;
    };
    let words: Vec<&str> = amount
        .split(|c: char| !(c.is_alphanumeric() || c == '\''))
        .collect();
    words
        .iter()
        .any(|w| matches!(*w, "they" | "their" | "them"))
        || amount.contains("that player")
}

fn each_player_counts(l: &str, b: &mut Builder) -> Option<Effect> {
    let players = [
        ("each opponent", PlayerRef::EachOpponent),
        ("each other player", PlayerRef::EachOtherPlayer),
        ("each player", PlayerRef::EachPlayer),
    ];
    for (p, who) in players {
        let Some(i) = l.find(p) else { continue };
        // A whole word, once.
        let after = &l[i + p.len()..];
        if !(after.is_empty() || after.starts_with([' ', ',', '\''])) || after.contains(p) {
            continue;
        }
        if !mentions_them_in_amount(after) || l.contains("of their choice") {
            continue;
        }
        let text = format!("{}that player{}", &l[..i], after);
        let saved = b.it_player.clone();
        b.it_player = PlayerRef::Iterated;
        let e = parse_clause(&text, b);
        b.it_player = saved;
        let e = e?;
        // The amount must have been read for that player.
        let json = serde_json::to_string(&e).ok()?;
        if !json.contains("Iterated") {
            return None;
        }
        return Some(Effect::ForEachPlayer {
            who,
            effect: Box::new(e),
        });
    }
    None
}

inventory::submit! { EffectPattern { name: "each player: amount depending on that player", priority: 65, parse: each_player_counts } }

/// "defending player loses 1 life for each card in their graveyard", "it deals damage to
/// defending player equal to the number of artifacts they control": "they" is the
/// defending player the instruction names.
fn defending_player_amount(l: &str, b: &mut Builder) -> Option<Effect> {
    const P: &str = "defending player";
    let i = l.find(P)?;
    let after = &l[i + P.len()..];
    if after.contains(P)
        || matches!(b.it_player, PlayerRef::DefendingPlayer)
        || !mentions_them_in_amount(after)
        || l.contains("of their choice")
    {
        return None;
    }
    let saved = std::mem::replace(&mut b.it_player, PlayerRef::DefendingPlayer);
    let e = parse_clause(l, b);
    if e.is_none() {
        b.it_player = saved;
    }
    e
}

inventory::submit! { EffectPattern { name: "defending player: amount depending on that player", priority: 65, parse: defending_player_amount } }

/// The last instruction of an effect (looking into sequences).
fn last(e: &Effect) -> &Effect {
    match e {
        Effect::Seq(v) => v.last().map_or(e, last),
        e => e,
    }
}

/// "Target opponent reveals their hand. You draw a card for each Mountain and red card in
/// it.": "it" is the revealed hand.
fn counted_in_revealed_hand(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Effect::RevealHand { who } = last(prev).clone() else {
        return false;
    };
    let Some(head) = l.strip_suffix(" in it") else {
        return false;
    };
    if !head.contains(" for each ") && !head.contains(" equal to ") {
        return false;
    }
    let saved = std::mem::replace(&mut b.it_player, who);
    let Some(e) = crate::oracle::effects::parse_clause(&format!("{head} in their hand"), b)
    else {
        b.it_player = saved;
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { super::FollowupPattern { name: "count cards in the revealed hand", priority: 60, apply: counted_in_revealed_hand } }
