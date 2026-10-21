//! More history conditions (CR 603.4, 608.2c), read with the history clause grammar
//! (`history_conditions::history_condition`):
//!
//! * "if a player other than you lost life this turn" (Ludevic, Necro-Alchemist): some
//!   player other than you did it ([`PlayerFilter::History`]).
//! * "as long as two or more creatures died under your control this turn" (Gimli,
//!   Mournful Avenger): "two or more creatures you controlled died this turn" (CR 700.4;
//!   whose control is judged as they died, CR 603.10a).

use super::history_conditions::history_condition;
use super::history_filters_4::player_history;
use crate::ability::*;
use crate::oracle::phrases::end;

/// "a player other than you [did something] [window]", "an opponent other than ..."
fn other_than_you(c: &str) -> Option<Condition> {
    let c = end(c);
    let (base, r) = if let Some(r) = c.strip_prefix("a player other than you ") {
        (PlayerFilter::NotYou, r)
    } else {
        return None;
    };
    let did = player_history(r)?;
    Some(Condition::Compare(
        Value::CountPlayers(PlayerFilter::And(vec![base, did])),
        Cmp::Ge,
        Value::Const(1),
    ))
}

inventory::submit! { super::ConditionPattern { name: "history 4: a player other than you [did something] this turn", priority: 900, parse: other_than_you } }

/// "[objects] died under your control [window]" / "[objects] left the battlefield under
/// your control [window]": "[objects] you controlled died [window]".
fn under_your_control(c: &str) -> Option<Condition> {
    let c = end(c);
    for verb in ["died", "left the battlefield"] {
        let pat = format!(" {verb} under your control ");
        let Some(i) = c.find(&pat) else { continue };
        let (np, rest) = (&c[..i], &c[i + pat.len()..]);
        if np.contains(" you ") || np.contains(" control") || np.starts_with("~") {
            return None;
        }
        return history_condition(&format!("{np} you controlled {verb} {rest}"));
    }
    None
}

inventory::submit! { super::ConditionPattern { name: "history 4: [objects] died under your control this turn", priority: 900, parse: under_your_control } }
