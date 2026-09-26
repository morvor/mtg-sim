//! Additional costs a spell's own text lets its controller choose as it's cast
//! (CR 601.2b): "As an additional cost to cast this spell, you may [cost]" and
//! "As an additional cost to cast this spell, [cost] or [cost]".
//!
//! The choice is announced before targets are chosen; the cost is added to the total cost
//! and paid with the rest of it (CR 601.2f–h). The name of what was chosen is recorded in
//! the spell's `CastInfo::paid`, which conditions such as "If a Dragon was beheld" check
//! (CR 701.4b).

use crate::ability::*;
use crate::casting::add_cost;
use crate::decision::{Answer, Decision};
use crate::game::Game;
use crate::object::Characteristics;
use crate::types::*;
use smol_str::SmolStr;

/// The spell's own optional additional costs and additional-cost choices.
fn own_choices(chars: &Characteristics) -> Vec<CostChange> {
    chars
        .abilities
        .iter()
        .filter_map(|a| match &a.kind {
            AbilityKind::Static(s) => match &s.effect {
                StaticEffect::CostModifier(CostModifier {
                    applies_to: CostTarget::ThisSpell,
                    change:
                        c @ (CostChange::OptionalAdditionalCost { .. }
                        | CostChange::AdditionalCostChoice(_)),
                    ..
                }) => Some(c.clone()),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

/// Announces the spell's own optional additional costs and choices between additional
/// costs (CR 601.2b), adding what was chosen to `extra` and recording its name in `paid`.
/// A cost that can't be paid can't be chosen.
pub fn announce(
    g: &mut Game,
    p: PlayerId,
    spell: ObjectId,
    chars: &Characteristics,
    extra: &mut Cost,
    paid: &mut Vec<SmolStr>,
) {
    for change in own_choices(chars) {
        match change {
            CostChange::OptionalAdditionalCost { name, cost } => {
                if g.can_pay_cost_optimistic(p, &cost, Some(spell), chars)
                    && matches!(
                        g.ask(
                            p,
                            Decision::OptionalCost {
                                source: spell,
                                name: name.to_string(),
                                repeatable: false,
                            }
                        ),
                        Answer::Bool(true)
                    )
                {
                    add_cost(extra, &cost);
                    paid.push(name);
                }
            }
            CostChange::AdditionalCostChoice(options) => {
                let payable: Vec<(SmolStr, Cost)> = options
                    .iter()
                    .filter(|(_, c)| g.can_pay_cost_optimistic(p, c, Some(spell), chars))
                    .cloned()
                    .collect();
                // If none can be paid, the first is chosen (and paying it will fail).
                let pool = if payable.is_empty() {
                    options.clone()
                } else {
                    payable
                };
                let i = g.ask_option(
                    p,
                    Some(spell),
                    "Choose an additional cost to pay",
                    pool.iter().map(|(n, _)| n.to_string()).collect(),
                );
                if let Some((name, cost)) = pool.into_iter().nth(i) {
                    add_cost(extra, &cost);
                    paid.push(name);
                }
            }
            _ => {}
        }
    }
}
