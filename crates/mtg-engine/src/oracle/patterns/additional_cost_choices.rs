//! Choices made as additional costs to cast a spell (CR 601.2b) and what its effect says
//! about them (cards5-costs-sets):
//!
//! - "As an additional cost to cast ~, you may pay {1}{R} and/or {1}{G} any number of
//!   times." (Primitive Justice): two repeatable additional costs, each counted on its own
//!   as the spell is cast (CR 601.2b, 601.2f), and "For each additional {1}{R} you paid,
//!   destroy another target artifact.": a target group whose size is that count (chosen
//!   after the costs are announced, CR 601.2c; the same artifact can't be targeted twice,
//!   its ruling), with "and you gain 1 life" once for each.

use super::{AbilityPattern, EffectPattern};
use crate::ability::*;
use crate::mana::ManaCost;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::{end, parse_number};
use crate::oracle::CompileContext;

/// The name a repeatable mana additional cost is announced under ("pay {1}{r} any number
/// of times", as `spell_cost_grammar` names it; lowercase mana).
fn repeat_name(mana: &str) -> String {
    format!("pay {mana} any number of times")
}

/// A mana cost written exactly as Oracle text writes it (lowercase), with no X.
fn plain_mana(m: &str) -> Option<ManaCost> {
    let mana = ManaCost::parse(&m.to_uppercase())?;
    (!mana.has_x() && format!("{mana}").to_lowercase() == m && !m.is_empty()).then_some(mana)
}

/// "As an additional cost to cast ~, you may pay {A} and/or {B} any number of times."
fn two_repeatable_mana_costs(text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if text.contains('\n') || !ctx.is_spell() {
        return None;
    }
    let lower = text.to_lowercase();
    let r = end(&lower)
        .strip_prefix("as an additional cost to cast ~, you may pay ")?
        .strip_suffix(" any number of times")?;
    let (a, b) = r.split_once(" and/or ")?;
    if a == b {
        return None;
    }
    let mut out = Vec::new();
    for m in [a, b] {
        let mana = plain_mana(m)?;
        let name = repeat_name(m);
        let times = Value::Custom(crate::kw::spell_cost_grammar::paid_times(&name).into());
        let cost = Cost::free().with(CostPart::Repeated {
            cost: Box::new(Cost::mana(mana)),
            times,
        });
        out.push(super::costs_casting_self::this_spell_cost_ability(
            CostChange::AdditionalCost(cost),
            None,
            text,
        ));
    }
    Some(out)
}

inventory::submit! { AbilityPattern { name: "additional cost choices: you may pay {A} and/or {B} any number of times", priority: 78, parse: two_repeatable_mana_costs } }

/// Whether the card's text has a repeatable additional mana cost `mana` ("you may pay
/// {1}{r} any number of times", alone or in an "and/or" pair).
fn has_repeatable_cost(mana: &str) -> bool {
    let raw = crate::oracle::raw_text().to_lowercase();
    let raw = raw.replace("as an additional cost to cast this spell,", "as an additional cost to cast ~,");
    raw.lines().any(|l| {
        let Some(r) = l
            .trim_end_matches('.')
            .strip_prefix("as an additional cost to cast ~, you may pay ")
            .and_then(|r| r.strip_suffix(" any number of times"))
        else {
            return false;
        };
        r == mana || r.split(" and/or ").any(|m| m == mana)
    })
}

/// "For each additional {1}{R} you paid, destroy another target artifact[, and you gain
/// N life]": the body's last target instance is chosen as many times as that cost was
/// paid (and none if it wasn't), and the life is gained once for each.
fn for_each_additional_paid(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("for each additional ")?;
    let (mana, body) = r.split_once(" you paid, ")?;
    plain_mana(mana)?;
    if !has_repeatable_cost(mana) {
        return None;
    }
    let times = Value::Custom(crate::kw::spell_cost_grammar::paid_times(&repeat_name(mana)).into());
    let (body, life) = match body.rsplit_once(", and you gain ") {
        Some((head, tail)) => {
            let (n, rest) = parse_number(tail.strip_suffix(" life")?)?;
            if !rest.trim().is_empty() || n.as_const().is_none() {
                return None;
            }
            (head, Some(n))
        }
        None => (body, None),
    };
    let saved = b.targets.len();
    let fail = |b: &mut Builder| {
        b.targets.truncate(saved);
        None
    };
    let Some(e) = parse_clause(body, b) else {
        return fail(b);
    };
    // Exactly one new instance of "target", of one object, which the count sets.
    if b.targets.len() != saved + 1 {
        return fail(b);
    }
    let t = &mut b.targets[saved];
    if t.min.as_const() != Some(1) || t.max.as_const() != Some(1) || t.per_player.is_some() {
        return fail(b);
    }
    // Only an effect on that target ("destroy another target artifact").
    let slot = saved as u8;
    let on_target = match &e {
        Effect::Destroy { what, .. } | Effect::Exile { what, .. } => {
            matches!(what, Sel::Target(s) if *s == slot)
        }
        _ => false,
    };
    if !on_target {
        return fail(b);
    }
    t.min = times.clone();
    t.max = times.clone();
    let mut v = vec![e];
    if let Some(Value::Const(n)) = life {
        v.push(Effect::GainLife {
            who: PlayerRef::You,
            n: match n {
                1 => times,
                n => Value::Mul(Box::new(Value::c(n)), Box::new(times)),
            },
        });
    }
    Some(Effect::seq(v))
}

inventory::submit! { EffectPattern { name: "additional cost choices: for each additional {M} you paid, [instruction on another target]", priority: 60, parse: for_each_additional_paid } }
