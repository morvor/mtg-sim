//! What destroyed (CR 701.8) or countered (CR 701.6) something: the spell or ability whose
//! effect did it, and its controller. A spell or ability destroys a permanent only if
//! its text says "destroy" (CR 701.8b: not by sacrificing, exiling, or lethal damage's
//! state-based action), and counters a spell only if its text says "counter".
//!
//! * "Whenever a spell or ability an opponent controls destroys a noncreature permanent
//!   you control, …" (Karmic Justice; looks back in time, so it triggers on its own
//!   destruction, CR 603.10a): "it" is the permanent, "that opponent" the controller of
//!   the spell or ability.
//! * "If a noncreature permanent under your control was destroyed this turn by a spell or
//!   ability an opponent controlled" (Cobra Trap), "If a creature spell you cast this turn
//!   was countered by a spell or ability an opponent controlled" (Summoning Trap): such an
//!   event happened this turn.

use super::{ConditionPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::patterns::triggers::parse_subject;
use crate::oracle::phrases::{end, parse_object_phrase};

/// "a spell or ability [an opponent / you] control(s/led)": the player relation.
fn spell_or_ability(s: &str) -> Option<(PlayerRel, &str)> {
    let r = s.strip_prefix("a spell or ability ")?;
    for (p, rel) in [
        ("an opponent controls", PlayerRel::Opponent),
        ("an opponent controlled", PlayerRel::Opponent),
        ("you control", PlayerRel::You),
        ("you controlled", PlayerRel::You),
    ] {
        if let Some(r) = r.strip_prefix(p) {
            return Some((rel, r));
        }
    }
    Some((PlayerRel::Any, r))
}

/// "a spell or ability you control counters a spell" (Baral, Chief of Compliance): "it"
/// is the countered spell (as it last existed on the stack), "that player" the controller
/// of the spell or ability that countered it.
fn counters_trigger(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let (by, r) = spell_or_ability(end(r))?;
    let spell = one_object(r.strip_prefix(" counters ")?)?;
    if !is_spell_filter(&spell) {
        return None;
    }
    Some((
        TriggerCond::CounteredBy { filter: spell, by },
        Sel::TriggerObject,
        PlayerRef::TriggerPlayer,
    ))
}

inventory::submit! { TriggerPattern { name: "r701 a spell or ability [player] controls counters a spell", priority: 100, parse: counters_trigger } }

/// The filter describes spells ("a spell", "a creature spell").
fn is_spell_filter(f: &Filter) -> bool {
    match f {
        Filter::Spell => true,
        Filter::And(v) => v.iter().any(|f| matches!(f, Filter::Spell)),
        _ => false,
    }
}

/// "a spell or ability an opponent controls destroys [permanent]".
fn destroys_trigger(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let (by, r) = spell_or_ability(end(r))?;
    let subj = parse_subject(r.strip_prefix(" destroys ")?)?;
    if subj.self_only || subj.one_or_more {
        return None;
    }
    Some((
        TriggerCond::DestroyedBy {
            filter: subj.filter,
            by,
        },
        Sel::TriggerObject,
        PlayerRef::TriggerPlayer,
    ))
}

inventory::submit! { TriggerPattern { name: "r701 a spell or ability [player] controls destroys [permanent]", priority: 100, parse: destroys_trigger } }

/// "Whenever a spell or ability an opponent controls destroys …, you may destroy target
/// permanent that opponent controls": "that opponent" is the trigger's player, the
/// controller of the spell or ability (read as "that player").
fn destroys_trigger_that_opponent(
    text: &str,
    ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    let (head, body) = text.split_once(", ")?;
    let cond = head.strip_prefix("Whenever ")?.to_lowercase();
    if !matches!(
        destroys_trigger(&cond),
        Some((
            TriggerCond::DestroyedBy {
                by: PlayerRel::Opponent,
                ..
            },
            ..
        ))
    ) || !body.contains("that opponent")
    {
        return None;
    }
    let text = format!("{head}, {}", body.replace("that opponent", "that player"));
    Some(vec![crate::oracle::triggers::parse_triggered(&text, ctx)?])
}

inventory::submit! { super::AbilityPattern { name: "r701 a spell or ability an opponent controls destroys ...: that opponent", priority: 100, parse: destroys_trigger_that_opponent } }

/// "a/an [object]" (singular).
fn one_object(s: &str) -> Option<Filter> {
    let s = s.strip_prefix("a ").or_else(|| s.strip_prefix("an "))?;
    let (f, plural, tail) = parse_object_phrase(s)?;
    (!plural && end(tail).is_empty()).then_some(f)
}

/// "[a permanent] (under your control) was destroyed this turn by a spell or ability an
/// opponent controlled", "[a creature spell] you cast this turn was countered by a spell
/// or ability an opponent controlled".
fn destroyed_or_countered_this_turn(c: &str) -> Option<Condition> {
    let c = end(c);
    if let Some((what, by)) = c.split_once(" was destroyed this turn by ") {
        let (by, tail) = spell_or_ability(by)?;
        if !tail.is_empty() {
            return None;
        }
        let (what, yours) = match what.strip_suffix(" under your control") {
            Some(w) => (w, true),
            None => (what, false),
        };
        let mut parts = vec![one_object(what)?];
        if yours {
            parts.push(Filter::ControlledBy(PlayerRel::You));
        }
        return Some(Condition::AllTriggerConditionsThisTurn(vec![
            TriggerCond::DestroyedBy {
                filter: Filter::and(parts),
                by,
            },
        ]));
    }
    let (what, by) = c.split_once(" was countered by ")?;
    let (by, tail) = spell_or_ability(by)?;
    if !tail.is_empty() {
        return None;
    }
    // "a creature spell you cast this turn": a spell you cast (not a copy put onto the
    // stack, CR 707.12); a spell countered this turn was on the stack this turn.
    let what = what.strip_suffix(" you cast this turn")?;
    let spell = one_object(what)?;
    if !is_spell_filter(&spell) {
        return None;
    }
    Some(Condition::AllTriggerConditionsThisTurn(vec![
        TriggerCond::CounteredBy {
            filter: Filter::and(vec![
                spell,
                Filter::Custom(crate::custom::WAS_CAST.into()),
                Filter::ControlledBy(PlayerRel::You),
            ]),
            by,
        },
    ]))
}

inventory::submit! { ConditionPattern { name: "r701 destroyed/countered this turn by a spell or ability [player] controlled", priority: 100, parse: destroyed_or_countered_this_turn } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses() {
        assert!(destroys_trigger(
            "a spell or ability an opponent controls destroys a noncreature permanent you control"
        )
        .is_some());
        assert!(counters_trigger("a spell or ability you control counters a spell").is_some());
        for c in [
            "a noncreature permanent under your control was destroyed this turn by a spell or ability an opponent controlled",
            "a creature spell you cast this turn was countered by a spell or ability an opponent controlled",
        ] {
            assert!(destroyed_or_countered_this_turn(c).is_some(), "{c}");
        }
    }
}
