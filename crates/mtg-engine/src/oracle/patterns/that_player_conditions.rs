//! Intervening "if" clauses about the player a trigger condition named ("At the
//! beginning of each opponent's end step, if that player has less than half their
//! starting life total, abandon this scheme." — You Cannot Hide from Me). "That player"
//! is the trigger's player; "their starting life total" is that player's own (CR 119.1),
//! and with a shared team life total it's the team's (CR 810.9, 904.13b).

use crate::ability::*;
use crate::oracle::phrases::end;

/// The condition `c` (lowercase, after "if ") about the trigger's player `who`.
pub fn that_player_condition(c: &str, who: &PlayerRef) -> Option<Condition> {
    let r = end(c).strip_prefix("that player ")?;
    if matches!(who, PlayerRef::You) || crate::oracle::patterns::oracle_hardening_referents::is_no_player_referent(who) {
        return None;
    }
    let filter = match r {
        "has less than half their starting life total"
        | "'s life total is less than half their starting life total" => {
            PlayerFilter::LessThanHalfStartingLife
        }
        _ => return None,
    };
    Some(Condition::PlayerMatches(who.clone(), filter))
}
