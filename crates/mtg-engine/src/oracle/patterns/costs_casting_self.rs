//! A spell's own cost changes (CR 601.2f, 118.7): "This spell costs {1} less to cast for
//! each creature card in your graveyard.", "This spell costs {2} less to cast if it
//! targets a tapped creature.", "If you control a Wizard, this spell costs {1} less to
//! cast.", "This spell costs {X} less to cast, where X is the greatest power among
//! creatures you control.", "During your turn, this spell costs {1}{U}{U} less to cast."
//!
//! The change is a static ability of the spell itself ([`CostTarget::ThisSpell`]) that
//! applies while the total cost is determined; a condition ("if ...", "as long as ...")
//! is the static ability's condition, checked at that time (the spell's targets, modes
//! and additional costs are already chosen then, CR 601.2b–c).

use super::AbilityPattern;
use crate::ability::*;
use crate::mana::{ManaCost, ManaSymbol};
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::statics::{parse_condition, parse_value_phrase};
use crate::oracle::CompileContext;

/// A static ability of the spell that changes its own cost, with an optional condition.
pub(crate) fn this_spell_cost_ability(
    change: CostChange,
    condition: Option<Condition>,
    text: &str,
) -> Ability {
    let mut s = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::ThisSpell,
        who: PlayerRel::You,
        change,
    }));
    s.condition = condition;
    s.zone = FunctionZone::Anywhere;
    AbilityDef::new(AbilityKind::Static(s), text)
}

/// "it targets a tapped creature": the spell has a target matching the filter (judged
/// once targets are chosen, CR 601.2c).
fn targets_condition(c: &str) -> Option<Condition> {
    let r = c.strip_prefix("it targets ")?;
    // One matching target ("if it targets two creatures" would need a count).
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (f, _, tail) = parse_object_phrase(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Condition::SelMatches(
        Sel::This,
        Filter::Targets(Box::new(f)),
    ))
}

/// A condition on a cost change: "it targets ...", "during your turn", or any condition
/// the compiler understands.
pub(crate) fn cost_condition(c: &str, ctx: &CompileContext) -> Option<Condition> {
    let c = end(c);
    match c {
        "during your turn" => return Some(Condition::YourTurn),
        "during turns other than yours" => return Some(Condition::NotYourTurn),
        _ => {}
    }
    if let Some(cond) = targets_condition(c) {
        return Some(cond);
    }
    let c = c
        .strip_prefix("if ")
        .or_else(|| c.strip_prefix("as long as "))?;
    if let Some(cond) = targets_condition(c) {
        return Some(cond);
    }
    // Spells cast before this one (it isn't cast until its costs are paid, CR 601.2i).
    let custom = |n: &str| Some(Condition::Custom(n.into()));
    match c {
        "you've cast another spell this turn" => {
            return custom(crate::spell_costs::CAST_ANOTHER_SPELL)
        }
        "you've cast another instant or sorcery spell this turn"
        | "you've cast an instant or sorcery spell this turn" => {
            return custom(crate::spell_costs::CAST_ANOTHER_INSTANT_OR_SORCERY)
        }
        _ => {}
    }
    parse_condition(c, ctx)
}

/// "{2}{U}" at the start of `s`: the mana and the rest.
fn leading_mana(s: &str) -> Option<(ManaCost, &str)> {
    let mut i = 0;
    let b = s.as_bytes();
    while i < b.len() && b[i] == b'{' {
        i += s[i..].find('}')? + 1;
    }
    if i == 0 {
        return None;
    }
    Some((ManaCost::parse(&s[..i])?, &s[i..]))
}

/// What's counted by "for each [...]" in a spell's own cost change.
fn for_each_value(s: &str) -> Option<Value> {
    let s = end(s);
    match s {
        // Strive (an ability word, CR 207.2c): counted once targets are chosen.
        "target beyond the first" => {
            return Some(Value::Custom(
                crate::spell_costs::TARGETS_BEYOND_FIRST.into(),
            ))
        }
        // CR 700.4: "dies" means put into a graveyard from the battlefield.
        "creature that died this turn" => return Some(Value::CreaturesDiedThisTurn),
        "card you've drawn this turn" => return Some(Value::CardsDrawnThisTurn(PlayerRef::You)),
        _ => {}
    }
    // Things that happened this turn, or other qualities the object phrase parser would
    // read loosely, aren't counted here.
    if s.contains(" this turn")
        || s.contains(" this way")
        || s.contains("target")
        || s.contains("spell")
        || s.contains("sacrificed")
    {
        return None;
    }
    super::statics::parse_for_each(s, Some(&Sel::This))
}

/// The change "{amount} less/more", repeated `times` when given.
fn cost_change(mana: ManaCost, more: bool, times: Option<Value>) -> Option<CostChange> {
    let generic_only = mana
        .symbols
        .iter()
        .all(|s| matches!(s, ManaSymbol::Generic(_)));
    let n = mana.generic_amount() as i32;
    let scaled = |v: Value| match &times {
        Some(t) if v.as_const() == Some(1) => t.clone(),
        Some(t) => Value::Mul(Box::new(v), Box::new(t.clone())),
        None => v,
    };
    if generic_only {
        return Some(if more {
            CostChange::IncreaseGeneric(scaled(Value::c(n)))
        } else {
            CostChange::ReduceGeneric(scaled(Value::c(n)))
        });
    }
    match (more, times) {
        (true, None) => Some(CostChange::IncreaseMana(mana)),
        (false, None) => Some(CostChange::ReduceMana {
            mana,
            colored_only: false,
        }),
        // "{G} less for each ...": one colored symbol, repeated.
        (false, Some(t)) => match mana.symbols.as_slice() {
            [ManaSymbol::Colored(c)] => Some(CostChange::ReduceColored(*c, t)),
            _ => None,
        },
        // "{1}{G} more for each ...": the mana, repeated (added to the total before
        // reductions apply).
        (true, Some(t)) => Some(CostChange::AdditionalCost(Cost::free().with(
            CostPart::Repeated {
                cost: Box::new(Cost::mana(mana)),
                times: t,
            },
        ))),
    }
}

/// "~ costs {1} less to cast [for each ... | if ... | as long as ... | during your turn |
/// , where X is ...]", "If [condition], ~ costs {1} less to cast.", "During your turn,
/// ~ costs {1} less to cast."
fn own_cost_change(text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if text.contains('\n') {
        return None;
    }
    let lower = text.to_lowercase();
    let l = end(&lower);
    // One sentence only ("It also costs {1} less ..." is a second change).
    if l.contains(". ") {
        return None;
    }
    // A leading condition: "If you control a Wizard, ~ costs ...".
    let (pre, body) = match l.find("~ costs ") {
        Some(0) => (None, l),
        Some(i) => {
            let head = l[..i].trim_end().strip_suffix(',')?;
            (Some(cost_condition(head, ctx)?), &l[i..])
        }
        None => return None,
    };
    let r = body.strip_prefix("~ costs ")?;
    let (mana, r) = leading_mana(r)?;
    let r = r.trim_start();
    let (more, tail) = if let Some(t) = r.strip_prefix("less to cast") {
        (false, t)
    } else if let Some(t) = r.strip_prefix("more to cast") {
        (true, t)
    } else {
        return None;
    };
    let tail = tail.trim();
    let is_x = mana.symbols.as_slice() == [ManaSymbol::X];
    if mana.has_x() && !is_x {
        return None;
    }
    let (change, post) = if is_x {
        // "{X} less to cast, where X is [value]".
        let v = tail.strip_prefix(", where x is ")?;
        let (v, rest) = parse_value_phrase(v, &mut Builder::new(ctx))?;
        if !end(&rest).is_empty() {
            return None;
        }
        let change = if more {
            CostChange::IncreaseGeneric(v)
        } else {
            CostChange::ReduceGeneric(v)
        };
        (change, None)
    } else if let Some(fe) = tail.strip_prefix("for each ") {
        (cost_change(mana, more, Some(for_each_value(fe)?))?, None)
    } else if tail.is_empty() {
        (cost_change(mana, more, None)?, None)
    } else {
        (cost_change(mana, more, None)?, Some(cost_condition(tail, ctx)?))
    };
    let cond = match (pre, post) {
        (Some(a), Some(b)) => Some(Condition::And(vec![a, b])),
        (a, b) => a.or(b),
    };
    // A cost change with no condition and nothing to count is still a change ("~ costs
    // {1} less to cast" granted by another effect).
    Some(vec![this_spell_cost_ability(change, cond, text)])
}

inventory::submit! { AbilityPattern { name: "costs_casting: this spell costs less/more", priority: 80, parse: own_cost_change } }
