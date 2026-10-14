//! An alternative cost whose object is described with X (CR 107.3a, 118.9): "You may
//! exile a blue card with mana value X from your hand rather than pay this spell's mana
//! cost." (Disrupting Shoal and the other Shoals). The caster announces X as the spell is
//! cast (CR 601.2b) and exiles a card with that mana value (CR 601.2h); the X in the
//! spell's mana cost and text is that value while it's on the stack (CR 107.3a), so its
//! mana value includes it although no mana was spent on X (CR 202.3e). See
//! [`crate::x_cost_filters`].

use super::costs_casting_self::this_spell_cost_ability;
use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::costs::parse_cost;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn alternative_cost_with_x(text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if text.contains('\n') {
        return None;
    }
    let lower = text.to_lowercase();
    let body = end(&lower).strip_prefix("you may ")?;
    let cost_s = body.strip_suffix(" rather than pay ~'s mana cost")?;
    let (cost, false) = parse_cost(cost_s)? else {
        return None;
    };
    // Only a single object described with X: "exile a [quality] card with mana value X
    // from your hand".
    let [part] = cost.parts.as_slice() else {
        return None;
    };
    if cost.mana.is_some()
        || !crate::x_cost_filters::part_has_x_filter(part)
        || !matches!(part, CostPart::Exile { count: Value::Const(1), .. })
    {
        return None;
    }
    Some(vec![this_spell_cost_ability(
        CostChange::AlternativeCost(cost),
        None,
        text,
    )])
}

inventory::submit! { AbilityPattern { name: "r107.3a: you may exile a card with mana value X rather than pay ~'s mana cost", priority: 80, parse: alternative_cost_with_x } }
