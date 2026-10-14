//! Keyword statics II (CR 702.1c, 613.1f):
//!
//! - "~ has flying as long as a card exiled with it has flying. The same is true for first
//!   strike, ..." (Urborg Scavengers), "As long as a card exiled with ~ has flying, equipped
//!   creature has flying. The same is true for ..." (Eater of Virtue, Death-Mask
//!   Duplicant), "As long as an exiled creature card with a blood counter on it has flying,
//!   ..." (Rayami), "... an exiled card used to craft it has flying" (Altar of the
//!   Wretched): each listed keyword, in every variant such a card has.
//! - "Each creature you control has vigilance if it's white, hexproof if it's blue, ..."
//!   (Scion of Draco): one keyword per quality of the affected object.
//! - "For each basic land type among lands you control, ~ has landwalk of that type."
//!   (Magnigoth Treefolk): one landwalk per basic land type you control.

use super::AbilityPattern;
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::types::*;

/// The keyword kinds of "flying" plus "first strike, double strike, ..., and trample".
fn kind_list(first: &str, list: &str) -> Option<Vec<KeywordKind>> {
    let mut kinds = vec![KeywordKind::from_name(first)?];
    for item in list
        .split(", and ")
        .flat_map(|p| p.split(", "))
        .flat_map(|p| p.split(" and "))
        .map(str::trim)
        .filter(|p| !p.is_empty())
    {
        kinds.push(KeywordKind::from_name(item)?);
    }
    Some(kinds)
}

/// The cards a keyword is copied from: "a card exiled with ~" / "a card exiled with it"
/// (cards exiled by the source's linked abilities, CR 607.2a), "an exiled creature card
/// with a blood counter on it", "an exiled card used to craft it" (CR 702.167c).
fn exiled_holders(s: &str) -> Option<Filter> {
    let s = s
        .strip_prefix("a ")
        .or_else(|| s.strip_prefix("an "))
        .unwrap_or(s);
    let linked = |f: Filter| {
        Filter::and(vec![
            f,
            Filter::InZone(ZoneKind::Exile),
            Filter::In(Box::new(Sel::Linked)),
        ])
    };
    for suffix in [" exiled with ~", " exiled with it"] {
        if let Some(r) = s.strip_suffix(suffix) {
            let (f, _, tail) = parse_object_phrase(r)?;
            return end(tail).is_empty().then(|| linked(f));
        }
    }
    let r = s.strip_prefix("exiled ")?;
    if let Some(r) = r.strip_suffix(" used to craft it") {
        let (f, _, tail) = parse_object_phrase(r)?;
        return end(tail)
            .is_empty()
            .then(|| Filter::and(vec![f, crate::kw::craft::used_to_craft_filter()]));
    }
    let (f, _, tail) = parse_object_phrase(r)?;
    end(tail)
        .is_empty()
        .then(|| Filter::and(vec![f, Filter::InZone(ZoneKind::Exile)]))
}

/// "[subject] has [kw] as long as [cards] has [kw]" / "As long as [cards] has [kw],
/// [subject] has [kw]", followed by ". The same is true for [keywords]".
fn same_is_true_for_exiled(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell() || block.contains('\n') || block.contains(':') {
        return None;
    }
    let lower = block.to_lowercase();
    let l = end(&lower);
    let (first, list) = l.split_once(". the same is true for ")?;
    let (subject, kw, holder) = match first.strip_prefix("as long as ") {
        Some(r) => {
            let (cond, main) = r.split_once(", ")?;
            let (subject, kw) = main.split_once(" has ")?;
            let holder = cond.strip_suffix(&format!(" has {kw}"))?;
            (subject, kw, holder)
        }
        None => {
            let (main, cond) = first.split_once(" as long as ")?;
            let (subject, kw) = main.split_once(" has ")?;
            let holder = cond.strip_suffix(&format!(" has {kw}"))?;
            (subject, kw, holder)
        }
    };
    let affected = super::k702_001_010::static_subject(subject)?;
    let kinds = kind_list(kw, list)?;
    let from = exiled_holders(holder)?;
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
            affected,
            mods: vec![Modification::AddKeywordsOf { kinds, from }],
        })),
        block,
    )])
}

inventory::submit! { AbilityPattern { name: "keyword statics: [subject] has [kw] as long as [an exiled card] has [kw]. the same is true for ...", priority: 49, parse: same_is_true_for_exiled } }

/// A keyword without parameters ("vigilance", "first strike").
fn plain_keyword(s: &str) -> Option<Keyword> {
    let tl = TypeLine::default();
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
    let abilities = crate::oracle::keywords::parse_keyword_line(s, &ctx)?;
    match abilities.as_slice() {
        [a] => match &a.kind {
            AbilityKind::Keyword(k) => Some(k.clone()),
            _ => None,
        },
        _ => None,
    }
}

/// "Each creature you control has vigilance if it's white, hexproof if it's blue, ...,
/// and trample if it's green." (Scion of Draco): for each color, the objects of that
/// color have that keyword.
fn keyword_per_color(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell() {
        return None;
    }
    let l = end(l);
    let (subject, list) = l.split_once(" has ").or_else(|| l.split_once(" have "))?;
    let subject = subject.strip_prefix("each ").unwrap_or(subject);
    let (f, _, tail) = parse_object_phrase(subject)?;
    if !end(tail).is_empty() {
        return None;
    }
    let mut out = Vec::new();
    for item in list
        .split(", and ")
        .flat_map(|p| p.split(", "))
        .map(str::trim)
    {
        let (kw, color) = item
            .split_once(" if it's ")
            .or_else(|| item.split_once(" if they're "))?;
        let color = Color::from_word(color)?;
        let k = plain_keyword(kw)?;
        out.push(AbilityDef::new(
            AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
                affected: Filter::and(vec![f.clone(), Filter::Color(color)]),
                mods: vec![Modification::AddKeyword(k)],
            })),
            text,
        ));
    }
    (out.len() >= 2).then_some(out)
}

inventory::submit! { super::StaticPattern { name: "keyword statics: [objects] have [kw] if it's [color], [kw] if it's [color], ...", priority: 1090, parse: keyword_per_color } }

/// "For each basic land type among lands you control, ~ has landwalk of that type."
/// (Magnigoth Treefolk): plainswalk as long as you control a Plains, and so on.
fn landwalk_per_basic_type(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell() {
        return None;
    }
    let rest = end(l).strip_prefix("for each basic land type among lands you control, ")?;
    let subject = rest.strip_suffix(" has landwalk of that type")?;
    let affected = super::k702_001_010::static_subject(subject)?;
    let mut out = Vec::new();
    for t in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        let walk = format!("{}walk", t.to_lowercase());
        let k = plain_keyword(&walk)?;
        let mut st = StaticAbility::new(StaticEffect::Continuous {
            affected: affected.clone(),
            mods: vec![Modification::AddKeyword(k)],
        });
        st.condition = Some(Condition::Exists(Filter::and(vec![
            Filter::Type(CardType::Land),
            Filter::Subtype(Subtype::from(t)),
            Filter::ControlledBy(PlayerRel::You),
            Filter::InZone(ZoneKind::Battlefield),
        ])));
        out.push(AbilityDef::new(AbilityKind::Static(st), text));
    }
    Some(out)
}

inventory::submit! { super::StaticPattern { name: "keyword statics: for each basic land type among lands you control, ~ has landwalk of that type", priority: 1090, parse: landwalk_per_basic_type } }
