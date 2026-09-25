//! Oracle patterns for the keyword abilities of CR 702.1–702.10 and the effects that
//! grant, modify, or refer to them:
//!
//! * "[This] has double strike as long as you have no cards in hand" — a static ability
//!   with a trailing condition, the same as "As long as [condition], [this] has ...".
//! * "Instant and sorcery spells you control have deathtouch" (keywords on spells,
//!   CR 702.2d).
//! * "Equip costs you pay cost {1} less" / "Equip abilities you activate cost {1} less to
//!   activate" (CR 702.1a: a "[keyword] cost" is the keyword's own cost).
//! * Self-state conditions: "it's attacking", "~ is equipped", ...

use super::{AbilityPattern, ConditionPattern, StaticPattern};
use crate::ability::*;
use crate::keywords::KeywordKind;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::types::CardType;

fn with_text(v: Vec<Ability>, text: &str) -> Vec<Ability> {
    v.into_iter()
        .map(|a| AbilityDef::with_link(a.kind.clone(), text, a.link))
        .collect()
}

/// "[subject] has [keywords] as long as [condition]" → "As long as [condition],
/// [subject] has [keywords]". (An ability pattern, since the static parser commits to
/// "~ has ..." lines before trying pluggable static patterns.)
fn trailing_as_long_as(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell() || block.contains('\n') || block.contains(':') || block.contains('"') {
        return None;
    }
    let lower = block.to_lowercase();
    let l = end(&lower);
    if l.starts_with("as long as ")
        || l.starts_with("when")
        || l.starts_with("at ")
        || l.contains(". ")
    {
        return None;
    }
    let (head, cond) = l.split_once(" as long as ")?;
    if cond.contains(" as long as ")
        || !(head.contains(" has ")
            || head.contains(" have ")
            || head.contains(" gets ")
            || head.contains(" get ")
            || head.contains(" can "))
    {
        return None;
    }
    let reordered = format!("as long as {cond}, {head}");
    let v = crate::oracle::statics::parse_static(&reordered, ctx)?;
    // Only accept it if the condition was attached to every ability.
    if v.is_empty()
        || !v
            .iter()
            .all(|a| matches!(&a.kind, AbilityKind::Static(s) if s.condition.is_some()))
    {
        return None;
    }
    Some(with_text(v, block))
}

inventory::submit! {
    AbilityPattern { name: "k702: trailing as long as", priority: 50, parse: trailing_as_long_as }
}

/// Parses a list of keywords ("deathtouch", "flying and first strike") into keyword
/// modifications.
pub fn keyword_mods(s: &str) -> Option<Vec<Modification>> {
    let s = end(s);
    let tl = crate::types::TypeLine::default();
    let ctx = CompileContext {
        card_name: "",
        full_name: "",
        type_line: &tl,
        layout: crate::card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    let mut out = Vec::new();
    for p in s
        .split(", and ")
        .flat_map(|p| p.split(" and "))
        .flat_map(|p| p.split(", "))
        .map(str::trim)
        .filter(|p| !p.is_empty())
    {
        for a in crate::oracle::keywords::parse_keyword_line(p, &ctx)? {
            if let AbilityKind::Keyword(k) = &a.kind {
                out.push(Modification::AddKeyword(k.clone()));
            }
        }
    }
    (!out.is_empty()).then_some(out)
}

/// "Instant and sorcery spells you control have deathtouch." — a continuous effect on
/// spells on the stack (CR 611.3, 702.2d).
fn spells_have_keywords(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (subject, kws) = l.split_once(" spells you control have ")?;
    let mut types = Vec::new();
    for w in subject
        .split(" and ")
        .flat_map(|p| p.split(" or "))
        .flat_map(|p| p.split(", "))
        .map(str::trim)
        .filter(|w| !w.is_empty() && *w != "and/or")
    {
        types.push(Filter::Type(CardType::from_word(w)?));
    }
    if types.is_empty() {
        return None;
    }
    let mods = keyword_mods(kws)?;
    let affected = Filter::and(vec![
        Filter::Or(types),
        Filter::Spell,
        Filter::ControlledBy(PlayerRel::You),
    ]);
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
            affected,
            mods,
        })),
        text,
    )])
}

inventory::submit! {
    StaticPattern { name: "k702: spells you control have keywords", priority: 50, parse: spells_have_keywords }
}

/// "Equip costs you pay cost {1} less." / "Equip abilities you activate cost {1} less to
/// activate." (CR 702.1a).
fn keyword_cost_modifier(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (kw, rest) = if let Some(r) = l.strip_prefix("equip costs you pay cost ") {
        (KeywordKind::Equip, r)
    } else if let Some(r) = l.strip_prefix("equip abilities you activate cost ") {
        (KeywordKind::Equip, r.strip_suffix(" to activate").unwrap_or(r))
    } else {
        return None;
    };
    let (amount, tail) = rest.split_once('}')?;
    let n: i32 = amount.strip_prefix('{')?.parse().ok()?;
    let change = match end(tail).trim() {
        "less" => CostChange::ReduceGeneric(Value::c(n)),
        "more" => CostChange::IncreaseGeneric(Value::c(n)),
        _ => return None,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::CostModifier(
            CostModifier {
                applies_to: CostTarget::Keyword(kw),
                who: PlayerRel::You,
                change,
            },
        ))),
        text,
    )])
}

inventory::submit! {
    StaticPattern { name: "k702: keyword cost modifiers", priority: 50, parse: keyword_cost_modifier }
}

/// Conditions about the object itself: "it's attacking", "~ is equipped", "it's
/// enchanted or equipped".
fn self_state_condition(c: &str) -> Option<Condition> {
    let c = end(c);
    let state = c
        .strip_prefix("it's ")
        .or_else(|| c.strip_prefix("it is "))
        .or_else(|| c.strip_prefix("~ is "))?;
    let f = match state {
        "attacking" => Filter::Attacking,
        "blocking" => Filter::Blocking,
        "attacking or blocking" => Filter::Or(vec![Filter::Attacking, Filter::Blocking]),
        "enchanted" => Filter::Enchanted,
        "equipped" => Filter::Equipped,
        "enchanted or equipped" | "equipped or enchanted" => {
            Filter::Or(vec![Filter::Enchanted, Filter::Equipped])
        }
        "modified" => Filter::Modified,
        "untapped" => Filter::Untapped,
        "tapped" => Filter::Tapped,
        _ => return None,
    };
    Some(Condition::SelMatches(Sel::This, f))
}

inventory::submit! {
    ConditionPattern { name: "k702: self state", priority: 150, parse: self_state_condition }
}
