//! Oracle text of the keywords of CR 702.98–702.110 that the generic keyword parser
//! doesn't handle, and phrases that go with them:
//!
//! * "if it's attacking the player with the most life or tied for most life" (CR 702.105a);
//! * "Whenever you activate ~'s outlast ability" (CR 702.107a).

use super::{ConditionPattern, TriggerPattern};
use crate::ability::*;

/// "if it's attacking the player with the most life or tied for most life" (Scourge of the
/// Throne; the dethrone condition, CR 702.105a).
fn attacking_player_with_most_life(l: &str) -> Option<Condition> {
    matches!(
        l,
        "it's attacking the player with the most life or tied for most life"
            | "~ is attacking the player with the most life or tied for most life"
    )
    .then(|| Condition::Custom(crate::kw::dethrone::ATTACKING_PLAYER_WITH_MOST_LIFE.into()))
}

inventory::submit! { ConditionPattern { name: "it's attacking the player with the most life", priority: 100, parse: attacking_player_with_most_life } }

/// "Whenever you activate ~'s outlast ability" (Herald of Anafenza).
fn activate_outlast(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    if r != "you activate ~'s outlast ability" {
        return None;
    }
    Some((
        TriggerCond::Where {
            trigger: Box::new(TriggerCond::AbilityActivated {
                who: PlayerRel::You,
                source: Filter::Source,
                include_mana: false,
            }),
            cond: Condition::Custom(crate::kw::outlast::OUTLAST_ACTIVATED.into()),
        },
        Sel::This,
        PlayerRef::You,
    ))
}

inventory::submit! { TriggerPattern { name: "you activate ~'s outlast ability", priority: 100, parse: activate_outlast } }
