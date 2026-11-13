//! A choice between costs (CR 118.3, 601.2h):
//!
//! * a cost "[cost A] or [cost B]" ([`CostPart::OneOf`]): "Ward—Discard a card or pay
//!   {2}." (the player whose spell or ability would be countered chooses which to pay as
//!   the ward ability resolves, CR 702.21a), "you may discard a card or pay {1}" in an
//!   effect;
//! * a keyword whose cost is a choice ("Equip—Pay {3} or discard a card.", "Equip—Pay {3}
//!   or remove a counter from ~."): one keyword ability for each cost, as for an activated
//!   ability with two costs (`either_cost.rs`), so that effects on the keyword's cost
//!   (reductions, "equip abilities you activate cost {1} less") apply to whichever is
//!   paid.

use super::AbilityPattern;
use crate::oracle::effects::Builder;
use crate::ability::*;
use crate::keywords::KeywordKind;
use crate::mana::ManaCost;
use crate::oracle::CompileContext;

const VERBS: [&str; 8] = [
    "discard ", "pay ", "sacrifice ", "exile ", "remove ", "tap ", "return ", "put ",
];

/// One side of the choice: "pay {2}" (mana), or a cost part.
fn side(s: &str) -> Option<Cost> {
    let s = s.trim();
    if let Some(m) = s.strip_prefix("pay {") {
        let sym = format!("{{{m}");
        if sym.ends_with('}') && !sym.to_lowercase().contains("{e}") {
            return Some(Cost::mana(ManaCost::parse(&sym.to_uppercase())?));
        }
    }
    let (c, loyalty) = crate::oracle::costs::parse_cost(s)?;
    (!loyalty && !c.parts.iter().any(|p| matches!(p, CostPart::OneOf(_)))).then_some(c)
}

/// "[cost A] or [cost B]" (lowercase), each a whole cost starting with its verb.
pub(crate) fn one_of(p: &str) -> Option<Vec<Cost>> {
    let p = crate::oracle::phrases::end(p.trim());
    // Two costs, not a list of three or more ("sacrifice a creature, discard a card, or
    // pay 4 life" is offered by `costs_casting_additional`).
    if !VERBS.iter().any(|v| p.starts_with(v)) || p.contains(", ") {
        return None;
    }
    let i = p
        .match_indices(" or ")
        .map(|(i, _)| i)
        .find(|i| VERBS.iter().any(|v| p[i + 4..].starts_with(v)))?;
    let (a, b) = (&p[..i], &p[i + 4..]);
    Some(vec![side(a)?, side(b)?])
}

/// "Ward—Discard a card or pay {2}.": the ward cost is a choice of two (made by the player
/// paying it as the ward ability resolves).
fn ward_choice(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    let rest = text.strip_prefix("Ward—")?;
    if rest.contains('\n') || rest.contains('(') {
        return None;
    }
    let costs = one_of(&rest.to_lowercase())?;
    let kw = crate::keywords::Keyword {
        cost: Some(Cost::free().with(CostPart::OneOf(costs))),
        text: Some(smol_str::SmolStr::new(text)),
        ..crate::keywords::Keyword::new(KeywordKind::Ward)
    };
    Some(vec![AbilityDef::new(AbilityKind::Keyword(kw), text)])
}

inventory::submit! { AbilityPattern { name: "cost choices: Ward—[cost] or [cost]", priority: 55, parse: ward_choice } }

/// "you may discard a card or pay {2}", "you may sacrifice a Food or pay {1}{W}": an
/// optional cost paid as the ability resolves (CR 118.12), either one; "you may sacrifice
/// ~ and pay {2}{G}{G}": both. A following "When you do" or "If you do" reads whether it
/// was paid.
pub(crate) fn may_pay_either(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = crate::oracle::phrases::end(l).strip_prefix("you may ")?;
    if r.contains("target") {
        return None;
    }
    let _ = b;
    let cost = match one_of(r) {
        Some(costs) => Cost::free().with(CostPart::OneOf(costs)),
        None => both(r)?,
    };
    Some(Effect::PayOptional {
        who: PlayerRef::You,
        cost,
        then: Box::new(Effect::Noop),
        otherwise: Box::new(Effect::Noop),
    })
}

/// "sacrifice ~ and pay {2}{G}{G}", "pay {2}{U}{U}{B}{B} and exile eight cards from your
/// graveyard": a cost of both parts, paid in full or not at all (CR 118.3, 118.12).
fn both(r: &str) -> Option<Cost> {
    if !VERBS.iter().any(|v| r.starts_with(v)) {
        return None;
    }
    let i = r
        .match_indices(" and ")
        .map(|(i, _)| i)
        .find(|i| VERBS.iter().any(|v| r[i + 5..].starts_with(v)))?;
    let (a, b) = (side(&r[..i])?, side(&r[i + 5..])?);
    let mut cost = a;
    crate::casting::add_cost(&mut cost, &b);
    // Mana is paid with the rest of the cost; nothing else may be paid twice.
    Some(cost)
}

/// "Equip—Pay {3} or discard a card.": one equip ability for each cost.
fn keyword_cost_choice(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    let (kw, rest) = text.split_once('—')?;
    if kw != "Equip" || rest.contains('\n') || rest.contains('(') {
        return None;
    }
    let costs = one_of(&rest.to_lowercase())?;
    let mut out = Vec::new();
    for c in costs {
        let mut abilities = crate::oracle::parse_ability(&format!("Equip {{0}}"), ctx)?;
        let [a] = abilities.as_mut_slice() else {
            return None;
        };
        let a = std::sync::Arc::make_mut(a);
        let AbilityKind::Keyword(k) = &mut a.kind else {
            return None;
        };
        if k.kind != KeywordKind::Equip {
            return None;
        }
        k.cost = Some(c);
        a.text = text.to_string();
        out.extend(abilities);
    }
    Some(out)
}

inventory::submit! { AbilityPattern { name: "cost choices: Equip—[cost] or [cost]", priority: 60, parse: keyword_cost_choice } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choices_of_costs() {
        let c = one_of("discard a card or pay {2}").unwrap();
        assert!(matches!(c[0].parts.as_slice(), [CostPart::Discard { .. }]));
        assert!(c[1].mana.is_some() && c[1].parts.is_empty());
        let c = one_of("pay {3} or remove a counter from ~").unwrap();
        assert!(c[0].mana.is_some());
        assert_eq!(c[1].parts.len(), 1);
        assert!(one_of("sacrifice a food or pay {1}{w}").is_some());
        // Not a choice of costs: one object of either kind.
        assert!(one_of("sacrifice an artifact or creature").is_none());
    }
}
