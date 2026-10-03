//! Damage dealt by each object of a group named before: "Each of those creatures deals
//! damage equal to its power to ~." (Living Inferno), "They each deal damage equal to
//! their power to target creature." Every one of those objects deals its own damage, all
//! at the same time (CR 120.2); the amount is evaluated for each of them.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

fn group_deals_damage(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (group, rest) = super::pronoun_groups::plural_object_ref(l, b)??;
    let rest = rest.trim_start();
    let rest = ["deals damage equal to ", "deal damage equal to ", "each deal damage equal to "]
        .iter()
        .find_map(|p| rest.strip_prefix(p))?;
    // "its power" / "their power": each of the sources in turn (the executor binds
    // `vars::AFFECTED` to each).
    let saved = std::mem::replace(&mut b.it, Sel::Var(vars::AFFECTED));
    let rest = match rest.strip_prefix("their ") {
        Some(r) => format!("its {r}"),
        None => rest.to_string(),
    };
    let parsed = crate::oracle::statics::parse_value_phrase(&rest, b);
    b.it = saved;
    let (amount, tail) = parsed?;
    let to = tail.trim_start().strip_prefix("to ")?;
    let (to, tail) = crate::oracle::effects::damage_recipients(to, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    Some(Effect::DealDamage {
        source: Sel::Union(vec![group]),
        amount,
        to,
    })
}

inventory::submit! { EffectPattern { name: "choice grammar: each of those objects deals damage equal to its power", priority: 60, parse: group_deals_damage } }

/// "If ~'s madness cost was paid, it deals X damage divided as you choose among those
/// permanents and/or players instead." (Avacyn's Judgment), "If X is 6 or more, ~ deals
/// twice X damage divided as you choose among them instead." (Shatterskull Smashing):
/// the same targets, another total. The division is announced as the spell is cast (CR
/// 601.2d), after the costs to pay and X are announced (CR 601.2b), so the condition is
/// read then: the total to divide becomes the second amount if it holds.
fn divided_instead(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (cond, rest) = l.strip_prefix("if ")?.split_once(", ")?;
    let rest = rest
        .strip_prefix("~ deals ")
        .or_else(|| rest.strip_prefix("it deals "))?;
    let (amount, rest) = if let Some(r) = rest.strip_prefix("twice x ") {
        (Value::Mul(Box::new(Value::c(2)), Box::new(Value::X)), r)
    } else {
        let (n, r) = parse_number(rest)?;
        if rest.starts_with("a ") || rest.starts_with("an ") {
            return None;
        }
        (n, r)
    };
    let among = rest.trim_start().strip_prefix("damage divided as you choose among ")?;
    let among = among.strip_suffix(" instead")?;
    if !matches!(
        among,
        "them" | "those targets" | "those permanents and/or players" | "those creatures and/or planeswalkers"
    ) {
        return None;
    }
    let cond = super::conditions_referents::parse_condition_with(cond, b)
        .or_else(|| crate::oracle::statics::parse_condition(cond, b.ctx))?;
    // The one slot whose damage is divided.
    let mut divided = b
        .targets
        .iter_mut()
        .filter(|s| s.divide.is_some());
    let spec = divided.next()?;
    if divided.next().is_some() {
        return None;
    }
    let old = spec.divide.take()?;
    // "Any number of targets" is at most one per point of damage.
    if format!("{:?}", spec.max) == format!("{old:?}") {
        spec.max = Value::If(
            Box::new(cond.clone()),
            Box::new(amount.clone()),
            Box::new(old.clone()),
        );
    }
    spec.divide = Some(Value::If(Box::new(cond), Box::new(amount), Box::new(old)));
    Some(Effect::Noop)
}

inventory::submit! { EffectPattern { name: "choice grammar: if [condition], divided damage among them instead", priority: 60, parse: divided_instead } }
