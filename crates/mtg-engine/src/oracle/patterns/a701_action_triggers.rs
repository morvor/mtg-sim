//! Oracle patterns for triggers, conditions and follow-up sentences about the keyword
//! actions of CR 701.29–701.71: "whenever a creature you control explores", "when ~
//! becomes monstrous", "whenever you clash", "If you win, ...", "as long as ~ is
//! monstrous", "∞ — [ability]" (while harnessed), and "support N" as an instruction.

use super::{AbilityPattern, ConditionPattern, EffectPattern, StaticPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use smol_str::SmolStr;

/// "~" or an object phrase ("a creature you control", "another Villain you control") as
/// the subject of a trigger. Returns the filter for the object.
fn subject_filter(s: &str) -> Option<Filter> {
    let s = s.trim();
    if s == "~" {
        return Some(Filter::Source);
    }
    let s = s
        .strip_prefix("a ")
        .or_else(|| s.strip_prefix("an "))
        .unwrap_or(s);
    let (f, plural, tail) = parse_object_phrase(s)?;
    (!plural && end(tail).is_empty()).then_some(f)
}

/// A keyword action event ([`crate::kwa::emit`]) about an object matching `f`.
fn object_action(name: &str, f: Filter, extra: Option<Condition>) -> TriggerCond {
    let mut conds = vec![Condition::SelMatches(Sel::TriggerObject, f)];
    conds.extend(extra);
    TriggerCond::Where {
        trigger: Box::new(TriggerCond::PlayerAction {
            name: SmolStr::new(name),
            who: PlayerRel::Any,
        }),
        cond: if conds.len() == 1 {
            conds.pop().unwrap()
        } else {
            Condition::And(conds)
        },
    }
}

/// "[object] explores[ a land card / a nonland card]", "[object] connives", "[object]
/// endures", "[object] becomes monstrous".
fn object_action_trigger(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    use crate::kwa::explore::{EXPLORED, REVEALED_LAND, REVEALED_NONLAND};
    let r = end(r);
    let amount = |n: i32| Some(Condition::Compare(Value::EventAmount, Cmp::Eq, Value::c(n)));
    let forms: [(&str, &str, Option<Condition>); 6] = [
        (" explores", EXPLORED, None),
        (" explores a land card", EXPLORED, amount(REVEALED_LAND)),
        (" explores a nonland card", EXPLORED, amount(REVEALED_NONLAND)),
        (" connives", crate::kwa::connive::CONNIVED, None),
        (" endures", crate::kwa::endure::ENDURED, None),
        (" becomes monstrous", crate::kwa::monstrosity::MONSTROUS, None),
    ];
    for (suffix, name, extra) in forms {
        if let Some(subj) = r.strip_suffix(suffix) {
            let f = subject_filter(subj)?;
            return Some((
                object_action(name, f, extra),
                Sel::TriggerObject,
                PlayerRef::TriggerPlayer,
            ));
        }
    }
    None
}

inventory::submit! { TriggerPattern { name: "a701 object keyword action triggers", priority: 60, parse: object_action_trigger } }

/// "[player] [action]s": "whenever you clash", "whenever you discover", "whenever an
/// opponent blights", ...
fn player_action_trigger(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = end(r);
    let (who, rest) = if let Some(x) = r.strip_prefix("you ") {
        (PlayerRel::You, x)
    } else if let Some(x) = r.strip_prefix("an opponent ") {
        (PlayerRel::Opponent, x)
    } else if let Some(x) = r.strip_prefix("a player ") {
        (PlayerRel::Any, x)
    } else {
        return None;
    };
    let name = match rest.trim_end_matches('s') {
        "clash" | "clashe" => crate::kwa::fateseal_clash::CLASHED,
        "discover" => crate::kwa::discover::DISCOVERED_EVENT,
        "blight" => crate::kwa::blight::BLIGHTED_EVENT,
        "amas" | "amass" => crate::kwa::amass::AMASSED_EVENT,
        "time travel" => crate::kwa::time_travel::TIME_TRAVELED,
        "learn" => crate::kwa::learn::LEARNED,
        _ => return None,
    };
    Some((
        TriggerCond::PlayerAction {
            name: SmolStr::new(name),
            who,
        },
        Sel::None,
        PlayerRef::TriggerPlayer,
    ))
}

inventory::submit! { TriggerPattern { name: "a701 player keyword action triggers", priority: 60, parse: player_action_trigger } }

/// Replaces `Value::X` with `x` in any part of an ability.
fn substitute_x<T: serde::Serialize + serde::de::DeserializeOwned>(t: &T, x: &Value) -> Option<T> {
    fn subst(v: serde_json::Value, from: &serde_json::Value, to: &serde_json::Value) -> serde_json::Value {
        use serde_json::Value as J;
        if &v == from {
            return to.clone();
        }
        match v {
            J::Object(m) => J::Object(m.into_iter().map(|(k, x)| (k, subst(x, from, to))).collect()),
            J::Array(a) => J::Array(a.into_iter().map(|x| subst(x, from, to)).collect()),
            other => other,
        }
    }
    let json = serde_json::to_value(t).ok()?;
    let to = serde_json::to_value(x).ok()?;
    serde_json::from_value(subst(json, &serde_json::Value::String("X".into()), &to)).ok()
}

/// "When ~ becomes monstrous, [effect with X]": X is the value of X as it became
/// monstrous (CR 701.37c), the amount of the event.
fn monstrous_x_trigger(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    let eff = lower.strip_prefix("when ~ becomes monstrous, ")?;
    let words: Vec<&str> = eff
        .split(|c: char| !c.is_alphanumeric() && c != '/')
        .collect();
    if !words.iter().any(|w| *w == "x" || w.contains("x/") || w.contains("/x")) {
        return None;
    }
    let start = t.len() - eff.len();
    let body = crate::oracle::effects::parse_trigger_body(
        &t[start..],
        ctx,
        Sel::TriggerObject,
        PlayerRef::TriggerPlayer,
    )?;
    let body = substitute_x(&body, &Value::EventAmount)?;
    let tr = TriggeredAbility::new(
        object_action(crate::kwa::monstrosity::MONSTROUS, Filter::Source, None),
        body,
    );
    Some(vec![AbilityDef::new(AbilityKind::Triggered(tr), t)])
}

inventory::submit! { AbilityPattern { name: "a701 becomes monstrous with x", priority: 60, parse: monstrous_x_trigger } }

/// "~ is monstrous", "~ isn't monstrous", "~ is harnessed", "~ is suspected".
fn designation_condition(c: &str) -> Option<Condition> {
    let c = end(c);
    for (word, name) in [
        ("monstrous", crate::kwa::monstrosity::MONSTROUS),
        ("harnessed", crate::kwa::harness::HARNESSED),
        ("suspected", crate::kwa::suspect_detain::SUSPECTED),
    ] {
        for (subj, yes) in [
            (c.strip_suffix(&format!(" is {word}")), true),
            (c.strip_suffix(&format!(" isn't {word}")), false),
        ] {
            if let Some(s) = subj {
                if s != "~" && s != "it" {
                    return None;
                }
                let f = Filter::Custom(SmolStr::new(name));
                return Some(Condition::SelMatches(
                    Sel::This,
                    if yes { f } else { Filter::not(f) },
                ));
            }
        }
    }
    None
}

inventory::submit! { ConditionPattern { name: "a701 designations", priority: 60, parse: designation_condition } }

/// "As long as ~ is monstrous, it has [abilities]": "it" is the permanent itself.
fn designation_static(l: &str, _text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("as long as ")?;
    let (c, rest) = r.split_once(", ")?;
    designation_condition(c)?;
    let rest = rest.strip_prefix("it ")?;
    crate::oracle::statics::parse_static(&format!("as long as {c}, ~ {rest}"), ctx)
}

inventory::submit! { StaticPattern { name: "a701 designation statics", priority: 60, parse: designation_static } }

/// "If you win, [effect]" after "clash with an opponent" (CR 701.30d): the clash result is
/// the previous instruction's outcome. "If you won, [effect]" in an ability that triggers
/// "whenever you clash": the event's.
fn if_you_win(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (cond, rest) = if let Some(r) = l.strip_prefix("if you win, ") {
        (Condition::PrevHappened, r)
    } else if let Some(r) = l.strip_prefix("if you won, ") {
        (
            Condition::Compare(Value::EventAmount, Cmp::Eq, Value::c(1)),
            r,
        )
    } else {
        return None;
    };
    let then = crate::oracle::effects::parse_clause(rest, b)?;
    Some(Effect::If {
        cond,
        then: Box::new(then),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "a701 if you win the clash", priority: 60, parse: if_you_win } }

/// "support N" as an instruction (CR 701.41a): "put a +1/+1 counter on each of up to N
/// other target creatures" (a permanent's ability) or "up to N target creatures" (an
/// instant or sorcery's).
fn support_instruction(l: &str, b: &mut Builder) -> Option<Effect> {
    let (n, rest) = parse_number(end(l).strip_prefix("support ")?)?;
    if !end(rest).is_empty() {
        return None;
    }
    let spell = b.ctx.is_spell();
    let text = if spell {
        "up to N target creatures"
    } else {
        "up to N other target creatures"
    };
    let what = if spell {
        Filter::creature()
    } else {
        Filter::and(vec![Filter::creature(), Filter::Other])
    };
    let spec = TargetSpec {
        what: TargetKind::Object(what),
        min: 0,
        max: n,
        distinct_from: vec![],
        divide: None,
        chosen_by_opponent: false,
        text: String::new(),
        condition: None,
    };
    let slot = b.add_target(spec, text);
    Some(Effect::AddCounters {
        what: Sel::Target(slot),
        kind: SmolStr::new(crate::types::counters::PLUS1),
        n: Value::c(1),
    })
}

inventory::submit! { EffectPattern { name: "a701 support", priority: 60, parse: support_instruction } }

/// "∞ — [ability]" (CR 702.186b): "As long as this permanent is harnessed, it has
/// [ability]" (CR 701.64).
fn infinity(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let rest = t
        .strip_prefix("∞ — ")
        .or_else(|| t.strip_prefix("∞ - "))?;
    let granted = crate::oracle::parse_ability(rest, ctx)?;
    if granted
        .iter()
        .any(|a| matches!(a.kind, AbilityKind::Unsupported(_)))
    {
        return None;
    }
    let mut s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::Source,
        mods: granted.into_iter().map(Modification::AddAbility).collect(),
    });
    s.condition = Some(Condition::SelMatches(
        Sel::This,
        Filter::Custom(SmolStr::new(crate::kwa::harness::HARNESSED)),
    ));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), t)])
}

inventory::submit! { AbilityPattern { name: "a701 infinity abilities", priority: 60, parse: infinity } }
