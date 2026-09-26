//! Oracle text that goes with the keywords of CR 702.125–702.139:
//!
//! * mentor (CR 702.134c): "whenever ~ mentors a creature", "whenever equipped creature
//!   mentors a creature";
//! * spectacle (CR 702.137a): "if its spectacle cost was paid";
//! * escape (CR 702.138b–d): "~ escaped", "sacrifice it unless it escaped", "~ escapes
//!   with [counters]" (optionally followed by "When it enters this way, ..."), "~ escapes
//!   with [ability]", "~ enters with N counters on it. It escapes with M counters on it
//!   instead.", "Each [quality] card in your graveyard has escape. The escape cost is
//!   equal to the card's mana cost plus [cost].";
//! * "If [condition], instead [effect]." after a sentence, the word order used with
//!   spectacle and ascend ("If ~'s spectacle cost was paid, instead discard your hand,
//!   then draw three cards.", "If you have the city's blessing, instead each opponent
//!   sacrifices ...").

use super::{
    AbilityPattern, ConditionPattern, EffectPattern, FollowupPattern, StaticPattern, TriggerPattern,
};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::oracle::effects::{parse_effect_text, Builder};
use crate::oracle::phrases::end;
use crate::oracle::phrases::parse_object_phrase;
use crate::oracle::statics::parse_condition;
use crate::oracle::CompileContext;
use smol_str::SmolStr;

// ---------------------------------------------------------------------------
// Mentor (CR 702.134)
// ---------------------------------------------------------------------------

/// "~ mentors a creature" / "equipped creature mentors a creature": a mentor ability of
/// that creature resolved targeting a creature, which is "that creature" (CR 702.134c).
fn mentors(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let name = match end(r) {
        "~ mentors a creature" | "this creature mentors a creature" => crate::kw::mentor::MENTORS,
        "equipped creature mentors a creature" => crate::kw::mentor::EQUIPPED_MENTORS,
        _ => return None,
    };
    Some((
        TriggerCond::Custom(SmolStr::new(name)),
        Sel::TriggerObject,
        PlayerRef::You,
    ))
}

inventory::submit! { TriggerPattern { name: "k702.134 mentors a creature", priority: 100, parse: mentors } }

// ---------------------------------------------------------------------------
// Spectacle (CR 702.137)
// ---------------------------------------------------------------------------

/// "its spectacle cost was paid", "~'s spectacle cost was paid", "this spell's spectacle
/// cost was paid".
fn spectacle_cost_paid(c: &str) -> Option<Condition> {
    let c = end(c);
    let r = c
        .strip_prefix("its ")
        .or_else(|| c.strip_prefix("~'s "))
        .or_else(|| c.strip_prefix("this spell's "))?;
    (r == "spectacle cost was paid")
        .then(|| Condition::CostPaid(SmolStr::new(crate::kw::spectacle::SPECTACLE)))
}

inventory::submit! { ConditionPattern { name: "k702.137 spectacle cost was paid", priority: 100, parse: spectacle_cost_paid } }

// ---------------------------------------------------------------------------
// "If [condition], instead [effect]."
// ---------------------------------------------------------------------------

/// "If [condition], instead [effect].": the previous sentence's effect is replaced by
/// `effect` when the condition holds as the spell or ability resolves (CR 608.2c). The
/// replacement can't introduce targets of its own.
fn if_instead(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("if ") else {
        return false;
    };
    let Some((c, x)) = r.split_once(", instead ") else {
        return false;
    };
    if matches!(prev, Effect::Noop) {
        return false;
    }
    let Some(cond) = parse_condition(c, b.ctx) else {
        return false;
    };
    let targets = b.targets.len();
    let Some(e) = parse_effect_text(x, b) else {
        b.targets.truncate(targets);
        return false;
    };
    if b.targets.len() != targets {
        b.targets.truncate(targets);
        return false;
    }
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::If {
        cond,
        then: Box::new(e),
        otherwise: Box::new(old),
    };
    true
}

inventory::submit! { FollowupPattern { name: "k702.137 if [condition], instead [effect]", priority: 100, apply: if_instead } }

// ---------------------------------------------------------------------------
// Escape (CR 702.138)
// ---------------------------------------------------------------------------

/// "~ escaped", "it escaped", "this creature escaped" (CR 702.138b).
fn escaped_condition(c: &str) -> Option<Condition> {
    matches!(
        end(c),
        "~ escaped" | "it escaped" | "this creature escaped" | "this permanent escaped"
    )
    .then(crate::kw::escape::escaped)
}

inventory::submit! { ConditionPattern { name: "k702.138 ~ escaped", priority: 100, parse: escaped_condition } }

/// "sacrifice ~ unless it escaped", "sacrifice it unless it escaped".
fn sacrifice_unless_escaped(l: &str, b: &mut Builder) -> Option<Effect> {
    let what = match end(l) {
        "sacrifice ~ unless it escaped" | "sacrifice ~ unless ~ escaped" => Sel::This,
        "sacrifice it unless it escaped" => b.it.clone(),
        _ => return None,
    };
    Some(Effect::If {
        cond: crate::kw::escape::escaped(),
        then: Box::new(Effect::Noop),
        otherwise: Box::new(Effect::SacrificeObjects { what }),
    })
}

inventory::submit! { EffectPattern { name: "k702.138 sacrifice it unless it escaped", priority: 100, parse: sacrifice_unless_escaped } }

/// "~ escapes with [counters or abilities]" (CR 702.138c–d), optionally followed by its
/// linked "When it enters this way, [effect]." (CR 603.11), which triggers when the
/// permanent enters after the replacement effect was applied — that is, when it escaped;
/// and "~ enters with N counters on it. It escapes with M counters on it instead."
fn escapes_with(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() || block.contains('\n') {
        return None;
    }
    let lower = block.to_lowercase();
    let l = end(&lower);
    let parse = |t: &str| -> Option<Vec<Ability>> {
        let v = crate::oracle::parse_ability(t, ctx)?;
        (!v.is_empty()
            && !v
                .iter()
                .any(|a| matches!(a.kind, AbilityKind::Unsupported(_))))
        .then_some(v)
    };
    let with_text = |mut v: Vec<Ability>| {
        for a in v.iter_mut() {
            std::sync::Arc::make_mut(a).text = block.to_string();
        }
        v
    };
    // "~ enters with six +1/+1 counters on it. It escapes with twelve +1/+1 counters on
    // it instead."
    if let Some((a, b)) = l.split_once(". it escapes with ") {
        let b = b.strip_suffix(" instead")?;
        let enters = a.strip_prefix("~ enters with ")?;
        let mut v = parse(&format!("~ enters with {enters} unless ~ escaped"))?;
        v.extend(parse(&format!("~ enters with {b} if ~ escaped"))?);
        return Some(with_text(v));
    }
    let r = l.strip_prefix("~ escapes with ")?;
    let (what, linked) = match r.split_once(". when it enters this way, ") {
        Some((w, e)) => (w, Some(e)),
        None => (r, None),
    };
    let mut v = if what.contains(" counter") && !what.contains('"') {
        // "If this permanent escaped, it enters with [those counters]" (CR 702.138c).
        parse(&format!("~ enters with {what} if ~ escaped"))?
    } else {
        // "If this permanent escaped, it has [ability]" (CR 702.138d). The original case
        // of a quoted ability is kept.
        let at = block.to_lowercase().find("escapes with ")? + "escapes with ".len();
        let orig = block.get(at..)?.trim_end_matches('.');
        crate::oracle::statics::parse_static(&format!("As long as ~ escaped, ~ has {orig}"), ctx)
            .filter(|v| !v.is_empty())?
    };
    if let Some(e) = linked {
        v.extend(parse(&format!("when ~ enters, if ~ escaped, {e}"))?);
    }
    Some(with_text(v))
}

inventory::submit! { AbilityPattern { name: "k702.138 ~ escapes with", priority: 100, parse: escapes_with } }

/// "Each [quality] card in your graveyard has escape. The escape cost is equal to the
/// card's mana cost plus exile three other cards from your graveyard." (Underworld
/// Breach): each of those cards has an escape ability whose cost is its mana cost plus
/// the rest (see [`crate::kw::escape::MANA_COST_PLUS`]).
fn graveyard_cards_have_escape(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("each ")?;
    let (subject, rest) = r.split_once(" card in your graveyard has escape. ")?;
    let plus = rest
        .strip_prefix("the escape cost is equal to the card's mana cost plus ")
        .or_else(|| rest.strip_prefix("the escape cost is equal to its mana cost plus "))?;
    let (cost, _) = crate::oracle::costs::parse_cost(plus)?;
    if cost.mana.is_some() || cost.parts.is_empty() {
        return None;
    }
    let phrase = format!("{subject} card");
    let (f, _, tail) = parse_object_phrase(&phrase)?;
    if !end(tail).is_empty() {
        return None;
    }
    let affected = Filter::and(vec![
        f,
        Filter::Card,
        Filter::InZone(ZoneKind::Graveyard),
        Filter::OwnedBy(PlayerRel::You),
    ]);
    let kw = Keyword::with_cost(KeywordKind::Escape, cost).text(crate::kw::escape::MANA_COST_PLUS);
    let s = StaticAbility::new(StaticEffect::Continuous {
        affected,
        mods: vec![Modification::AddKeyword(kw)],
    });
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "k702.138 each card in your graveyard has escape", priority: 100, parse: graveyard_cards_have_escape } }
