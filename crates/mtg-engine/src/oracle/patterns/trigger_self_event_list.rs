//! Trigger conditions listing three or more events of the source: "Whenever ~ attacks,
//! blocks, or becomes the target of a spell, ..." (Giggling Skitterspike). Each listed
//! event must be a whole trigger condition of its own ("~ attacks", "~ blocks", "~
//! becomes the target of a spell"); the ability triggers once for each event matching
//! any of them (CR 603.2, 603.2c). Two events joined by "or" are parsed elsewhere.

use super::TriggerPattern;
use crate::ability::*;

type Parsed = (TriggerCond, Sel, PlayerRef);

fn self_event_list(r: &str) -> Option<Parsed> {
    let rest = r.strip_prefix("~ ")?;
    let (head, last) = rest.rsplit_once(", or ")?;
    let mut parts: Vec<&str> = head.split(", ").collect();
    parts.push(last);
    if parts.len() < 3 {
        return None;
    }
    let mut conds = Vec::new();
    let mut it: Option<Sel> = None;
    let mut player: Option<PlayerRef> = None;
    let dbg = |x: &dyn std::fmt::Debug| format!("{x:?}");
    for p in parts {
        let (c, i, pl) =
            crate::oracle::triggers::parse_trigger_condition(&format!("whenever ~ {p}"))?;
        // When the listed events give "it" or "that player" different meanings (Giggling
        // Skitterspike: the player who targeted it, or you), the effect may not use that
        // word: it has no antecedent, so an effect mentioning it doesn't compile.
        it = match it {
            Some(prev) if dbg(&prev) != dbg(&i) => Some(super::oracle_hardening_referents::no_referent()),
            Some(prev) => Some(prev),
            None => Some(i),
        };
        player = match player {
            Some(prev) if dbg(&prev) != dbg(&pl) => {
                Some(super::oracle_hardening_referents::no_player_referent())
            }
            Some(prev) => Some(prev),
            None => Some(pl),
        };
        match c {
            TriggerCond::AnyOf(v) => conds.extend(v),
            c => conds.push(c),
        }
    }
    let (it, player) = (it?, player?);
    Some((TriggerCond::AnyOf(conds), it, player))
}

inventory::submit! { TriggerPattern { name: "~ [event], [event], or [event]", priority: 900, parse: self_event_list } }
