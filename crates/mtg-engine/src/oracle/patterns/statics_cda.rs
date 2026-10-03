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

/// An amount the value grammar reads ("20 minus the highest life total among players",
/// "your life total minus the life total of an opponent with the most life"), with no
/// targets and "it" meaning the source.
fn grammar_amount(s: &str, ctx: &CompileContext) -> Option<Value> {
    let mut b = crate::oracle::effects::Builder::new(ctx);
    b.it = Sel::This;
    let (v, rest) = crate::oracle::statics::parse_value_phrase(s, &mut b)?;
    (rest.trim().is_empty() && b.targets.is_empty()).then_some(v)
}

fn parse_cda(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let it = Some(Sel::This);
    if let Some(r) = l.strip_prefix("~'s power and toughness are each equal to ") {
        if !is_star(ctx.power) || !is_star(ctx.toughness) {
            return None;
        }
        let v = parse_amount(r, it.as_ref()).or_else(|| grammar_amount(r, ctx))?;
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

/// "As long as ~ isn't attacking, its power and toughness are each equal to the number of
/// Forests you control. As long as ~ is attacking, its power and toughness are each equal
/// to the number of Forests defending player controls." (Gaea's Liege): a
/// characteristic-defining ability whose value depends on the object's state.
fn conditional_cda(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !is_star(ctx.power) || !is_star(ctx.toughness) {
        return None;
    }
    let l = end(l);
    let mut out = Vec::new();
    for part in l.split(". ") {
        let r = part.strip_prefix("as long as ")?;
        let (c, amount) = r.split_once(", its power and toughness are each equal to ")?;
        let cond = crate::oracle::statics::parse_condition(c, ctx)?;
        let v = parse_amount(amount, Some(&Sel::This)).or_else(|| grammar_amount(amount, ctx))?;
        let mut a = cda(Some(v.clone()), Some(v), text);
        if let AbilityKind::Static(st) = &mut std::sync::Arc::make_mut(&mut a).kind {
            st.condition = Some(cond);
        }
        out.push(a);
    }
    (out.len() >= 2).then_some(out)
}

inventory::submit! {
    StaticPattern {
        name: "statics: conditional power/toughness CDAs",
        priority: 50,
        parse: conditional_cda,
    }
}

inventory::submit! {
    StaticPattern {
        name: "statics: power/toughness CDAs",
        priority: 50,
        parse: parse_cda,
    }
}
