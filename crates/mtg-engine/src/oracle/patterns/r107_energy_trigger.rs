//! "Whenever you get one or more {E}" (CR 107.14, 122.1): energy counters are put on
//! you, by any spell, ability or cost (Aether Revolt, Fabrication Module, Territorial
//! Gorger); "... during your turn" (Brotherhood Scribe) only on your turn. One trigger for
//! each event, however many {E} you get at once; "that much"/"that many" is how many
//! ([`TriggerCond::CountersPutBy`]).

use super::TriggerPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::types::counters;

fn get_energy(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = end(r).strip_prefix("you get one or more {e}")?.trim();
    let your_turn = match r {
        "" => false,
        "during your turn" => true,
        _ => return None,
    };
    let mut cond = TriggerCond::CountersPutBy {
        who: PlayerRel::Any,
        on_objects: None,
        on_players: Some(PlayerFilter::You),
        kind: Some(counters::ENERGY.into()),
        each: false,
    };
    if your_turn {
        cond = TriggerCond::Where {
            trigger: Box::new(cond),
            cond: Condition::YourTurn,
        };
    }
    Some((cond, Sel::This, PlayerRef::You))
}

inventory::submit! { TriggerPattern { name: "r107 you get one or more energy", priority: 100, parse: get_energy } }
