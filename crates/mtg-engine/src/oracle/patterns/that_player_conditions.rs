//! Intervening "if" clauses about the player a trigger condition named ("At the
//! beginning of each opponent's end step, if that player has less than half their
//! starting life total, abandon this scheme." — You Cannot Hide from Me). "That player"
//! is the trigger's player; "their starting life total" is that player's own (CR 119.1),
//! and with a shared team life total it's the team's (CR 810.9, 904.13b).

use crate::ability::*;
use crate::oracle::phrases::end;

/// The condition `c` (lowercase, after "if ") about the trigger's player `who`.
pub fn that_player_condition(c: &str, who: &PlayerRef) -> Option<Condition> {
    if matches!(who, PlayerRef::You) || crate::oracle::patterns::oracle_hardening_referents::is_no_player_referent(who) {
        return None;
    }
    // "if they were attacked this turn by an Assassin you controlled" (Ramses).
    if let Some(r) = end(c).strip_prefix("they ") {
        return super::history_conditions_4::was_attacked_by(r, who);
    }
    let r = end(c).strip_prefix("that player ")?;
    let filter = match r {
        "has less than half their starting life total"
        | "'s life total is less than half their starting life total" => {
            PlayerFilter::LessThanHalfStartingLife
        }
        // "had another land enter the battlefield under their control this turn" (Tunnel
        // Ignus): besides the land the ability triggered on, so two or more this turn.
        r => {
            let x = r.strip_prefix("had another ")?;
            let (noun, rest) = x.split_once(' ')?;
            if !matches!(noun, "land" | "creature" | "artifact" | "permanent") {
                return None;
            }
            super::history_filters_4::player_history(&format!(
                "had two or more {noun}s {rest}"
            ))?
        }
    };
    Some(Condition::PlayerMatches(who.clone(), filter))
}
