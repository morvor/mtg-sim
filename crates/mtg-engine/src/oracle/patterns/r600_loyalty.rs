//! Oracle patterns for effects that modify the costs of loyalty abilities (CR 606.4):
//! "Planeswalkers' loyalty abilities you activate cost an additional [+1] to activate."
//! and "Loyalty abilities of planeswalkers your opponents control cost {1} more to
//! activate."

use super::StaticPattern;
use crate::ability::*;
use crate::mana::ManaCost;
use crate::oracle::CompileContext;
use crate::types::CardType;

fn loyalty_symbol(s: &str) -> Option<i32> {
    let r = s.strip_prefix('[')?.strip_suffix(']')?;
    let r = r.replace('\u{2212}', "-");
    if let Some(n) = r.strip_prefix('+') {
        n.parse().ok()
    } else if let Some(n) = r.strip_prefix('-') {
        n.parse::<i32>().ok().map(|n| -n)
    } else {
        None
    }
}

fn loyalty_cost_modifier(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let planeswalkers = Filter::Type(CardType::Planeswalker);
    let (who, change) = if let Some(r) =
        l.strip_prefix("planeswalkers' loyalty abilities you activate cost an additional ")
    {
        let n = loyalty_symbol(r.strip_suffix(" to activate")?)?;
        (
            PlayerRel::You,
            CostChange::AdditionalCost(Cost::default().with(CostPart::Loyalty(n))),
        )
    } else if let Some(r) =
        l.strip_prefix("loyalty abilities of planeswalkers your opponents control cost ")
    {
        let m = r.strip_suffix(" more to activate")?;
        let cost = ManaCost::parse(&m.to_uppercase())?;
        (PlayerRel::Opponent, CostChange::IncreaseMana(cost))
    } else {
        return None;
    };
    let s = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::LoyaltyAbilities(planeswalkers),
        who,
        change,
    }));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "loyalty ability cost modifiers", priority: 0, parse: loyalty_cost_modifier } }
