//! Trigger events, grammar IV (cards3-triggers-durations).
//!
//! Events qualified by the spell or ability that caused them (CR 603.2; see
//! `kw/trigger_events.rs`): "[cause] causes [subject] to [verb phrase]", read as the
//! trigger "[subject] [verb phrase]" with a condition on its cause:
//! - "a spell or ability causes a player to shuffle their library" (Psychogenic Probe);
//! - "a spell or ability causes its controller to shuffle their library" (Widespread
//!   Panic): the player who shuffles controls the spell or ability;
//! - "a spell or ability an opponent controls causes a land to be put into your graveyard
//!   from the battlefield" (Sacred Ground);
//! - "a white instant or sorcery spell causes you to gain life" (Firesong and
//!   Sunspeaker): a spell described by an object phrase.

use super::TriggerPattern;
use crate::ability::*;
use crate::kw::trigger_events::{
    CAUSED_BY_EVENT_PLAYERS, CAUSED_BY_OPPONENTS, CAUSED_BY_SPELL_OR_ABILITY, THE_CAUSE,
};
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::triggers::parse_trigger_condition;

/// "[subject] to [verb phrase]" as a clause: "a player to shuffle their library" → "a
/// player shuffles their library", "a land to be put into ..." → "a land is put into
/// ...", "you to gain life" → "you gain life".
fn clause(s: &str) -> Option<String> {
    let (subj, vp) = s.split_once(" to ")?;
    if subj == "you" {
        return Some(format!("you {vp}"));
    }
    if let Some(r) = vp.strip_prefix("be ") {
        return Some(format!("{subj} is {r}"));
    }
    let (verb, rest) = vp.split_once(' ').unwrap_or((vp, ""));
    let verb = if verb.ends_with('s') || verb.ends_with("sh") || verb.ends_with("ch") {
        format!("{verb}es")
    } else {
        format!("{verb}s")
    };
    Some(format!("{subj} {verb} {rest}").trim_end().to_string())
}

fn caused_event(t: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let t = end(t);
    let (cause, rest) = t.split_once(" causes ")?;
    // "its controller": the spell or ability's controller is the event's player.
    let (rest, own) = match rest.strip_prefix("its controller to ") {
        Some(r) => (format!("a player to {r}"), true),
        None => (rest.to_string(), false),
    };
    let cond = match cause {
        "a spell or ability" => Condition::Custom(CAUSED_BY_SPELL_OR_ABILITY.into()),
        "a spell or ability an opponent controls" => Condition::Custom(CAUSED_BY_OPPONENTS.into()),
        _ => {
            // A spell described by an object phrase: "a white instant or sorcery spell".
            let np = cause.strip_prefix("a ").or_else(|| cause.strip_prefix("an "))?;
            let (f, plural, tail) = parse_object_phrase(np)?;
            if plural || !end(tail).is_empty() || !np.ends_with(" spell") {
                return None;
            }
            Condition::Compare(
                Value::CountSel(Box::new(Sel::All(Filter::and(vec![
                    Filter::Custom(THE_CAUSE.into()),
                    f,
                ])))),
                Cmp::Ge,
                Value::Const(1),
            )
        }
    };
    let cond = if own {
        Condition::And(vec![cond, Condition::Custom(CAUSED_BY_EVENT_PLAYERS.into())])
    } else {
        cond
    };
    let (trigger, it, player) = parse_trigger_condition(&format!("whenever {}", clause(&rest)?))?;
    // An event of the kinds read here, not a batch or a qualified trigger.
    if matches!(trigger, TriggerCond::Where { .. } | TriggerCond::Batched { .. }) {
        return None;
    }
    Some((
        TriggerCond::Where {
            trigger: Box::new(trigger),
            cond,
        },
        it,
        player,
    ))
}

inventory::submit! { TriggerPattern { name: "trigger grammar 4: [a spell or ability] causes [an event]", priority: 100, parse: caused_event } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_caused_events() {
        assert_eq!(clause("a player to shuffle their library").as_deref(), Some("a player shuffles their library"));
        assert_eq!(
            clause("a land to be put into your graveyard from the battlefield").as_deref(),
            Some("a land is put into your graveyard from the battlefield")
        );
        for t in [
            "a spell or ability causes a player to shuffle their library",
            "a spell or ability causes its controller to shuffle their library",
            "a spell or ability an opponent controls causes a land to be put into your graveyard from the battlefield",
            "a white instant or sorcery spell causes you to gain life",
        ] {
            assert!(
                matches!(caused_event(t), Some((TriggerCond::Where { .. }, _, _))),
                "{t}"
            );
        }
        assert!(caused_event("a creature causes you to gain life").is_none());
    }
}
