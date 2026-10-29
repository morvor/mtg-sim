//! Triggers, grammar V (cards4-triggers-durations): delayed triggered abilities and
//! triggers created by effects (CR 603.7).
//!
//! - "create X 1/1 Sand Warrior creature tokens ... at the beginning of your next upkeep,
//!   where X is the number of lands you control at that time" (Hazezon Tamar): a value
//!   the delayed ability determines as it resolves (CR 603.7c, 608.2h).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, Builder};
use crate::oracle::phrases::end;

/// "[instruction] at the beginning of your next upkeep, where X is [value] at that time":
/// the delayed ability's instruction, with X determined as it resolves (the core grammar
/// reads the X of a delayed instruction inside its body).
fn where_x_at_that_time(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let head = l.strip_suffix(" at that time")?;
    let (instruction, value) = head.split_once(", where x is ")?;
    let (trigger, inner) = super::triggers_delayed::split_delay(instruction)?;
    let saved = (b.targets.len(), b.it.clone());
    let effect = parse_effect_text(&format!("{inner}, where x is {value}"), b);
    // The delayed ability's own instruction: nothing it refers to comes from the creating
    // ability (no targets, objects or event of its own).
    let json = effect
        .as_ref()
        .and_then(|e| serde_json::to_string(e).ok())
        .unwrap_or_default();
    if effect.is_none()
        || b.targets.len() != saved.0
        || ["Target", "Trigger", "EventAmount", "\"This\""]
            .iter()
            .any(|p| json.contains(p))
    {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    }
    Some(Effect::DelayedTrigger {
        trigger,
        body: Box::new(Body::effect(effect?)),
        once: true,
    })
}

inventory::submit! { EffectPattern { name: "triggers 5: delayed instruction, where X is [value] at that time", priority: 100, parse: where_x_at_that_time } }

/// "If it's a creature, put a +1/+1 counter on it for each 1 damage prevented this way at
/// the beginning of the next end step." (Scars of the Veteran): an instruction with its
/// delay written after it, about what the previous instruction did ("it", "this way"),
/// read as the delay written first ("At the beginning of the next end step, put ...",
/// Sacred Boon), with a leading condition kept in front.
fn delay_after_referent_instruction(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    if !l.contains(" this way ") {
        return false;
    }
    let (lead, rest) = match l.strip_prefix("if ").and_then(|r| r.split_once(", ")) {
        Some((c, r)) => (format!("if {c}, "), r),
        None => (String::new(), l),
    };
    let Some(delay) = [
        "at the beginning of the next end step",
        "at the beginning of the next cleanup step",
        "at the beginning of your next upkeep",
    ]
    .into_iter()
    .find(|d| rest.ends_with(&format!(" {d}") as &str)) else {
        return false;
    };
    let instruction = rest[..rest.len() - delay.len()].trim_end();
    if instruction.contains(", ") {
        return false;
    }
    crate::oracle_ext::apply_followup_ext(&format!("{lead}{delay}, {instruction}"), prev, b)
}

inventory::submit! { FollowupPattern { name: "triggers 5: [instruction about this way] at the beginning of the next step", priority: 100, apply: delay_after_referent_instruction } }

/// One event of "when you next [events] this turn": "cast an instant spell", "activate a
/// loyalty ability", "activate an ability that isn't a mana ability".
fn next_event(e: &str) -> Option<TriggerCond> {
    if let Some(spell) = e.strip_prefix("cast ") {
        let (cast, _, _) = crate::oracle::triggers::parse_trigger_condition(&format!(
            "whenever you cast {spell}"
        ))?;
        return matches!(
            cast,
            TriggerCond::CastSpell {
                who: PlayerRel::You,
                ..
            }
        )
        .then_some(cast);
    }
    let ability = e.strip_prefix("activate ")?;
    let activated = TriggerCond::AbilityActivated {
        who: PlayerRel::You,
        source: Filter::Any,
        include_mana: false,
    };
    let with = |name: &str| TriggerCond::Where {
        trigger: Box::new(activated.clone()),
        cond: Condition::SelMatches(Sel::TriggerSpell, Filter::Custom(name.into())),
    };
    Some(match ability {
        "an ability" | "an ability that isn't a mana ability" => activated,
        "a loyalty ability" => with(crate::stack_ability_filters::LOYALTY_ABILITY),
        "an ability with {x} in its activation cost" => {
            with(crate::stack_ability_filters::X_IN_ACTIVATION_COST)
        }
        _ => return None,
    })
}

/// "When you next cast an instant spell, cast a sorcery spell, or activate a loyalty
/// ability this turn, copy that spell or ability twice." (Repeated Reverberation), "When
/// you next activate an ability that isn't a mana ability this turn by spending four or
/// more mana to activate it, copy that ability." (Dynaheir, Invoker Adept): a delayed
/// triggered ability that triggers once, on the first of those events this turn
/// (CR 603.7b, 603.7c).
fn when_you_next(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("when you next ")?;
    let (events, rest) = r.split_once(" this turn")?;
    // "... by spending four or more mana to activate it, ..."
    let (spent, eff) = match rest.strip_prefix(", ") {
        Some(e) => (None, e),
        None => {
            let r = rest.strip_prefix(" by spending ")?;
            let (n, r) = crate::oracle::phrases::parse_number(r)?;
            let Value::Const(n) = n else { return None };
            let eff = r.trim_start().strip_prefix("or more mana to activate it, ")?;
            (Some(n), eff)
        }
    };
    let items: Vec<&str> = events
        .split(", ")
        .flat_map(|x| x.split(" or "))
        .map(|x| x.strip_prefix("or ").unwrap_or(x).trim())
        .filter(|x| !x.is_empty())
        .collect();
    // A list of at least one spell and one ability, or a single event, is read here; the
    // plain "cast [spell] or activate [ability]" pair also by `next_cast_or_activate`.
    let mut conds: Vec<TriggerCond> = items.iter().map(|e| next_event(e)).collect::<Option<_>>()?;
    let any_ability = conds
        .iter()
        .any(|c| !matches!(c, TriggerCond::CastSpell { .. }));
    if let Some(n) = spent {
        // Only about an activated ability.
        if conds.len() != 1 || !any_ability {
            return None;
        }
        let c = conds.pop()?;
        conds.push(TriggerCond::Where {
            trigger: Box::new(c),
            cond: Condition::Custom(
                format!("{}{n}", crate::kw::activation_mana_spent::MANA_SPENT_TO_ACTIVATE).into(),
            ),
        });
    }
    if !any_ability {
        return None;
    }
    let trigger = if conds.len() == 1 {
        conds.pop()?
    } else {
        TriggerCond::AnyOf(conds)
    };
    let body =
        crate::oracle::effects::parse_trigger_body(eff, b.ctx, Sel::TriggerSpell, PlayerRef::You)?;
    if body.modal.is_some() {
        return None;
    }
    Some(Effect::DelayedTrigger {
        trigger: TriggerCond::ThisTurn(Box::new(trigger)),
        body: Box::new(body),
        once: true,
    })
}

inventory::submit! { EffectPattern { name: "triggers 5: when you next [cast a spell or activate an ability, ...] this turn", priority: 105, parse: when_you_next } }

/// "copy the next loyalty ability you activate this turn when you activate it" (Jaya's
/// Phoenix): "when you next activate a loyalty ability this turn, copy that ability".
fn copy_the_next_ability(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("copy the next ")?;
    let (ability, rest) = r.split_once(" you activate this turn when you activate it")?;
    let rest = rest.trim_start();
    let tail = if rest.is_empty() {
        String::new()
    } else {
        format!(" {rest}")
    };
    when_you_next(
        &format!("when you next activate a {ability} this turn, copy that ability{tail}"),
        b,
    )
}

inventory::submit! { EffectPattern { name: "triggers 5: copy the next [ability] you activate this turn when you activate it", priority: 105, parse: copy_the_next_ability } }
