//! "X can't be greater than [value]." (CR 107.3a, 601.2b): the greatest value that may be
//! announced for X in a spell's cost, determined as X is announced (Open the Way ruling:
//! "The number of players in the game is checked only as you choose the value for X").
//! A line of its own ("X can't be greater than the number of players in the game.", Open
//! the Way) or a sentence after the spell's additional cost ("As an additional cost to
//! cast this spell, blight X. X can't be greater than the greatest toughness among
//! creatures you control.", Soul Immolation).

use super::costs_casting_self::this_spell_cost_ability;
use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

/// "x can't be greater than [value]" (lowercase, with or without its period).
fn cap(s: &str, ctx: &CompileContext) -> Option<Value> {
    let r = end(s.trim()).strip_prefix("x can't be greater than ")?;
    let mut b = Builder::new(ctx);
    let (v, rest) = super::value_grammar::parse_value(r, &mut b)?;
    if !rest.trim().is_empty() || !b.targets.is_empty() || matches!(v, Value::X) {
        return None;
    }
    Some(v)
}

fn rule(v: Value, text: &str) -> Ability {
    this_spell_cost_ability(CostChange::Rule(CostRule::XAtMost(Box::new(v))), None, text)
}

/// "X can't be greater than the number of players in the game." as a line of a spell.
fn cap_line(text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if text.contains('\n') || !ctx.is_spell() {
        return None;
    }
    let v = cap(&text.to_lowercase(), ctx)?;
    Some(vec![rule(v, text)])
}

inventory::submit! { AbilityPattern { name: "X can't be greater than [value] (a spell's rule)", priority: 80, parse: cap_line } }

/// "[the spell's ability]. X can't be greater than [value].": the ability, and the rule.
fn trailing_cap(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if block.contains('\n') || !ctx.is_spell() {
        return None;
    }
    let lower = block.to_lowercase();
    let i = lower.find(". x can't be greater than ")?;
    let head = &block[..i + 1];
    let v = cap(&lower[i + 2..], ctx)?;
    let mut out = crate::oracle::parse_ability(head, ctx)?;
    if out.is_empty() {
        return None;
    }
    out.push(rule(v, &block[i + 2..]));
    Some(out)
}

inventory::submit! { AbilityPattern { name: "[ability]. X can't be greater than [value]. (a spell's rule)", priority: 80, parse: trailing_cap } }
