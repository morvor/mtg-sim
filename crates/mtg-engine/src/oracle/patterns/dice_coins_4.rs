//! More grammar for dice (CR 706) and coins (CR 705), round 3:
//!
//! - results-table rows that use loyalty symbols as instructions (Comet, Stellar Pup:
//!   "1 or 2 — [+2], then create ...", "4 or 5 — ..., then [−2]"): [+N] puts N loyalty
//!   counters on the source, [−N] removes N (its rulings);
//! - "~ deals N damage to a creature or player": not targeted, the creature or player is
//!   chosen as the instruction is performed (CR 608.2c; Comet's rulings).
//!
//! The behaviors are in `kw/dice_coins_4.rs`.

use super::EffectPattern;
use crate::ability::*;
use crate::kw::dice_coins_4::{choose_creature_or_player, CHOSEN_CREATURE_OR_PLAYER};
use crate::oracle::effects::Builder;
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
