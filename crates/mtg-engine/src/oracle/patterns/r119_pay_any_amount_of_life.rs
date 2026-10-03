//! "As ~ enters, pay any amount of life." and "The amount you pay can't be more than
//! [value]." (Nameless Race): a replacement effect that modifies how the permanent enters
//! (CR 614.1c, 614.12a), the payment kept for "the life paid as it entered" (CR 607.2g;
//! see `kw/pay_any_amount_of_life.rs`).

use super::AbilityPattern;
use crate::ability::*;
use crate::kw::pay_any_amount_of_life::{CAP, PAY_ANY_LIFE};
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn pay_any_amount_of_life(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() {
        return None;
    }
    let lower = block.trim().to_lowercase();
    let l = end(&lower);
    let rest = l.strip_prefix("as ~ enters, pay any amount of life")?;
    let mut effects = Vec::new();
    if !rest.is_empty() {
        let cap = rest.strip_prefix(". the amount you pay can't be more than ")?;
        // "... your opponents control plus the total number of white cards in their
        // graveyards": the opponents' graveyards.
        let cap = if cap.contains("your opponents control") {
            cap.replace(" in their graveyards", " in an opponent's graveyard")
        } else {
            cap.to_string()
        };
        let cap = cap.as_str();
        let mut b = Builder::new(ctx);
        let (v, tail) = crate::oracle::statics::parse_value_phrase(cap, &mut b)?;
        if !tail.trim().is_empty() || !b.targets.is_empty() {
            return None;
        }
        effects.push(Effect::StoreValue { var: CAP, value: v });
    }
    effects.push(Effect::Custom(PAY_ANY_LIFE.into()));
    let s = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
        event: ReplacementEvent::EntersBattlefield(Filter::Source),
        action: ReplacementAction::AsEnters(Box::new(Effect::seq(effects))),
        self_replacement: false,
        optional: false,
    }));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), block.trim())])
}

inventory::submit! { AbilityPattern { name: "r119 as ~ enters, pay any amount of life", priority: 1100, parse: pay_any_amount_of_life } }
