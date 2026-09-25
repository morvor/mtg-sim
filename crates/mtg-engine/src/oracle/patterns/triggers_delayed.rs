//! Delayed triggered abilities created by effects (CR 603.7): "draw a card at the
//! beginning of the next turn's upkeep", "at the beginning of the next end step, sacrifice
//! ~", "... at end of combat".
//!
//! The delayed ability resolves long after the effect that created it, so everything its
//! effect refers to from the creating ability — targets, the triggering event's objects and
//! players, "that much", and the source itself — is captured into variables when the
//! delayed trigger is created (CR 603.7c: it still affects those objects, and only while
//! they remain in the zone they're expected to be in).

use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::patterns::{AbilityPattern, EffectPattern};
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

inventory::submit! {
    EffectPattern { name: "delayed trigger (next upkeep / end step / end of combat)", priority: 100, parse: delayed_trigger }
}

inventory::submit! {
    AbilityPattern { name: "delayed trigger line of an instant or sorcery", priority: 100, parse: spell_delayed_line }
}

inventory::submit! {
    EffectPattern { name: "you lose/win the game", priority: 100, parse: lose_or_win_game }
}

/// "you lose the game", "you win the game" (CR 104.3, 104.2).
fn lose_or_win_game(l: &str, _b: &mut Builder) -> Option<Effect> {
    match end(l) {
        "you lose the game" => Some(Effect::LoseGame {
            who: PlayerRef::You,
        }),
        "you win the game" => Some(Effect::WinGame {
            who: PlayerRef::You,
        }),
        _ => None,
    }
}

/// A line of an instant or sorcery that creates a delayed triggered ability when the spell
/// resolves: "At the beginning of your next upkeep, pay {2}{U}{U}. If you don't, you lose
/// the game." (the Pacts). The delayed ability chooses its own targets when it triggers
/// (CR 603.7, 603.3d).
fn spell_delayed_line(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_spell() || block.contains('\n') {
        return None;
    }
    let lower = block.trim().to_lowercase();
    let (trigger, inner) = split_delay(end(&lower))?;
    if !lower.starts_with("at ") || has_object_pronoun(inner) {
        return None;
    }
    // "pay {cost}. if you don't, [effect]": an "unless"-style payment (CR 118.12).
    let body = if let Some(r) = inner.strip_prefix("pay ") {
        let (cost_s, rest) = r.split_once(". if you don't, ")?;
        let cost = crate::oracle::keywords::parse_keyword_cost(cost_s)?;
        let otherwise = crate::oracle::effects::parse_body(rest, ctx)?;
        if otherwise.modal.is_some() {
            return None;
        }
        Body {
            targets: otherwise.targets,
            effect: Effect::PayOptional {
                who: PlayerRef::You,
                cost,
                then: Box::new(Effect::Noop),
                otherwise: Box::new(otherwise.effect),
            },
            modal: None,
        }
    } else {
        crate::oracle::effects::parse_body(inner, ctx)?
    };
    // Nothing in the delayed ability may depend on the spell's own targets or objects.
    let json = serde_json::to_string(&body.effect).ok()?;
    if ["\"This\"", "Trigger", "EventAmount"]
        .iter()
        .any(|p| json.contains(p))
    {
        return None;
    }
    let effect = Effect::DelayedTrigger {
        trigger,
        body: Box::new(body),
        once: true,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Spell(SpellAbility {
            body: Body::effect(effect),
        }),
        block,
    )])
}

/// First variable used to capture values for a delayed trigger.
const CAPTURE_BASE: Var = 200;

/// The delay phrases: (text, trigger).
fn delays() -> Vec<(&'static str, TriggerCond)> {
    let at = |step, whose| TriggerCond::BeginningOf { step, whose };
    vec![
        (
            "at the beginning of the next end step",
            at(TriggerStep::End, PlayerRel::Any),
        ),
        (
            "at the beginning of your next end step",
            at(TriggerStep::End, PlayerRel::You),
        ),
        (
            "at the beginning of the next turn's upkeep",
            at(TriggerStep::Upkeep, PlayerRel::Any),
        ),
        (
            "at the beginning of the next upkeep",
            at(TriggerStep::Upkeep, PlayerRel::Any),
        ),
        (
            "at the beginning of your next upkeep",
            at(TriggerStep::Upkeep, PlayerRel::You),
        ),
        (
            "at end of combat",
            at(TriggerStep::EndOfCombat, PlayerRel::Any),
        ),
        (
            "at the end of combat",
            at(TriggerStep::EndOfCombat, PlayerRel::Any),
        ),
    ]
}

/// Splits "[effect] [delay]" or "[delay], [effect]".
pub(crate) fn split_delay(l: &str) -> Option<(TriggerCond, &str)> {
    for (p, t) in delays() {
        if let Some(inner) = l.strip_prefix(p).and_then(|r| r.strip_prefix(", ")) {
            return Some((t, inner));
        }
        if let Some(inner) = l.strip_suffix(p).and_then(|r| r.strip_suffix(' ')) {
            return Some((t, inner));
        }
    }
    None
}

/// Whether the text has a pronoun whose referent could be something created or moved
/// by an earlier instruction of the same ability (which we can't track reliably here).
fn has_object_pronoun(s: &str) -> bool {
    s.split(|c: char| !c.is_alphanumeric() && c != '\'')
        .any(|w| {
            matches!(
                w,
                "it" | "them" | "that" | "those" | "they" | "these" | "this" | "it's"
            )
        })
}

inventory::submit! {
    EffectPattern { name: "until end of turn, whenever … / whenever … this turn", priority: 100, parse: this_turn_trigger }
}

/// "Until end of turn, whenever a player taps an Island for mana, that player adds an
/// additional {U}", "whenever a creature attacks this turn, put a +1/+1 counter on it": a
/// delayed triggered ability that lasts until end of turn (CR 603.7b). Its effect refers
/// to its own trigger event ("it", "that player") and chooses its own targets.
fn this_turn_trigger(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (cond_s, eff) = if let Some(r) = l.strip_prefix("until end of turn, whenever ") {
        let (c, e) = split_at_comma(r)?;
        (c.to_string(), e)
    } else {
        let r = l.strip_prefix("whenever ")?;
        let (c, e) = split_at_comma(r)?;
        (c.strip_suffix(" this turn")?.to_string(), e)
    };
    let (trigger, it, it_player) =
        crate::oracle::triggers::parse_trigger_condition(&format!("whenever {cond_s}"))?;
    if matches!(it, Sel::None) && has_object_pronoun(eff) {
        return None;
    }
    let body = crate::oracle::effects::parse_trigger_body(eff, b.ctx, it, it_player)?;
    Some(Effect::DelayedTrigger {
        trigger: TriggerCond::ThisTurn(Box::new(trigger)),
        body: Box::new(body),
        once: false,
    })
}

/// Splits at the first comma outside quotes.
fn split_at_comma(s: &str) -> Option<(&str, &str)> {
    let mut in_quote = false;
    for (i, ch) in s.char_indices() {
        match ch {
            '"' => in_quote = !in_quote,
            ',' if !in_quote => return Some((&s[..i], s[i + 1..].trim_start())),
            _ => {}
        }
    }
    None
}

fn delayed_trigger(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (trigger, inner) = split_delay(l)?;
    if inner.is_empty() || has_object_pronoun(inner) {
        return None;
    }
    let effect = parse_sentence(inner, b)?;
    let (stores, effect) = capture(&effect)?;
    let mut seq = stores;
    seq.push(Effect::DelayedTrigger {
        trigger,
        body: Box::new(Body::effect(effect)),
        once: true,
    });
    Some(Effect::seq(seq))
}

/// Rewrites references that depend on the creating ability's context (targets, trigger
/// event, source) into variables, returning the effects that fill those variables when
/// the delayed trigger is created.
pub fn capture(e: &Effect) -> Option<(Vec<Effect>, Effect)> {
    use serde_json::Value as J;
    let mut stores: Vec<Effect> = Vec::new();
    let mut used: Vec<Var> = Vec::new();
    fn walk(v: J, used: &mut Vec<Var>, stores: &mut Vec<Effect>) -> J {
        let mut store = |var: Var, eff: Effect, used: &mut Vec<Var>| {
            if !used.contains(&var) {
                used.push(var);
                stores.push(eff);
            }
            let mut m = serde_json::Map::new();
            m.insert("Var".into(), J::Number(var.into()));
            J::Object(m)
        };
        match v {
            J::String(s) => {
                let sel = |sel: Sel| Effect::Store { var: 0, sel };
                let (var, eff) = match s.as_str() {
                    "This" => (CAPTURE_BASE, sel(Sel::This)),
                    "TriggerObject" => (CAPTURE_BASE + 1, sel(Sel::TriggerObject)),
                    "TriggerLki" => (CAPTURE_BASE + 2, sel(Sel::TriggerLki)),
                    "TriggerOtherObject" => (CAPTURE_BASE + 3, sel(Sel::TriggerOtherObject)),
                    "TriggerSpell" => (CAPTURE_BASE + 4, sel(Sel::TriggerSpell)),
                    "TriggerPlayer" => (CAPTURE_BASE + 5, sel(Sel::TriggerPlayer)),
                    "EventAmount" => (
                        CAPTURE_BASE + 6,
                        Effect::StoreValue {
                            var: 0,
                            value: Value::EventAmount,
                        },
                    ),
                    _ => return J::String(s),
                };
                let eff = match eff {
                    Effect::Store { sel, .. } => Effect::Store { var, sel },
                    Effect::StoreValue { value, .. } => Effect::StoreValue { var, value },
                    other => other,
                };
                store(var, eff, used)
            }
            J::Object(m) => {
                if m.len() == 1 {
                    if let Some(J::Number(n)) = m.get("Target") {
                        let slot = n.as_u64().unwrap_or(0) as u8;
                        let var = CAPTURE_BASE + 10 + slot as Var;
                        return store(
                            var,
                            Effect::Store {
                                var,
                                sel: Sel::Target(slot),
                            },
                            used,
                        );
                    }
                }
                J::Object(
                    m.into_iter()
                        .map(|(k, v)| (k, walk(v, used, stores)))
                        .collect(),
                )
            }
            J::Array(a) => J::Array(a.into_iter().map(|x| walk(x, used, stores)).collect()),
            other => other,
        }
    }
    let json = serde_json::to_value(e).ok()?;
    let rewritten = walk(json, &mut used, &mut stores);
    // A reference that isn't a selection (e.g. a player relation in a filter) can't be
    // captured: deserialization fails and the text stays unsupported.
    let effect: Effect = serde_json::from_value(rewritten).ok()?;
    Some((stores, effect))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captures_targets_and_event() {
        let e = Effect::Seq(vec![
            Effect::Draw {
                who: PlayerRef::Target(0),
                n: Value::EventAmount,
            },
            Effect::SacrificeObjects { what: Sel::This },
        ]);
        let (stores, out) = capture(&e).unwrap();
        assert_eq!(stores.len(), 3);
        let s = format!("{out:?}");
        assert!(s.contains("Var(210)") && s.contains("Var(206)") && s.contains("Var(200)"));
        // Player relations in filters can't be captured.
        let e = Effect::Destroy {
            what: Sel::All(Filter::ControlledBy(PlayerRel::TriggerPlayer)),
            no_regen: false,
        };
        assert!(capture(&e).is_none());
    }
}
