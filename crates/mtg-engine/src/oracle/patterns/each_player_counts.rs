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
