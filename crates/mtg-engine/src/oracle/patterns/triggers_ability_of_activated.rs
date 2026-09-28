//! Triggers on activating an ability of certain permanents (CR 602.2), with the
//! intervening "if it isn't a mana ability" (CR 603.4; mana abilities never trigger
//! these): "Whenever you activate an ability of an artifact, if it isn't a mana ability,
//! ..." (Kurkesh, Onakke Ancient), "Whenever an ability of equipped creature is
//! activated, if it isn't a mana ability, ..." (Illusionist's Bracers). The body's "that
//! ability" is the ability activated.

use super::TriggerPattern;
use crate::ability::*;
use crate::oracle::phrases::{end, parse_object_phrase};

fn ability_of_activated(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = end(r).strip_suffix(", if it isn't a mana ability")?;
    let (who, source) = if let Some(x) = r.strip_prefix("you activate an ability of ") {
        let x = x.strip_prefix("a ").or_else(|| x.strip_prefix("an "))?;
        let (f, plural, tail) = parse_object_phrase(x)?;
        if plural || !end(tail).is_empty() {
            return None;
        }
        (PlayerRel::You, f)
    } else {
        let x = r
            .strip_prefix("an ability of ")?
            .strip_suffix(" is activated")?;
        let f = match x {
            "equipped creature" | "enchanted creature" => Filter::AttachedToSource,
            "~" => Filter::Source,
            _ => return None,
        };
        (PlayerRel::Any, f)
    };
    Some((
        TriggerCond::AbilityActivated {
            who,
            source,
            include_mana: false,
        },
        Sel::TriggerSpell,
        PlayerRef::TriggerPlayer,
    ))
}

inventory::submit! { TriggerPattern { name: "an ability of [permanent] is activated, if it isn't a mana ability", priority: 100, parse: ability_of_activated } }
