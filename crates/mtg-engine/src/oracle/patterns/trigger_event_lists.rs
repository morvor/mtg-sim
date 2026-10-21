//! Trigger conditions listing several events (CR 603.2, 603.2c): "Whenever another
//! creature dies, or a creature card is put into a graveyard from anywhere other than the
//! battlefield, or a creature card leaves your graveyard" (Syr Konrad, the Grim), and
//! lists sharing one subject: "Whenever an opponent attacks you with two or more
//! creatures, draws their second card each turn, or casts their second spell each turn"
//! (Trouble in Pairs), "Whenever an opponent sacrifices a nontoken permanent or discards a
//! permanent card" (Tergrid, God of Fright), "Whenever an opponent scries, surveils, or
//! searches their library" (River Song). The ability triggers once for each event that
//! matches one of them ([`TriggerCond::AnyOf`]).

use super::TriggerPattern;
use crate::ability::*;

type Parsed = (TriggerCond, Sel, PlayerRef);

fn condition(s: &str) -> Option<Parsed> {
    crate::oracle::triggers::parse_trigger_condition(&format!("whenever {s}"))
}

/// The items of "A, B, or C" / "A or B" / "A, or B, or C".
fn list_items(r: &str) -> Vec<Vec<&str>> {
    let mut out = Vec::new();
    // "A, or B, or C" (Syr Konrad).
    if r.contains(", or ") {
        let v: Vec<&str> = r.split(", or ").collect();
        if v.len() >= 2 {
            out.push(v);
        }
    }
    // "A, B, or C".
    if let Some((head, last)) = r.rsplit_once(", or ") {
        let mut v: Vec<&str> = head.split(", ").collect();
        v.push(last);
        if v.len() >= 3 {
            out.push(v);
        }
    }
    // "A or B": each " or " in turn.
    for (i, _) in r.match_indices(" or ") {
        out.push(vec![&r[..i], &r[i + 4..]]);
    }
    out
}

fn same(x: &dyn std::fmt::Debug, y: &dyn std::fmt::Debug) -> bool {
    format!("{x:?}") == format!("{y:?}")
}

/// The parsed conditions, if each item is a whole trigger condition, or each later item
/// is a verb phrase of the first item's subject.
fn conditions(items: &[&str]) -> Option<Vec<Parsed>> {
    if let Some(v) = items
        .iter()
        .map(|s| condition(s))
        .collect::<Option<Vec<_>>>()
    {
        return Some(v);
    }
    // A shared subject: the first words of the first item, the longest reading first
    // ("one or more creatures you control", not "one or more creatures").
    let words: Vec<&str> = items[0].split(' ').collect();
    for k in (1..=6.min(words.len().saturating_sub(1))).rev() {
        let subject = words[..k].join(" ");
        let v: Option<Vec<Parsed>> = items
            .iter()
            .enumerate()
            .map(|(i, s)| {
                if i == 0 {
                    condition(s)
                } else {
                    condition(&format!("{subject} {s}"))
                }
            })
            .collect();
        if v.is_some() {
            return v;
        }
    }
    None
}

fn event_list(r: &str) -> Option<Parsed> {
    let r = r.trim();
    for items in list_items(r) {
        if items.iter().any(|s| s.trim().is_empty()) {
            continue;
        }
        let Some(parsed) = conditions(&items) else {
            continue;
        };
        // Batched events combine only with batched events of the same grouping (each
        // batch of events triggers it once, `triggers.rs`).
        let batched: Vec<bool> = parsed
            .iter()
            .map(|p| super::trigger_grammar_events::is_batched(&p.0))
            .collect();
        if batched.iter().any(|b| *b) {
            let per = |c: &TriggerCond| match c {
                TriggerCond::Batched { per, .. } => Some(*per),
                _ => None,
            };
            let first = per(&parsed[0].0);
            if !batched.iter().all(|b| *b)
                || first.is_none()
                || parsed.iter().any(|p| per(&p.0) != first)
            {
                return None;
            }
        }
        // "that card": the card a sacrificed permanent became in its graveyard, or the
        // discarded card: both the event's object (`EventInfo::object`).
        fn inner(c: &TriggerCond) -> &TriggerCond {
            match c {
                TriggerCond::Where { trigger, .. } => inner(trigger),
                other => other,
            }
        }
        let card_events = parsed.iter().all(|p| {
            matches!(p.1, Sel::TriggerLki | Sel::TriggerObject)
                && matches!(
                    inner(&p.0),
                    TriggerCond::Sacrificed(_)
                        | TriggerCond::YouSacrifice(_)
                        | TriggerCond::Discards { .. }
                )
        });
        let it = if parsed.iter().all(|p| same(&p.1, &parsed[0].1)) {
            parsed[0].1.clone()
        } else if card_events {
            Sel::TriggerObject
        } else {
            Sel::None
        };
        // No single "that player" when the events name different ones (a body that
        // refers to one isn't read, see `oracle/triggers.rs`).
        let player = if parsed.iter().all(|p| same(&p.2, &parsed[0].2)) {
            parsed[0].2.clone()
        } else {
            PlayerRef::Iterated
        };
        // An item that is itself several events ("a creature card is put into a graveyard
        // from anywhere other than the battlefield") contributes each of them.
        let mut conds = Vec::new();
        for p in parsed {
            match p.0 {
                TriggerCond::AnyOf(v) => conds.extend(v),
                c => conds.push(c),
            }
        }
        // "Whenever one or more creatures you control fight or become blocked": one
        // trigger event with alternatives, so a batch matching several of them triggers
        // it once (CR 603.2c), with all of their events.
        if batched.iter().all(|b| *b) {
            let mut inner = Vec::new();
            let mut per = None;
            for c in &conds {
                match c {
                    TriggerCond::Batched { trigger, per: p }
                        if !matches!(**trigger, TriggerCond::FirstTimeEachTurn(_)) =>
                    {
                        per = Some(*p);
                        inner.push((**trigger).clone());
                    }
                    _ => return None,
                }
            }
            return Some((
                TriggerCond::Batched {
                    trigger: Box::new(TriggerCond::AnyOf(inner)),
                    per: per?,
                },
                it,
                player,
            ));
        }
        return Some((TriggerCond::AnyOf(conds), it, player));
    }
    None
}

inventory::submit! { TriggerPattern { name: "event lists: A, B, or C", priority: 905, parse: event_list } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists() {
        for t in [
            "an opponent attacks you with two or more creatures, draws their second card each turn, or casts their second spell each turn",
            "an opponent sacrifices a nontoken permanent or discards a permanent card",
            "another creature dies, or a creature card is put into a graveyard from anywhere other than the battlefield, or a creature card leaves your graveyard",
            "an opponent scries, surveils, or searches their library",
        ] {
            let (c, _, _) = event_list(t).unwrap_or_else(|| panic!("{t}"));
            assert!(matches!(c, TriggerCond::AnyOf(_)), "{t}");
        }
        // One description with "or" in it isn't a list.
        assert!(event_list("a creature or planeswalker you control dies").is_none());
    }
}
