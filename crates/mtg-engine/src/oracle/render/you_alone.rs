//! Instructions "for each player" where the only player is you ("you lose half your
//! life, rounded up", "shuffle the cards from your hand into your library, then draw that
//! many cards"): performed once, as you, so "that player" in the body is you.

use super::*;

/// The body of `ForEachPlayer { who: You, effect }` with the iterated player read as you,
/// or `None` if the body has its own "each player" instruction (which rebinds it).
pub(super) fn iterated_as_you(e: &Effect) -> Option<Effect> {
    fn walk(v: &mut serde_json::Value) -> bool {
        match v {
            serde_json::Value::String(s) if s == "Iterated" => {
                // `PlayerRef::Iterated` and `PlayerRel::Iterated`.
                *s = "You".into();
                true
            }
            serde_json::Value::Array(a) => a.iter_mut().all(walk),
            serde_json::Value::Object(m) => {
                if m.contains_key("ForEachPlayer") {
                    return false;
                }
                m.values_mut().all(walk)
            }
            _ => true,
        }
    }
    let mut v = serde_json::to_value(e).ok()?;
    if !walk(&mut v) {
        return None;
    }
    serde_json::from_value(v).ok()
}
