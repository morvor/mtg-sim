//! "[You may] have [subject] [verb] ...": the causative form of an effect, as in "you may
//! have target creature get -2/-2 until end of turn", "you may have it fight target
//! creature you don't control", "you may have target player mill two cards". It means
//! the same as the plain clause "[subject] [verb]s ..." (the "you may" is handled by the
//! sentence parser). The clause is rewritten with the verb conjugated and parsed again,
//! so it's supported exactly when the plain clause is.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};

/// Third-person singular forms of verbs that follow "have [subject]".
fn conjugate(verb: &str) -> Option<&'static str> {
    Some(match verb {
        "get" => "gets",
        "fight" => "fights",
        "deal" => "deals",
        "lose" => "loses",
        "gain" => "gains",
        "mill" => "mills",
        "discard" => "discards",
        "draw" => "draws",
        "become" => "becomes",
        "sacrifice" => "sacrifices",
        "create" => "creates",
        "return" => "returns",
        "search" => "searches",
        "put" => "puts",
        "shuffle" => "shuffles",
        "explore" => "explores",
        "connive" => "connives",
        "exile" => "exiles",
        "tap" => "taps",
        "untap" => "untaps",
        "reveal" => "reveals",
        "phase" => "phases",
        _ => return None,
    })
}

fn p_have(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("have ")?;
    // The subject runs up to the first word that is one of the verbs above.
    let words: Vec<&str> = r.split(' ').collect();
    let i = (1..words.len()).find(|&i| conjugate(words[i]).is_some())?;
    let text = format!(
        "{} {} {}",
        words[..i].join(" "),
        conjugate(words[i])?,
        words[i + 1..].join(" ")
    );
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let e = parse_clause(text.trim(), b);
    if e.is_none() {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        b.it_player = saved.2;
    }
    e
}

inventory::submit! { EffectPattern { name: "damage_removal: have [subject] [verb]", priority: 200, parse: p_have } }
