//! "Each opponent may [effect]. For each opponent who does, [effect]." — the tempting
//! offer ability word (Tempt with Glory, Tempt with Vengeance, ...) and the like (Tempting
//! Contract).
//!
//! The opponents choose in APNAP order whether to do it, each knowing the choices made
//! before theirs (CR 101.4, 101.4b); then each opponent who accepted performs the effect
//! (as the player performing it, "you" in it is that opponent), and after that "for each
//! opponent who does" performs the next effect once per opponent who accepted.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;
use crate::scry_rules::{OPTED, OPT_IN};

/// The opponents who accepted the most recent "each opponent may [effect]".
pub const ACCEPTED: Var = vars::USER + 1911;

/// The effect an opponent performs, worded for that opponent as "you": "put a +1/+1
/// counter on each creature they control" is "put a +1/+1 counter on each creature you
/// control" performed by the opponent.
fn as_performer(text: &str) -> String {
    let mut t = format!(" {text} ");
    for (from, to) in [
        (" they control ", " you control "),
        (" they own ", " you own "),
        (" their ", " your "),
    ] {
        t = t.replace(from, to);
    }
    t.trim().to_string()
}

/// "each opponent may [effect]" (also after "then").
fn each_opponent_may(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("then ").unwrap_or(l);
    let r = r.strip_prefix("each opponent may ")?;
    let text = as_performer(r);
    let saved_targets = b.targets.len();
    let Some(effect) = parse_clause(&text, b) else {
        b.targets.truncate(saved_targets);
        return None;
    };
    Some(Effect::seq(vec![
        // CR 101.4: they decide in APNAP order...
        Effect::Store {
            var: OPTED,
            sel: Sel::None,
        },
        Effect::ForEachPlayer {
            who: PlayerRef::EachOpponent,
            effect: Box::new(Effect::May {
                who: PlayerRef::Iterated,
                effect: Box::new(Effect::Custom(OPT_IN.into())),
            }),
        },
        Effect::Store {
            var: ACCEPTED,
            sel: Sel::Var(OPTED),
        },
        // ...then those who accepted do it.
        Effect::ForEachPlayer {
            who: PlayerRef::Var(ACCEPTED),
            effect: Box::new(Effect::AsPlayer {
                who: PlayerRef::Iterated,
                effect: Box::new(effect),
            }),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "each opponent may [effect]", priority: 150, parse: each_opponent_may } }

/// "for each opponent who does, [effect]" (after "each opponent may [effect]"); "for each
/// opponent who searches a library this way, [effect]" (Tempt with Discovery).
fn for_each_opponent_who_does(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = [
        "for each opponent who does, ",
        "for each opponent who searches a library this way, ",
    ]
    .iter()
    .find_map(|p| l.strip_prefix(p))?;
    let saved_targets = b.targets.len();
    let Some(effect) = parse_clause(r, b) else {
        b.targets.truncate(saved_targets);
        return None;
    };
    Some(Effect::Repeat {
        times: Value::CountSel(Box::new(Sel::Var(ACCEPTED))),
        effect: Box::new(effect),
    })
}

inventory::submit! { EffectPattern { name: "for each opponent who does, [effect]", priority: 150, parse: for_each_opponent_who_does } }

/// "then each player who searched a library this way shuffles" (Tempt with Discovery):
/// you, and each opponent who accepted to search.
fn each_player_who_searched_shuffles(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    if l != "each player who searched a library this way shuffles" {
        return None;
    }
    Some(Effect::seq(vec![
        Effect::Shuffle {
            who: PlayerRef::You,
        },
        Effect::ForEachPlayer {
            who: PlayerRef::Var(ACCEPTED),
            effect: Box::new(Effect::Shuffle {
                who: PlayerRef::Iterated,
            }),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "each player who searched a library this way shuffles", priority: 150, parse: each_player_who_searched_shuffles } }
