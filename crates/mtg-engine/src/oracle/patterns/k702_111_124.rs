//! Oracle text that goes with the keywords of CR 702.111–702.124:
//!
//! * renown (CR 702.112b): "~ is renowned", "as long as ~ is renowned, it has ...",
//!   "when ~ becomes renowned", "whenever a creature you control becomes renowned";
//! * surge and emerge (CR 702.117a, 702.119a): "if its surge cost was paid", "if ~'s
//!   emerge cost was paid";
//! * "Emerge from [quality] [cost]" (CR 702.119b).

use super::{AbilityPattern, ConditionPattern, FollowupPattern, StaticPattern, TriggerPattern};
use crate::oracle::effects::Builder;
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::kw::renown::RENOWNED;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;
use smol_str::SmolStr;

// ---------------------------------------------------------------------------
// Renown (CR 702.112)
// ---------------------------------------------------------------------------

fn renowned() -> Filter {
    Filter::Custom(SmolStr::new(RENOWNED))
}

/// "~ is renowned", "it's renowned", "~ isn't renowned": the renowned designation of the
/// permanent itself (CR 702.112b).
fn renowned_condition(c: &str) -> Option<Condition> {
    let yes = match end(c) {
        "~ is renowned" | "it's renowned" | "it is renowned" => true,
        "~ isn't renowned" | "~ is not renowned" | "it isn't renowned" | "it's not renowned" => {
            false
        }
        _ => return None,
    };
    let f = if yes {
        renowned()
    } else {
        Filter::not(renowned())
    };
    Some(Condition::SelMatches(Sel::This, f))
}

inventory::submit! { ConditionPattern { name: "k702.112 ~ is renowned", priority: 100, parse: renowned_condition } }

/// "As long as ~ is renowned, it has [abilities]": "it" is the permanent itself.
fn renowned_static(l: &str, _text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("as long as ~ is renowned, ")?;
    let rest = r.strip_prefix("it ")?;
    crate::oracle::statics::parse_static(&format!("as long as ~ is renowned, ~ {rest}"), ctx)
}

inventory::submit! { StaticPattern { name: "k702.112 as long as ~ is renowned", priority: 100, parse: renowned_static } }

/// "If it's renowned, [effect]" after an instruction about another object ("Target
/// creature gets +1/+1 until end of turn. ... If it's renowned, untap it."): "it" is that
/// object, not the source.
fn if_its_renowned_followup(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(rest) = end(l).strip_prefix("if it's renowned, ") else {
        return false;
    };
    if matches!(b.it, Sel::This) {
        return false;
    }
    let subject = b.it.clone();
    let Some(then) = crate::oracle::effects::parse_clause(rest, b) else {
        return false;
    };
    *prev = Effect::seq(vec![
        prev.clone(),
        Effect::If {
            cond: Condition::SelMatches(subject, renowned()),
            then: Box::new(then),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "k702.112 if it's renowned", priority: 100, apply: if_its_renowned_followup } }

/// "~ becomes renowned", "a creature you control becomes renowned": a permanent became
/// renowned (the [`RENOWNED`] event).
fn becomes_renowned(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let subj = end(r).strip_suffix(" becomes renowned")?;
    let f = if subj == "~" {
        Filter::Source
    } else {
        let s = subj
            .strip_prefix("a ")
            .or_else(|| subj.strip_prefix("an "))
            .or_else(|| subj.strip_prefix("another "))?;
        let (f, plural, tail) = parse_object_phrase(s)?;
        if plural || !end(tail).is_empty() {
            return None;
        }
        if subj.starts_with("another ") {
            Filter::and(vec![f, Filter::Other])
        } else {
            f
        }
    };
    Some((
        TriggerCond::Where {
            trigger: Box::new(TriggerCond::PlayerAction {
                name: SmolStr::new(RENOWNED),
                who: PlayerRel::Any,
            }),
            cond: Condition::SelMatches(Sel::TriggerObject, f),
        },
        Sel::TriggerObject,
        PlayerRef::TriggerPlayer,
    ))
}

inventory::submit! { TriggerPattern { name: "k702.112 becomes renowned", priority: 100, parse: becomes_renowned } }

// ---------------------------------------------------------------------------
// Surge, emerge (CR 702.117a, 702.119a)
// ---------------------------------------------------------------------------

/// "Emerge from [quality] [cost]" (CR 702.119b): emerge whose sacrificed permanent is a
/// [quality] permanent (the keyword's filter) rather than a creature.
fn emerge_from(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim().trim_end_matches('.');
    let lower = t.to_lowercase();
    let rest = lower.strip_prefix("emerge from ")?;
    let at = rest.find('{')?;
    let quality = crate::oracle::keywords::quality_phrase(rest[..at].trim())?;
    let cost_at = "emerge from ".len() + at;
    let cost = crate::oracle::keywords::parse_keyword_cost(&t[cost_at..])?;
    let kw = Keyword {
        cost: Some(cost),
        filter: Some(quality),
        text: Some(SmolStr::new(t)),
        ..Keyword::new(KeywordKind::Emerge)
    };
    Some(crate::oracle::keywords::compile_keyword(kw, t))
}

inventory::submit! { AbilityPattern { name: "k702.119b emerge from [quality]", priority: 100, parse: emerge_from } }

/// "its surge cost was paid", "~'s surge cost was paid", "this spell's emerge cost was
/// paid", "you cast it for its surge cost".
fn alt_cost_paid(c: &str) -> Option<Condition> {
    let c = end(c);
    let r = c
        .strip_prefix("its ")
        .or_else(|| c.strip_prefix("~'s "))
        .or_else(|| c.strip_prefix("this spell's "));
    let name = match r {
        Some("surge cost was paid") => crate::kw::surge::SURGE,
        Some("emerge cost was paid") => crate::kw::emerge::EMERGE,
        _ => match c {
            "you cast it for its surge cost" | "you cast ~ for its surge cost" => {
                crate::kw::surge::SURGE
            }
            _ => return None,
        },
    };
    Some(Condition::CostPaid(SmolStr::new(name)))
}

inventory::submit! { ConditionPattern { name: "k702.117/119 surge or emerge cost was paid", priority: 100, parse: alt_cost_paid } }
