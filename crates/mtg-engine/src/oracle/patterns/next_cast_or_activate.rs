//! "When you next cast a spell with {X} in its mana cost or activate an ability with {X}
//! in its activation cost this turn, copy that spell or ability." (Magus Lucea Kane): a
//! delayed triggered ability that triggers once, on the next such spell cast or ability
//! activated this turn (CR 603.7b, 603.7c).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn next_cast_or_activate(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("when you next cast ")?;
    let (spell, rest) = r.split_once(" or activate ")?;
    let (ability, eff) = rest.split_once(" this turn, ")?;
    let (cast, _, _) =
        crate::oracle::triggers::parse_trigger_condition(&format!("whenever you cast {spell}"))?;
    if !matches!(
        cast,
        TriggerCond::CastSpell {
            who: PlayerRel::You,
            ..
        }
    ) {
        return None;
    }
    let activated = TriggerCond::AbilityActivated {
        who: PlayerRel::You,
        source: Filter::Any,
        include_mana: false,
    };
    let activate = match ability {
        "an ability" => activated,
        "an ability with {x} in its activation cost" => TriggerCond::Where {
            trigger: Box::new(activated),
            cond: Condition::SelMatches(
                Sel::TriggerSpell,
                Filter::Custom(crate::stack_ability_filters::X_IN_ACTIVATION_COST.into()),
            ),
        },
        _ => return None,
    };
    let body =
        crate::oracle::effects::parse_trigger_body(eff, b.ctx, Sel::TriggerSpell, PlayerRef::You)?;
    if body.modal.is_some() {
        return None;
    }
    Some(Effect::DelayedTrigger {
        trigger: TriggerCond::ThisTurn(Box::new(TriggerCond::AnyOf(vec![cast, activate]))),
        body: Box::new(body),
        once: true,
    })
}

inventory::submit! { EffectPattern { name: "when you next cast [a spell] or activate [an ability] this turn", priority: 110, parse: next_cast_or_activate } }
