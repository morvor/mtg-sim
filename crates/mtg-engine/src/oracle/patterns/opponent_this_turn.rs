//! Conditions of the Zendikar Traps' alternative costs (CR 118.9): what an opponent did
//! this turn ("an opponent gained life this turn", "an opponent drew three or more cards
//! this turn", "an opponent had two or more creatures enter the battlefield under their
//! control this turn", see `kw/opponent_this_turn.rs`) and how many creatures are
//! attacking ("three or more creatures are attacking", "exactly one creature is
//! attacking").

use super::ConditionPattern;
use crate::ability::*;
use crate::kw::opponent_this_turn::{drew_cards, gained_life, had_enter};
use crate::oracle::phrases::{end, parse_number};
use crate::types::CardType;
use smol_str::SmolStr;

fn custom(name: String) -> Condition {
    Condition::Custom(SmolStr::new(name))
}

/// "N or more" / "a" / "an" as a number, and the rest.
fn at_least(s: &str) -> Option<(u32, &str)> {
    let (n, rest) = parse_number(s)?;
    let n = n.as_const()? as u32;
    let rest = rest.trim_start();
    Some(match rest.strip_prefix("or more ") {
        Some(r) => (n, r),
        None if n == 1 => (1, rest),
        None => return None,
    })
}

fn card_type_word(w: &str) -> Option<CardType> {
    Some(match w.trim_end_matches('s') {
        "artifact" => CardType::Artifact,
        "creature" => CardType::Creature,
        "enchantment" => CardType::Enchantment,
        "land" => CardType::Land,
        "planeswalker" => CardType::Planeswalker,
        _ => return None,
    })
}

fn opponent_this_turn(c: &str) -> Option<Condition> {
    let c = end(c);
    let r = c.strip_prefix("an opponent ")?;
    if r == "gained life this turn" {
        return Some(custom(gained_life()));
    }
    if let Some(r) = r
        .strip_prefix("drew ")
        .and_then(|r| r.strip_suffix(" cards this turn").or(r.strip_suffix(" card this turn")))
    {
        let (n, rest) = at_least(&format!("{r} "))
            .map(|(n, rest)| (n, rest.trim().to_string()))?;
        return rest.is_empty().then(|| custom(drew_cards(n)));
    }
    let r = r
        .strip_prefix("had ")?
        .strip_suffix(" enter the battlefield under their control this turn")?;
    let (n, what) = at_least(r)?;
    let t = card_type_word(what)?;
    Some(custom(had_enter(n, t)))
}

inventory::submit! { ConditionPattern { name: "an opponent [gained life / drew N cards / had N permanents enter] this turn", priority: 100, parse: opponent_this_turn } }

/// "three or more creatures are attacking", "exactly one creature is attacking".
fn creatures_attacking(c: &str) -> Option<Condition> {
    let c = end(c);
    let attacking = Value::Count(Filter::and(vec![Filter::creature(), Filter::Attacking]));
    if let Some(r) = c.strip_prefix("exactly ") {
        let (n, rest) = parse_number(r)?;
        let rest = rest.trim();
        if rest != "creature is attacking" && rest != "creatures are attacking" {
            return None;
        }
        return Some(Condition::Compare(attacking, Cmp::Eq, n));
    }
    let r = c.strip_suffix(" creatures are attacking")?;
    let (n, rest) = parse_number(r)?;
    if rest.trim() != "or more" {
        return None;
    }
    Some(Condition::Compare(attacking, Cmp::Ge, n))
}

inventory::submit! { ConditionPattern { name: "N or more creatures are attacking", priority: 100, parse: creatures_attacking } }
