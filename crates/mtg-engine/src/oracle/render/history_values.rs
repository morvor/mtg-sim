//! Amounts from this turn's events ([`Value::EventsThisTurn`]), rendered the way the
//! value grammar reads them (`oracle/patterns/value_results.rs`).

use super::*;

impl Renderer<'_> {
    pub(crate) fn events_this_turn(&mut self, c: &TriggerCond, t: Tally) -> String {
        let _ = t;
        self.gap(format!("EventsThisTurn({c:?})"))
    }
}
