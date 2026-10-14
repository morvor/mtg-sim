//! "The difference" (CR 107.1b): the amount by which one side of the comparison an
//! earlier condition made exceeds the other.
//!
//! - "If your life total is less than your starting life total, this spell costs {X} less
//!   to cast, where X is the difference." (Shadow of Mortality);
//! - "Whenever a creature dies, if it had power greater than ~'s power, put a number of
//!   +1/+1 counters on ~ equal to the difference." (Drizzt Do'Urden);
//! - "If its controller has more than four cards in hand, they exile cards from their hand
//!   equal to the difference." (Isildur's Fateful Strike);
//! - "if that player lost less than 8 life this turn, they lose life equal to the
//!   difference" (Lolth, Spider Queen's emblem).
//!
//! The comparison is read as text into its two sides ("A is less than B": B minus A), and
//! "the difference" is written as that calculation for the value grammar. The condition
//! guarantees the result is positive where it's used.

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::CompileContext;

/// The two sides of a comparison, greater first, as value phrases.
fn sides(cond: &str) -> Option<(String, String)> {
    let cond = cond.trim();
    // "A is less than B" / "A is greater than B".
    for (w, greater_first) in [(" is less than ", false), (" is greater than ", true)] {
        if let Some((a, b)) = cond.split_once(w) {
            return Some(if greater_first {
                (a.to_string(), b.to_string())
            } else {
                (b.to_string(), a.to_string())
            });
        }
    }
    // "it had power greater than ~'s power", "it has toughness greater than ...".
    for verb in [" had ", " has "] {
        if let Some((subject, r)) = cond.split_once(verb) {
            for stat in ["power", "toughness", "mana value"] {
                if let Some(other) = r.strip_prefix(&format!("{stat} greater than ")) {
                    return Some((possessive(subject, stat)?, other.to_string()));
                }
                if let Some(other) = r.strip_prefix(&format!("{stat} less than ")) {
                    return Some((other.to_string(), possessive(subject, stat)?));
                }
                // "that creature has greater power than ~".
                if let Some(other) = r
                    .strip_prefix(&format!("greater {stat} than "))
                    .and_then(|o| o.strip_suffix("").map(|o| o.to_string()))
                {
                    let other_stat = if other == "~" || other.starts_with("this ") {
                        format!("{other}'s {stat}")
                    } else {
                        possessive(&other, stat)?
                    };
                    return Some((possessive(subject, stat)?, other_stat));
                }
            }
            // "its controller has more than four cards in hand".
            if let Some(n) = r
                .strip_prefix("more than ")
                .and_then(|x| x.strip_suffix(" cards in hand"))
            {
                return Some((
                    format!("the number of cards in {} hand", player_possessive(subject)?),
                    n.to_string(),
                ));
            }
        }
    }
    // "that player lost less than 8 life this turn".
    for (subject, verb) in [("that player", " lost less than "), ("you", " lost less than ")] {
        if let Some(n) = cond
            .strip_prefix(subject)
            .and_then(|r| r.strip_prefix(verb))
            .and_then(|r| r.strip_suffix(" life this turn"))
        {
            let lost = if subject == "you" {
                "the amount of life you lost this turn".to_string()
            } else {
                format!("the amount of life {subject} lost this turn")
            };
            return Some((n.to_string(), lost));
        }
    }
    None
}

/// "it" + "power" → "its power"; "that creature" + "power" → "that creature's power".
fn possessive(subject: &str, stat: &str) -> Option<String> {
    Some(match subject {
        "it" => format!("its {stat}"),
        "~" => format!("~'s {stat}"),
        s if s.starts_with("that ") || s.starts_with("target ") || s.starts_with("the ") => {
            format!("{s}'s {stat}")
        }
        _ => return None,
    })
}

/// "its controller" → "its controller's", "that player" → "that player's", "they" → "their".
fn player_possessive(subject: &str) -> Option<String> {
    Some(match subject {
        "they" => "their".to_string(),
        "you" => "your".to_string(),
        s if s.starts_with("its ") || s.starts_with("that ") || s.starts_with("target ") => {
            format!("{s}'s")
        }
        _ => return None,
    })
}

/// The text with "the difference" written as the calculation the comparison before it
/// describes: the comparison is the condition of the "if" clause the phrase follows.
fn rewrite(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    let d = lower.find("the difference")?;
    let after = &lower[d + "the difference".len()..];
    if after.starts_with(" between") || lower[d + 1..].contains("the difference") {
        return None;
    }
    // The nearest "if " before the phrase, and its condition up to the comma.
    let i = lower[..d].rfind("if ")?;
    let cond_end = i + lower[i..d].find(", ")?;
    let cond = &lower[i + 3..cond_end];
    let (big, small) = sides(cond)?;
    // "If that creature has greater power than ~": the condition is read in the same words
    // as the calculation, so that "that creature" names the same object in both.
    let head = if cond.starts_with("that ") && !cond.starts_with("that player ") {
        format!("{}{big} is greater than {small}{}", &text[..i + 3], &text[cond_end..d])
    } else {
        text[..d].to_string()
    };
    // Keep the original case outside the phrase.
    Some(format!(
        "{head}{big} minus {small}{}",
        &text[d + "the difference".len()..]
    ))
}

fn difference(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if super::referents_results::in_standard() {
        return None;
    }
    if !block.to_lowercase().contains("the difference")
        || super::referents_results::standard_ok(block, ctx)
    {
        return None;
    }
    let text = crate::oracle::strip_ability_word(block.trim());
    let rewritten = rewrite(text)?;
    let v = crate::oracle::parse_ability(&rewritten, ctx)?;
    let ok = v.iter().all(|a| {
        !(super::oracle_hardening_referents::has_no_referent(a)
            || crate::repeat_process::has_stray_repeat(a)
            || super::filters_relational::unresolved(a))
    });
    if !ok {
        return None;
    }
    Some(
        v.into_iter()
            .map(|a| {
                let mut d = (*a).clone();
                d.text = text.to_string();
                std::sync::Arc::new(d)
            })
            .collect(),
    )
}

inventory::submit! { AbilityPattern { name: "referents: the difference (the sides of the comparison before it)", priority: 50, parse: difference } }
