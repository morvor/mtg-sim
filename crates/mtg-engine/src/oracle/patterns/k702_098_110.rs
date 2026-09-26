//! Oracle text of the keywords of CR 702.98–702.110 that the generic keyword parser
//! doesn't handle, and phrases that go with them:
//!
//! * "if it's attacking the player with the most life or tied for most life" (CR 702.105a);
//! * "Whenever you activate ~'s outlast ability" (CR 702.107a);
//! * "When ~ exploits a creature", "Whenever a creature you control exploits a [quality]
//!   creature" (CR 702.110b).

use super::{ConditionPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::phrases::{end, parse_object_phrase};

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

/// "When ~ exploits a creature", "Whenever a creature you control exploits a [quality]
/// creature" (CR 702.110b). "It" is the exploited creature.
fn exploits(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let (who, what) = r.split_once(" exploits ")?;
    let name = match who {
        "~" => crate::kw::exploit::THIS_EXPLOITS,
        "a creature you control" => crate::kw::exploit::YOURS_EXPLOITS,
        _ => return None,
    };
    let mut trigger = TriggerCond::Custom(name.into());
    if what != "a creature" {
        let what = what
            .strip_prefix("a ")
            .or_else(|| what.strip_prefix("an "))?;
        let (f, _, tail) = parse_object_phrase(what)?;
        if !end(tail).is_empty() {
            return None;
        }
        // The exploited creature as it last existed on the battlefield.
        trigger = TriggerCond::Where {
            trigger: Box::new(trigger),
            cond: Condition::SelMatches(Sel::TriggerLki, f),
        };
    }
    Some((trigger, Sel::TriggerObject, PlayerRef::You))
}

inventory::submit! { TriggerPattern { name: "[creature] exploits a creature", priority: 100, parse: exploits } }
