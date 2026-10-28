//! Cost grammar V (cards4-costs-mana-replacements).
//!
//! - Rules about a spell's cost that apply only when it's cast a particular way
//!   ([`CostRule::CastThisWay`], see `payment_rules.rs`): "You may pay {R}{G} rather than
//!   pay ~'s mana cost. Spend only mana produced by Treasures to cast it this way."
//!   (Security Rhox: an alternative cost of the spell's own text, CR 118.9, whose mana may
//!   come only from such sources, CR 106.3, 601.2h), "Flashback—{3}{R}, Remove X loyalty
//!   counters from among planeswalkers you control. If you cast ~ this way, X can't be 0."
//!   (Light Up the Night: CR 107.3a when cast with flashback).

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

/// The rule of a "this way" sentence (lowercase, no period).
fn this_way_rule(s: &str) -> Option<CostRule> {
    if matches!(
        s,
        "if you cast ~ this way, x can't be 0" | "if you cast this spell this way, x can't be 0"
    ) {
        return Some(CostRule::XAtLeast(1));
    }
    let r = s
        .strip_prefix("spend only mana produced by ")?
        .strip_suffix(" to cast it this way")
        .or_else(|| {
            s.strip_prefix("spend only mana produced by ")?
                .strip_suffix(" to cast ~ this way")
        })?;
    let (f, plural, tail) = parse_object_phrase(r)?;
    if !plural || !end(tail).trim().is_empty() {
        return None;
    }
    let f = Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]);
    Some(CostRule::OnlyManaFrom(crate::mana::SpendFilter::new(f)))
}

/// "[alternative cost or keyword alternative cost]. [rule that applies when cast this
/// way]."
fn cast_this_way_rule(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if block.contains('\n') {
        return None;
    }
    let block = block.trim_end();
    let (head, tail) = ["Spend only mana produced by ", "If you cast ~ this way, "]
        .iter()
        .find_map(|p| {
            let i = block.find(p)?;
            (i > 0 && block[..i].trim_end().ends_with('.')).then(|| (&block[..i], &block[i..]))
        })?;
    let rule = this_way_rule(end(&tail.to_lowercase()))?;
    let out = crate::oracle::parse_ability(head.trim_end(), ctx)?;
    let [a] = out.as_slice() else {
        return None;
    };
    let way = match &a.kind {
        // A keyword's alternative cost (flashback, ...).
        AbilityKind::Keyword(k) if k.cost.is_some() => CastWay::Keyword(k.kind),
        AbilityKind::Static(s)
            if matches!(
                &s.effect,
                StaticEffect::CostModifier(CostModifier {
                    applies_to: CostTarget::ThisSpell,
                    change: CostChange::AlternativeCost(_)
                        | CostChange::AlternativeCostWithFlash(_),
                    ..
                })
            ) =>
        {
            CastWay::OwnAlternativeCost
        }
        _ => return None,
    };
    let rule = CostRule::CastThisWay {
        way,
        rule: Box::new(rule),
    };
    let mut out = out.clone();
    out.push(super::costs_casting_self::this_spell_cost_ability(
        CostChange::Rule(rule),
        None,
        tail.trim(),
    ));
    Some(out)
}

inventory::submit! { AbilityPattern { name: "costs 5: [alternative cost]. [rule] to cast it this way", priority: 80, parse: cast_this_way_rule } }
