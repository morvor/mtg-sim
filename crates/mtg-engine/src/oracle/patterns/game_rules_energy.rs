//! Energy (CR 107.14, 122.1): amounts of {E} spelled out ("you get six {E}", "you may pay
//! eight {E}", "Equip—Pay {E}{E}"), payments of a chosen amount ("you may pay any amount
//! of {E}", "you may pay one or more {E}") and what refers to it ("that much", "the amount
//! of {E} paid this way", "for each {E} paid this way"), "you may pay an amount of {E}
//! equal to [value]", a mandatory payment ("Pay {E}{E}. If you can't, ..."), and "you
//! may get {E}{E}".
//!
//! Paying energy is removing that many energy counters from yourself (CR 107.14): a
//! player can't pay more than they have.

use super::{CostPattern, EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::kw::game_rules as gr;
use crate::kw::spell_cost_grammar as rules;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::*;
use crate::types::counters;

/// A run of energy symbols: "{e}{e}{e}" → 3 and the rest.
pub fn energy_symbols(s: &str) -> Option<(i32, &str)> {
    let mut n = 0;
    let mut r = s.trim_start();
    while let Some(x) = r.strip_prefix("{e}") {
        n += 1;
        r = x;
    }
    (n > 0).then_some((n, r))
}

/// An amount of energy: "{e}{e}", "six {e}", "x {e}".
fn energy_amount(s: &str) -> Option<(Value, &str)> {
    if let Some((n, r)) = energy_symbols(s) {
        return Some((Value::c(n), r));
    }
    let (v, r) = parse_number(s)?;
    // "a" / "an" aren't amounts of energy here ("an amount of {E}").
    if s.trim_start().starts_with('a') {
        return None;
    }
    let r = r.trim_start().strip_prefix("{e}")?;
    Some((v, r))
}

/// "you get six {E}", "you get X {E}", and "get {E}{E}" after "you may" ("You may get
/// {E}{E}. If you don't, ...").
fn get_energy(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l
        .strip_prefix("you get ")
        .or_else(|| l.strip_prefix("get "))?;
    let (n, tail) = energy_amount(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Effect::AddPlayerCounters {
        who: PlayerRef::You,
        kind: counters::ENERGY.into(),
        n,
    })
}

inventory::submit! { EffectPattern { name: "game rules: you get N {E}", priority: 60, parse: get_energy } }

/// "you may pay eight {E}", "you may pay an amount of {E} equal to that permanent's mana
/// value": an optional payment during resolution (CR 118.12); "if you do" / "when you do"
/// read whether it was paid.
fn may_pay_energy(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("you may pay ")?;
    let n = if let Some(v) = r.strip_prefix("an amount of {e} equal to ") {
        let (v, tail) = crate::oracle::statics::parse_value_phrase(v, b)?;
        if !end(&tail).is_empty() {
            return None;
        }
        v
    } else {
        // Symbols alone are the core payment pattern's.
        if energy_symbols(r).is_some() {
            return None;
        }
        let (n, tail) = energy_amount(r)?;
        if !end(tail).is_empty() || matches!(n, Value::X) {
            return None;
        }
        n
    };
    Some(Effect::PayOptional {
        who: PlayerRef::You,
        cost: Cost {
            mana: None,
            parts: vec![CostPart::PayEnergy(n)],
        },
        then: Box::new(Effect::Noop),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "game rules: you may pay N {E} / an amount of {E} equal to", priority: 60, parse: may_pay_energy } }

/// "you may pay any amount of {E}", "pay any amount of {E}", "you may pay one or more
/// {E}": the player chooses how much energy to pay (none if they don't pay); the amount
/// paid is the instruction's X from then on.
fn pay_any_energy(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (may, r) = match l.strip_prefix("you may pay ") {
        Some(r) => (true, r),
        None => (false, l.strip_prefix("pay ")?),
    };
    match r {
        "any amount of {e}" => {}
        // Paying one or more is a choice not to pay at all, or to pay some (the same
        // choice: "if you do" means at least one was paid).
        "one or more {e}" if may => {}
        _ => return None,
    }
    Some(Effect::Custom(rules::pay_any_amount("{e}", may).into()))
}

inventory::submit! { EffectPattern { name: "game rules: pay any amount of {E}", priority: 60, parse: pay_any_energy } }

/// Whether `e` ends with a payment of an amount of energy the player chose.
fn ends_with_energy_paid(e: &Effect) -> bool {
    match e {
        Effect::Custom(n) => rules::is_pay_any_amount(n) && n.contains("{e}"),
        Effect::Seq(v) => v.last().is_some_and(ends_with_energy_paid),
        _ => false,
    }
}

/// Rewords the references to the energy paid into X ("the amount of {E} paid this way",
/// "where X is the amount of {E} paid this way", "-1/-1 ... for each {E} paid this way",
/// "if you paid one or more {E} this way", "seven or more {E} was paid this way").
fn reword_energy_paid(l: &str) -> Option<String> {
    let mut s = l.to_string();
    if let Some(r) = s.strip_prefix("if you paid one or more {e} this way, ") {
        s = format!("if you do, {r}");
    }
    s = s.replace(", where x is the amount of {e} paid this way", "");
    s = s.replace("the amount of {e} paid this way", "x");
    // "gets -1/-1 until end of turn for each {E} paid this way": X times the change.
    if let Some((head, tail)) = s.split_once(" for each {e} paid this way") {
        if !end(tail).is_empty() {
            return None;
        }
        let (subject, change) = head.rsplit_once(" gets ")?;
        let (pt, rest) = change.split_once(' ').unwrap_or((change, ""));
        let (p, t) = pt.split_once('/')?;
        let unit = |v: &str| matches!(v, "+1" | "-1");
        if !unit(p) || !unit(t) {
            return None;
        }
        let px = format!("{}x", &p[..1]);
        let tx = format!("{}x", &t[..1]);
        s = format!("{subject} gets {px}/{tx} {rest}")
            .trim_end()
            .to_string();
    }
    if s.contains("{e}") && s.contains("paid") {
        return None;
    }
    Some(s.replace("that many", "x").replace("that much", "x"))
}

/// The instructions after "you may pay any amount of {E}" that refer to the amount paid:
/// X is that amount (set by the payment, `kw/spell_cost_grammar.rs`).
fn energy_paid_followup(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !ends_with_energy_paid(prev) || !l.contains("{e}") {
        return false;
    }
    let Some(reworded) = reword_energy_paid(end(l)) else {
        return false;
    };
    let Some(e) = super::value_grammar::with_x_defined(true, || parse_sentence(&reworded, b))
    else {
        return false;
    };
    let p = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![p, e]);
    true
}

inventory::submit! { FollowupPattern { name: "game rules: the amount of {E} paid this way", priority: 90, apply: energy_paid_followup } }

/// "Pay {E}{E}." as an instruction: the player pays if they can (CR 107.14); "If you
/// can't, ..." follows.
fn pay_energy(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("pay ")?;
    let (n, tail) = energy_amount(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    let n = n.as_const()?;
    Some(Effect::Custom(gr::pay_energy(n).into()))
}

inventory::submit! { EffectPattern { name: "game rules: pay {E}{E} (instruction)", priority: 60, parse: pay_energy } }

/// "If you can't, ..." after a mandatory "Pay {E}{E}.".
fn if_you_cant_pay(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    fn ends_with_pay(e: &Effect) -> bool {
        match e {
            Effect::Custom(n) => gr::is_pay_energy(n),
            Effect::Seq(v) => v.last().is_some_and(ends_with_pay),
            _ => false,
        }
    }
    let Some(r) = end(l).strip_prefix("if you can't, ") else {
        return false;
    };
    if !ends_with_pay(prev) {
        return false;
    }
    let Some(e) = crate::oracle::effects::parse_clause(r, b) else {
        return false;
    };
    let p = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![
        p,
        Effect::If {
            cond: Condition::Not(Box::new(Condition::PrevHappened)),
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "game rules: if you can't (pay energy)", priority: 90, apply: if_you_cant_pay } }

/// "Pay {E}{E}" in a keyword's cost ("Equip—Pay {E}{E}.", "Reconfigure—Pay {2} or
/// {E}{E}{E}.").
fn pay_energy_cost(p: &str) -> Option<CostPart> {
    let r = end(p.trim()).strip_prefix("pay ")?;
    let (n, tail) = energy_symbols(r)?;
    end(tail)
        .is_empty()
        .then_some(CostPart::PayEnergy(Value::c(n)))
}

inventory::submit! { CostPattern { name: "game rules: pay {E}{E} (cost)", priority: 60, parse: pay_energy_cost } }

/// "If you pay, ..." after "you may pay [cost]" (Lightning Runner): whether it was paid.
fn you_pay(c: &str) -> Option<Condition> {
    (end(c) == "you pay").then_some(Condition::PrevHappened)
}

inventory::submit! { super::ConditionPattern { name: "game rules: (if) you pay", priority: 900, parse: you_pay } }

/// "Reconfigure—Pay {2} or {E}{E}{E}." (Razorfield Ripper): a keyword whose cost is
/// either of two costs; the same as the keyword once with each cost ("Equip {2} or
/// {B}"): a player activating it pays one or the other (CR 602.2b, 118.1).
fn keyword_either_cost(block: &str, _ctx: &crate::oracle::CompileContext) -> Option<Vec<Ability>> {
    use crate::keywords::{Keyword, KeywordKind};
    use crate::oracle::keywords::parse_keyword_cost;
    let text = block.trim();
    let lower = text.to_lowercase();
    let (name, r) = lower.trim_end_matches('.').split_once("—pay ")?;
    let kind = match name {
        "reconfigure" => KeywordKind::Reconfigure,
        "equip" => KeywordKind::Equip,
        _ => return None,
    };
    let (a, b) = r.split_once(" or ")?;
    let mut out = Vec::new();
    for c in [a, b] {
        let cost = if c.starts_with('{') && energy_symbols(c).is_none() {
            parse_keyword_cost(c)?
        } else {
            parse_keyword_cost(&format!("pay {c}"))?
        };
        let kw = Keyword {
            cost: Some(cost),
            text: Some(smol_str::SmolStr::new(text)),
            ..Keyword::new(kind)
        };
        out.push(AbilityDef::new(AbilityKind::Keyword(kw), text));
    }
    Some(out)
}

inventory::submit! { super::AbilityPattern { name: "game rules: [keyword]—pay [cost] or [cost]", priority: 100, parse: keyword_either_cost } }
