//! Oracle patterns for CR 702.27–702.37 (buyback, shadow, cycling, echo, horsemanship,
//! fading, kicker, flashback, madness, fear, morph):
//!
//! * "Buyback costs cost {2} less." / "Cycling abilities you activate cost {2} less to
//!   activate." — modifications of a keyword's costs (CR 601.2f, 602.2b; typecycling costs
//!   are cycling costs, CR 702.29f);
//! * "if it was kicked with its {1}{B} kicker" — a condition linked to one specific kicker
//!   cost (CR 702.33f, 607.2i).

use super::{ConditionPattern, StaticPattern};
use crate::ability::*;
use crate::keywords::KeywordKind;
use crate::mana::ManaCost;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

/// "{2}" → 2.
fn generic_amount(s: &str) -> Option<i32> {
    s.trim().strip_prefix('{')?.strip_suffix('}')?.parse().ok()
}

/// "[Keyword] costs cost {N} less/more" and "[Keyword] abilities you activate cost {N}
/// less/more to activate".
fn keyword_cost_change(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (kind, who, rest) = if let Some(r) = l.strip_prefix("buyback costs cost ") {
        (KeywordKind::Buyback, PlayerRel::Any, r)
    } else if let Some(r) = l.strip_prefix("cycling abilities you activate cost ") {
        let r = r.strip_suffix(" to activate").unwrap_or(r);
        (KeywordKind::Cycling, PlayerRel::You, r)
    } else {
        return None;
    };
    let (amount, dir) = end(rest).rsplit_once(' ')?;
    let n = generic_amount(amount)?;
    let change = match dir {
        "less" => CostChange::ReduceGeneric(Value::c(n)),
        "more" => CostChange::IncreaseGeneric(Value::c(n)),
        _ => return None,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::CostModifier(
            CostModifier {
                applies_to: CostTarget::Keyword(kind),
                who,
                change,
            },
        ))),
        text,
    )])
}

/// The name recorded in `CastInfo::paid` for a paid kicker cost with this mana cost
/// (see `Game::cast_inner`, CR 607.2i).
pub fn kicker_cost_name(cost: &ManaCost) -> String {
    format!("kicker {cost}")
}

/// "it was kicked with its {1}{b} kicker" (CR 702.33f); "it was kicked twice" (a spell with
/// two kicker costs whose controller paid both, CR 702.33d).
fn kicked_with(c: &str) -> Option<Condition> {
    let c = end(c);
    let r = ["it ", "~ ", "this spell ", "this creature ", "this permanent "]
        .iter()
        .find_map(|p| c.strip_prefix(p))?;
    if r == "was kicked twice" {
        return Some(Condition::Compare(Value::TimesKicked, Cmp::Ge, Value::c(2)));
    }
    let sym = r
        .strip_prefix("was kicked with its ")?
        .strip_suffix(" kicker")?;
    let cost = ManaCost::parse(&sym.to_uppercase())?;
    Some(Condition::CostPaid(kicker_cost_name(&cost).into()))
}

/// "~ can block creatures with shadow as though it had shadow" / "... as though they
/// didn't have shadow" (see `kw/shadow.rs`, CR 702.28b).
fn blocks_shadow(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    use crate::kw::shadow::*;
    let name = match end(l) {
        "~ can block creatures with shadow as though it had shadow" => {
            BLOCKS_SHADOW_AS_THOUGH_IT_HAD_SHADOW
        }
        "~ can block creatures with shadow as though they didn't have shadow" => {
            BLOCKS_SHADOW_AS_THOUGH_THEY_DIDNT
        }
        _ => return None,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(name.into()))),
        text,
    )])
}

/// "Players can't cycle cards." (stops typecycling too, CR 702.29f).
fn cant_cycle(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if end(l) != "players can't cycle cards" {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Restriction(
            Restriction::Custom(crate::kw::cycling::PLAYERS_CANT_CYCLE.into()),
        ))),
        text,
    )])
}

/// "Creature spells you cast have sticker kicker {1}." (CR 702.33h): the keyword is granted
/// to those spells while they're on the stack, where it functions.
fn spells_you_cast_have_sticker_kicker(
    l: &str,
    text: &str,
    ctx: &CompileContext,
) -> Option<Vec<Ability>> {
    let (subject, granted) = end(l).split_once(" spells you cast have ")?;
    if !granted.starts_with("sticker kicker ") {
        return None;
    }
    let kws: Vec<crate::keywords::Keyword> =
        crate::oracle::keywords::parse_keyword_line(granted, ctx)?
            .into_iter()
            .filter_map(|a| match &a.kind {
                AbilityKind::Keyword(k) => Some(k.clone()),
                _ => None,
            })
            .collect();
    if kws.len() != 1 {
        return None;
    }
    let (f, _, tail) = crate::oracle::phrases::parse_object_phrase(subject)?;
    if !end(tail).is_empty() {
        return None;
    }
    let affected = Filter::and(vec![f, Filter::Spell, Filter::ControlledBy(PlayerRel::You)]);
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
            affected,
            mods: kws.into_iter().map(Modification::AddKeyword).collect(),
        })),
        text,
    )])
}

inventory::submit! {
    StaticPattern { name: "k702.27-37: keyword cost changes", priority: 50, parse: keyword_cost_change }
}
inventory::submit! {
    StaticPattern { name: "k702.33h: spells you cast have sticker kicker", priority: 50, parse: spells_you_cast_have_sticker_kicker }
}
inventory::submit! {
    StaticPattern { name: "k702.29: players can't cycle", priority: 50, parse: cant_cycle }
}
inventory::submit! {
    StaticPattern { name: "k702.28: blocks creatures with shadow", priority: 50, parse: blocks_shadow }
}
inventory::submit! {
    ConditionPattern { name: "k702.33f: kicked with a specific kicker", priority: 50, parse: kicked_with }
}
