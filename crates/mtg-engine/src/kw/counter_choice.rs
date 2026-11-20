//! Choosing what gets counters (CR 122.1, 614.17): "each player may put two +1/+1 counters on
//! a creature they control" (Agitator Ant) offers only the objects that can have counters of
//! that kind put on them. Agitator Ant's ruling: "If a creature can't have +1/+1 counters put
//! on it, its controller can't choose it just to have it become goaded." Register
//! `counter-choice-excludes-what-cant-have-them`.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::ObjectId;

/// `Filter::Custom` prefix: the object can have counters of the kind that follows put on it
/// ([`crate::counter_rules::counters_prevented`] doesn't stop them).
const CAN_HAVE: &str = "counters:can have ";

/// The choice of `what` to put `kind` counters on, narrowed to the objects that can have
/// them; any other selection is returned as it is.
pub fn chosen_for_counters(what: &Sel, kind: &str) -> Sel {
    match what {
        Sel::Choose {
            chooser,
            filter,
            count,
            up_to,
            store,
        } => Sel::Choose {
            chooser: chooser.clone(),
            filter: Filter::And(vec![
                filter.clone(),
                Filter::Custom(format!("{CAN_HAVE}{kind}").into()),
            ]),
            count: count.clone(),
            up_to: *up_to,
            store: *store,
        },
        other => other.clone(),
    }
}

struct CounterChoice;

impl KeywordRules for CounterChoice {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        let kind = name.strip_prefix(CAN_HAVE)?;
        Some(!crate::counter_rules::counters_prevented(g, id, kind))
    }
}

inventory::submit! { KeywordRegistration(&CounterChoice) }
