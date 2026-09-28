//! Triggers on activating abilities (CR 602.2) that copy them (CR 707.10): "Whenever you
//! activate an ability that targets a creature or player, copy that ability. You may
//! choose new targets for the copy." (Ertha Jo, Frontier Mentor), "Whenever you activate
//! an ability that isn't a mana ability, copy it." A copy of an ability isn't activated,
//! so it doesn't trigger these again.

use super::{EffectPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::types::CardType;

/// "you activate an ability that isn't a mana ability", "you activate an ability that
/// targets a creature or player" (CR 115.9b: some current target is one).
fn you_activate_an_ability(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let rest = end(r).strip_prefix("you activate an ability")?;
    let activated = TriggerCond::AbilityActivated {
        who: PlayerRel::You,
        source: Filter::Any,
        include_mana: false,
    };
    let trigger = match rest {
        " that isn't a mana ability" => activated,
        " that targets a creature or player" => TriggerCond::Where {
            trigger: Box::new(activated),
            cond: Condition::SelMatches(
                Sel::TriggerSpell,
                Filter::StackTargets(Box::new(TargetsFilter::Targets {
                    objects: Some(Filter::and(vec![
                        Filter::Permanent,
                        Filter::Type(CardType::Creature),
                    ])),
                    players: Some(PlayerFilter::Any),
                })),
            ),
        },
        _ => return None,
    };
    Some((trigger, Sel::TriggerSpell, PlayerRef::You))
}

inventory::submit! { TriggerPattern { name: "you activate an ability [that ...]", priority: 100, parse: you_activate_an_ability } }

/// "copy that ability" in an ability that triggers on activating one: the ability
/// activated ("copy it" is `copy_spells`).
fn copy_that_ability(l: &str, b: &mut Builder) -> Option<Effect> {
    if end(l) != "copy that ability" || !matches!(b.it, Sel::TriggerSpell) {
        return None;
    }
    Some(Effect::CopySpell {
        what: Sel::TriggerSpell,
        count: Value::c(1),
        new_targets: false,
    })
}

inventory::submit! { EffectPattern { name: "copy that ability (the one activated)", priority: 100, parse: copy_that_ability } }
