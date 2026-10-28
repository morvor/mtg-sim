//! More grammar for dice (CR 706) and coins (CR 705), round 3:
//!
//! - results-table rows that use loyalty symbols as instructions (Comet, Stellar Pup:
//!   "1 or 2 — [+2], then create ...", "4 or 5 — ..., then [−2]"): [+N] puts N loyalty
//!   counters on the source, [−N] removes N (its rulings);
//! - "~ deals N damage to a creature or player": not targeted, the creature or player is
//!   chosen as the instruction is performed (CR 608.2c; Comet's rulings).
//!
//! The behaviors are in `kw/dice_coins_4.rs`.

use super::{ConditionPattern, EffectPattern};
use crate::ability::*;
use crate::kw::dice_coins_4::{choose_creature_or_player, CHOSEN_CREATURE_OR_PLAYER};
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;

/// "[+2]" → "put two loyalty counters on ~", "[−1]" / "[-1]" → "remove a loyalty counter
/// from ~": a loyalty symbol used as an instruction.
fn loyalty_symbol(s: &str) -> Option<(String, &str)> {
    let r = s.strip_prefix('[')?;
    let (sym, rest) = r.split_once(']')?;
    let (add, n) = if let Some(n) = sym.strip_prefix('+') {
        (true, n)
    } else if let Some(n) = sym.strip_prefix('-').or_else(|| sym.strip_prefix('−')) {
        (false, n)
    } else {
        return None;
    };
    let n: u32 = n.parse().ok().filter(|n| *n > 0)?;
    let count = if n == 1 {
        "a loyalty counter".to_string()
    } else {
        format!("{n} loyalty counters")
    };
    Some((
        if add {
            format!("put {count} on ~")
        } else {
            format!("remove {count} from ~")
        },
        rest,
    ))
}

/// A results-table row's text with loyalty symbols written out as instructions: "[+2],
/// then create ..." / "[+1], and you may ..." / "..., then [−2]" (Comet, Stellar Pup).
/// "him" / "her" in such a row is the source. `None` if the row has no loyalty symbol.
pub fn row_with_loyalty_symbols(t: &str) -> Option<String> {
    let t = t.trim();
    if let Some((first, rest)) = loyalty_symbol(t) {
        let rest = rest.trim_start();
        if rest.is_empty() || rest == "." {
            return Some(format!("{first}."));
        }
        let joined = rest
            .strip_prefix(", then ")
            .map(|r| format!("{first}, then {r}"))
            .or_else(|| {
                rest.strip_prefix(", and ")
                    .map(|r| format!("{first}, and {r}"))
            })?;
        return Some(joined);
    }
    let body = t.strip_suffix('.').unwrap_or(t);
    let (head, sym) = body.rsplit_once(", then ")?;
    let (last, tail) = loyalty_symbol(sym)?;
    if !tail.trim().is_empty() {
        return None;
    }
    let head = head
        .replace(" on him ", " on ~ ")
        .replace(" on her ", " on ~ ");
    Some(format!("{head}, then {last}."))
}

/// "~ deals 2 damage to a creature or player" / "~ deals damage equal to [value] to a
/// creature or player": the controller chooses a creature or player as the instruction is
/// performed (not a target, CR 115.1), and it's dealt that much damage.
fn damage_to_a_creature_or_player(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("~ deals ")?;
    let amount = if let Some(v) = r.strip_prefix("damage equal to ") {
        let (v, rest) = crate::oracle::statics::parse_value_phrase(v, b)?;
        if rest.trim() != "to a creature or player" {
            return None;
        }
        v
    } else {
        let (n, rest) = parse_number(r)?;
        if rest.trim() != "damage to a creature or player" {
            return None;
        }
        n
    };
    Some(Effect::seq(vec![
        choose_creature_or_player(),
        Effect::DealDamage {
            source: Sel::This,
            amount,
            to: Sel::Var(CHOSEN_CREATURE_OR_PLAYER),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "dice coins 4: ~ deals damage to a creature or player", priority: 100, parse: damage_to_a_creature_or_player } }

/// "If any of those results was 10 or higher" (after "whenever you roll one or more
/// dice"): one of the results of the dice rolled together that triggered the ability, or
/// of the ability's own roll.
fn any_of_those_results(c: &str) -> Option<Condition> {
    let r = end(c)
        .strip_prefix("any of those results was ")
        .or_else(|| end(c).strip_prefix("any of those results were "))?;
    let n: i64 = r.strip_suffix(" or higher")?.parse().ok()?;
    Some(Condition::Custom(
        format!("{}:{n}", crate::dice::ANY_RESULT_AT_LEAST).into(),
    ))
}

inventory::submit! { ConditionPattern { name: "dice coins 4: any of those results was N or higher", priority: 100, parse: any_of_those_results } }

/// "For each different result, create a 1/1 ... token." (Circuits Act): once for each
/// different result of the roll.
fn for_each_different_result(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let head = l
        .strip_prefix("for each different result, ")
        .map(str::to_string)
        .or_else(|| {
            l.strip_suffix(" for each different result")
                .map(str::to_string)
        })?;
    let e = parse_clause(&head, b)?;
    Some(Effect::Repeat {
        times: Value::Var(crate::dice::DIFFERENT),
        effect: Box::new(e),
    })
}

inventory::submit! { EffectPattern { name: "dice coins 4: for each different result", priority: 100, parse: for_each_different_result } }

/// "Return up to X cards from among cards exiled with ~ to their owners' hands" (Bag of
/// Devouring): the same as "up to X cards exiled with ~".
fn from_among_cards_exiled_with(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.contains(" from among cards exiled with ~") {
        return None;
    }
    parse_clause(
        &l.replace(" from among cards exiled with ~", " exiled with ~"),
        b,
    )
}

inventory::submit! { EffectPattern { name: "dice coins 4: cards from among cards exiled with ~", priority: 100, parse: from_among_cards_exiled_with } }

/// "Put a creature card from any graveyard onto the battlefield under your control." (The
/// Deck of Many Things): the same as "from a graveyard" (any player's).
fn from_any_graveyard(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let i = l.find(" card from any graveyard ")?;
    if !l[..i].starts_with("put ") && !l[..i].starts_with("return ") {
        return None;
    }
    parse_clause(
        &l.replacen(" card from any graveyard ", " card from a graveyard ", 1),
        b,
    )
}

inventory::submit! { EffectPattern { name: "dice coins 4: a card from any graveyard", priority: 100, parse: from_any_graveyard } }

/// The greatest result an instruction "If the result is N or less, [effect]." covers.
fn result_at_most(e: &Effect) -> Option<i64> {
    let Effect::If {
        cond: Condition::Compare(Value::Custom(v), cmp, Value::Const(n)),
        otherwise,
        ..
    } = e
    else {
        return None;
    };
    if v.as_str() != crate::dice::THE_RESULT || !matches!(**otherwise, Effect::Noop) {
        return None;
    }
    match cmp {
        Cmp::Le => Some(i64::from(*n)),
        Cmp::Lt => Some(i64::from(*n) - 1),
        _ => None,
    }
}

/// "Roll a d20 and subtract the number of cards in your hand. If the result is 0 or less,
/// discard your hand. / 1—9 | ..." (The Deck of Many Things): a results table after
/// sentences about results it doesn't cover belongs to the roll before them. The table's
/// effects and those sentences' never both happen, so their order doesn't matter; they're
/// joined into one instruction for the table's further rows and sentences.
pub fn table_after_sentences(s: &str, effects: &mut Vec<Effect>, b: &mut Builder) -> bool {
    let lower = s.to_lowercase();
    let Some((lo, _, _)) = super::r705_706_dice::table_row(&lower) else {
        return false;
    };
    let Some(k) = effects
        .iter_mut()
        .rposition(|e| super::r705_706_dice::last_roll(e).is_some())
    else {
        return false;
    };
    if k + 1 >= effects.len()
        || !effects[k + 1..]
            .iter()
            .all(|e| result_at_most(e).is_some_and(|n| n < lo))
    {
        return false;
    }
    let mut joined = Effect::seq(effects[k..].to_vec());
    if !crate::oracle_ext::apply_followup_ext(s, &mut joined, b) {
        return false;
    }
    effects.truncate(k);
    effects.push(joined);
    true
}
