//! "Whenever a spell you've cast is countered, [effect]" (Multani's Presence): triggers
//! when a spell its controller cast is countered (CR 701.6a) — a copy of a card cast by
//! an effect was cast (CR 707.12), a copy of a spell put onto the stack wasn't; the ability
//! looks back in time (CR 603.10e). A spell removed from the stack another way (exiled as
//! the turn ends, CR 724.1a) or that doesn't resolve because its targets are illegal
//! (CR 608.2b) isn't countered.

use super::TriggerPattern;
use crate::ability::*;
use crate::oracle::phrases::end;

fn spell_you_cast_countered(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    if !matches!(
        end(r),
        "a spell you've cast is countered" | "a spell you cast is countered"
    ) {
        return None;
    }
    Some((
        TriggerCond::SpellCountered(Filter::and(vec![
            Filter::Custom(crate::custom::WAS_CAST.into()),
            Filter::ControlledBy(PlayerRel::You),
        ])),
        Sel::TriggerObject,
        PlayerRef::You,
    ))
}

inventory::submit! { TriggerPattern { name: "a spell you've cast is countered", priority: 100, parse: spell_you_cast_countered } }
