//! Characteristic-defining power/toughness abilities (CR 604.3, 613.4a): "~'s power
//! and toughness are each equal to [amount]", "~'s power is equal to [amount]", and
//! "~'s power is equal to [amount] and its toughness is equal to that number plus N".
//! Amounts are parsed by [`super::statics::parse_amount`]. A CDA functions in every
//! zone (CR 604.3), so it's compiled with `FunctionZone::Anywhere`.

use super::statics::parse_amount;
use crate::ability::*;
use crate::oracle::patterns::StaticPattern;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn cda(p: Option<Value>, t: Option<Value>, text: &str) -> Ability {
    let mut s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::Source,
        mods: vec![Modification::CdaPT(p, t)],
    });
    s.is_cda = true;
    s.zone = FunctionZone::Anywhere;
    AbilityDef::new(AbilityKind::Static(s), text)
}

/// Whether the printed value is a `*` (CR 208.2a: the CDA defines it). A CDA for a
/// characteristic that's printed as a number would be something else.
fn is_star(v: Option<&str>) -> bool {
    v.is_some_and(|v| v.contains('*'))
}

fn parse_cda(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let it = Some(Sel::This);
    if let Some(r) = l.strip_prefix("~'s power and toughness are each equal to ") {
        if !is_star(ctx.power) || !is_star(ctx.toughness) {
            return None;
        }
        let v = parse_amount(r, it.as_ref())?;
        return Some(vec![cda(Some(v.clone()), Some(v), text)]);
    }
    if let Some(r) = l.strip_prefix("~'s power is equal to ") {
        if !is_star(ctx.power) {
            return None;
        }
        // "... and its toughness is equal to that number plus N"
        if let Some((a, b)) = r.split_once(" and its toughness is equal to ") {
            if !is_star(ctx.toughness) {
                return None;
            }
            let p = parse_amount(a, it.as_ref())?;
            let t = if let Some(n) = b.strip_prefix("that number plus ") {
                Value::Sum(vec![p.clone(), parse_amount(n, it.as_ref())?])
            } else if b == "that number" {
                p.clone()
            } else {
                parse_amount(b, it.as_ref())?
            };
            return Some(vec![cda(Some(p), Some(t), text)]);
        }
        if is_star(ctx.toughness) {
            return None;
        }
        let p = parse_amount(r, it.as_ref())?;
        return Some(vec![cda(Some(p), None, text)]);
    }
    if let Some(r) = l.strip_prefix("~'s toughness is equal to ") {
        if !is_star(ctx.toughness) || is_star(ctx.power) {
            return None;
        }
        let t = parse_amount(r, it.as_ref())?;
        return Some(vec![cda(None, Some(t), text)]);
    }
    None
}

inventory::submit! {
    StaticPattern {
        name: "statics: power/toughness CDAs",
        priority: 50,
        parse: parse_cda,
    }
}
