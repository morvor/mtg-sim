//! "The top N cards of a library" for a count, worded the way English allows it: "the top
//! three cards of your library", but "that many cards from the top of your library" (an
//! amount carried over from the event or the instruction before can't stand between "the
//! top" and "cards").

use super::*;

impl Renderer<'_> {
    /// The top `n` cards of the library named by the possessive `p` ("your", "target
    /// player's"), with any "where X is ..." clause the count needs.
    pub(crate) fn top_cards(&mut self, n: &Value, p: &str) -> String {
        match n {
            Value::Const(1) => format!("the top card of {p} library"),
            Value::Const(k) => format!("the top {} cards of {p} library", number_word(*k)),
            // The amount of the event ("deals combat damage to a player, look at that many
            // cards") or the number the instruction before acted on.
            Value::EventAmount | Value::Prev => {
                format!("that many cards from the top of {p} library")
            }
            other => {
                let (v, w) = self.amount(other);
                format!("the top {v} cards of {p} library{}", w.unwrap_or_default())
            }
        }
    }
}
