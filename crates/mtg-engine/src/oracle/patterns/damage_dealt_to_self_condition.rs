//! "if N or more damage was dealt to it this turn" (Burning-Eye Zubera's dies trigger):
//! the total damage dealt this turn, from all sources, to the object the trigger is about
//! (as it last existed on the battlefield, CR 603.10a) or to the source ("~"), compared
//! with N (CR 603.4).

use super::ConditionPattern;
use crate::ability::*;
use crate::oracle::phrases::*;

fn damage_dealt_to_self(l: &str) -> Option<Condition> {
    let l = end(l);
    let (n, r) = parse_number(l)?;
    let r = r.trim_start().strip_prefix("or more damage was dealt to ")?;
    let (to, r) = if let Some(r) = r.strip_prefix("~") {
        (Filter::Source, r)
    } else {
        let r = r.strip_prefix("it")?;
        (Filter::In(Box::new(Sel::TriggerLki)), r)
    };
    if r != " this turn" {
        return None;
    }
    let total = Value::EventsThisTurn(
        Box::new(TriggerCond::DealsDamage {
            source: Filter::Any,
            to: DamageRecipient::Object(to),
            combat_only: false,
        }),
        Tally::Amount,
    );
    Some(Condition::Compare(total, Cmp::Ge, n))
}

inventory::submit! { ConditionPattern { name: "N or more damage was dealt to ~ this turn", priority: 300, parse: damage_dealt_to_self } }
