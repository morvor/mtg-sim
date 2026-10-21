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
//!
//! Exact amounts: "Whenever another source you control deals exactly 1 damage to a
//! permanent or player, ~ deals 2 damage to that permanent or player." (Ghyrson Starn,
//! Kelermorph: one source dealing exactly that much to one recipient at once, each such
//! event triggering it, CR 603.2c), "Whenever one or more opponents each lose exactly 1
//! life" (Ob Nixilis, Captive Kingpin: each opponent's total loss in one batch of
//! simultaneous events, triggering once, [`BatchPer::AnyPlayer`]).

use super::{EffectPattern, TriggerPattern};
use crate::ability::*;
use crate::kw::trigger_events::{
    CAUSED_BY_EVENT_PLAYERS, CAUSED_BY_OPPONENTS, CAUSED_BY_SPELL_OR_ABILITY, THE_CAUSE,
};
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_number, parse_object_phrase};
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

/// "exactly N " / "N or more ": (comparison, N, rest).
fn amount(x: &str) -> Option<(Cmp, i32, &str)> {
    if let Some(y) = x.strip_prefix("exactly ") {
        let (n, rest) = parse_number(y)?;
        return Some((Cmp::Eq, n.as_const()?, rest.trim_start()));
    }
    let (n, rest) = parse_number(x)?;
    let rest = rest.trim_start().strip_prefix("or more ")?;
    Some((Cmp::Ge, n.as_const()?, rest))
}

/// Each damage event of the trigger (alternatives included) with a condition on its
/// amount, and its sources other than the ability's source ("another source").
fn qualify_damage(c: TriggerCond, amount: Option<(Cmp, i32)>, other: bool) -> Option<TriggerCond> {
    match c {
        TriggerCond::DealsDamage {
            source,
            to,
            combat_only,
        } => {
            let source = if other {
                Filter::and(vec![Filter::Other, source])
            } else {
                source
            };
            let d = TriggerCond::DealsDamage {
                source,
                to,
                combat_only,
            };
            Some(match amount {
                Some((cmp, n)) => TriggerCond::Where {
                    trigger: Box::new(d),
                    cond: Condition::Compare(Value::EventAmount, cmp, Value::c(n)),
                },
                None => d,
            })
        }
        TriggerCond::AnyOf(v) => Some(TriggerCond::AnyOf(
            v.into_iter()
                .map(|c| qualify_damage(c, amount, other))
                .collect::<Option<_>>()?,
        )),
        _ => None,
    }
}

/// "another source you control deals exactly 1 damage to a permanent or player", "a
/// source you control deals exactly 1 damage to a permanent or player": one event per
/// source and recipient (the simpler forms are read in `trigger_grammar_events.rs`).
fn damage_amounts(t: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let t = end(t);
    let (other, t) = match t.strip_prefix("another ") {
        Some(r) => (true, format!("a {r}")),
        None => (false, t.to_string()),
    };
    let (subj, x) = t.split_once(" deals ")?;
    let (amt, rest) = match amount(x) {
        Some((cmp, n, r)) => (Some((cmp, n)), r),
        None => (None, x),
    };
    let rest = rest.strip_prefix("damage to ")?;
    // (Without "another" or an amount, it's what the other patterns read.)
    if !other && amt.is_none() {
        return None;
    }
    let (c, it, p) = parse_trigger_condition(&format!("whenever {subj} deals damage to {rest}"))?;
    // Only what the other patterns don't read: "another", or alternatives with an amount.
    if !other && !matches!(c, TriggerCond::AnyOf(_)) {
        return None;
    }
    // "that permanent or player": the recipient of the damage, whichever it is (the
    // events' object, else their player).
    let (it, p) = if matches!(c, TriggerCond::AnyOf(_)) && rest == "a permanent or player" {
        (Sel::TriggerObject, PlayerRef::TriggerPlayer)
    } else {
        (it, p)
    };
    Some((qualify_damage(c, amt, other)?, it, p))
}

inventory::submit! { TriggerPattern { name: "trigger grammar 4: another source deals exactly N damage", priority: 110, parse: damage_amounts } }

/// "one or more opponents each lose exactly N life", "... each lose N or more life".
fn each_lose_life(t: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let t = end(t);
    let (who, r) = if let Some(r) = t.strip_prefix("one or more opponents each lose ") {
        (PlayerRel::Opponent, r)
    } else {
        (PlayerRel::Any, t.strip_prefix("one or more players each lose ")?)
    };
    let (cmp, n, rest) = amount(r)?;
    if rest != "life" {
        return None;
    }
    Some((
        TriggerCond::Where {
            trigger: Box::new(TriggerCond::Batched {
                trigger: Box::new(TriggerCond::LosesLife { who }),
                per: BatchPer::AnyPlayer,
            }),
            cond: Condition::Compare(Value::EventAmount, cmp, Value::c(n)),
        },
        Sel::None,
        PlayerRef::TriggerPlayer,
    ))
}

inventory::submit! { TriggerPattern { name: "trigger grammar 4: one or more opponents each lose exactly N life", priority: 100, parse: each_lose_life } }

/// "~ deals N damage to that permanent or player" in an ability that triggers on damage
/// dealt to a permanent or player: the permanent, or else the player, the trigger
/// event's damage was dealt to.
fn damage_to_that_permanent_or_player(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("~ deals ")?;
    let (n, r) = parse_number(r)?;
    if r.trim_start() != "damage to that permanent or player"
        || !b.in_trigger
        || !matches!(b.it, Sel::TriggerObject)
        || !matches!(b.it_player, PlayerRef::TriggerPlayer)
    {
        return None;
    }
    let deal = |to: Sel| Effect::DealDamage {
        source: Sel::This,
        amount: n.clone(),
        to,
    };
    Some(Effect::If {
        cond: Condition::Compare(
            Value::CountSel(Box::new(Sel::TriggerObject)),
            Cmp::Ge,
            Value::c(1),
        ),
        then: Box::new(deal(Sel::TriggerObject)),
        otherwise: Box::new(deal(Sel::Players(PlayerRef::TriggerPlayer))),
    })
}

inventory::submit! { EffectPattern { name: "~ deals N damage to that permanent or player (the damaged one)", priority: 100, parse: damage_to_that_permanent_or_player } }

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

    #[test]
    fn reads_exact_amounts() {
        let (c, _, _) = damage_amounts(
            "another source you control deals exactly 1 damage to a permanent or player",
        )
        .expect("ghyrson");
        assert!(matches!(c, TriggerCond::AnyOf(ref v) if v.len() == 2));
        assert!(damage_amounts("a source you control deals damage to a creature").is_none());
        assert!(matches!(
            each_lose_life("one or more opponents each lose exactly 1 life"),
            Some((TriggerCond::Where { .. }, _, _))
        ));
    }
}
