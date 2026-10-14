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
    let mut same_it = true;
    let mut same_player = true;
    for p in parts {
        let (c, i, pl) =
            crate::oracle::triggers::parse_trigger_condition(&format!("whenever ~ {p}"))?;
        let dbg = |x: &dyn std::fmt::Debug| format!("{x:?}");
        match &it {
            Some(prev) if dbg(prev) != dbg(&i) => same_it = false,
            Some(_) => {}
            None => it = Some(i),
        }
        match &player {
            Some(prev) if dbg(prev) != dbg(&pl) => same_player = false,
            Some(_) => {}
            None => player = Some(pl),
        }
        match c {
            TriggerCond::AnyOf(v) => conds.extend(v),
            c => conds.push(c),
        }
    }
    let it = if same_it { it? } else { Sel::None };
    let player = if same_player {
        player?
    } else {
        PlayerRef::Iterated
    };
    Some((TriggerCond::AnyOf(conds), it, player))
}

inventory::submit! { TriggerPattern { name: "~ [event], [event], or [event]", priority: 900, parse: self_event_list } }
