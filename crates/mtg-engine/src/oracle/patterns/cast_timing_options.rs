//! Ways to cast spells that depend on which spell it is or how it's proposed (CR 601.3b,
//! 601.3c, 601.2e, 118.9):
//!
//! * "You may cast this spell as though it had flash if X is 3 or less." (Silver
//!   Scrutiny), "... if it targets a permanent you control." (Flash Photography): a flash
//!   permission for the card with a quality its proposal must have (`kw/proposal_qualities.rs`);
//! * "You may cast the first creature spell you cast each turn as though it had flash."
//!   (The Blue Spirit) and "The first creature spell you cast each turn costs {2} less to
//!   cast and can be cast as though it had flash." (Radagast of Rhosgobel): a flash
//!   permission for the first such spell (`kw/first_spell_each_turn.rs`);
//! * "The first spell each player casts during each of their turns may be cast without
//!   paying its mana cost." (Weftwalking): an alternative cost offered for that spell
//!   (`kw/offered_costs.rs`).

use super::{AbilityPattern, StaticPattern};
use crate::ability::*;
use crate::kw::proposal_qualities::{CASTER_IS_ACTIVE, X_AT_MOST};
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

/// A quality of the proposed spell: "X is 3 or less", "it targets a permanent you
/// control". Before the card is a spell, any choice could give it the quality.
fn proposal_quality(s: &str) -> Option<Filter> {
    let q = if let Some(n) = s
        .strip_prefix("x is ")
        .and_then(|r| r.strip_suffix(" or less"))
    {
        let n: u32 = n.parse().ok()?;
        Filter::Custom(format!("{X_AT_MOST}{n}").into())
    } else {
        let t = s.strip_prefix("it targets ")?;
        super::spell_cost_grammar::targeted(t)?
    };
    Some(Filter::Or(vec![
        Filter::not(Filter::InZone(ZoneKind::Stack)),
        q,
    ]))
}

/// "You may cast ~ as though it had flash if [quality of the proposal]."
fn a_flash_if_proposal(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let lower = block.trim().to_lowercase();
    let l = end(&lower);
    let q = l.strip_prefix("you may cast ~ as though it had flash if ")?;
    let quality = proposal_quality(q)?;
    let mut s = StaticAbility::new(StaticEffect::FlashPermission {
        who: PlayerRel::You,
        what: Filter::and(vec![Filter::Source, quality]),
    });
    s.zone = FunctionZone::Anywhere;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), block.trim())])
}

inventory::submit! { AbilityPattern { name: "cast timing options: you may cast ~ as though it had flash if [quality of the proposal]", priority: 110, parse: a_flash_if_proposal } }

/// The first [quality] spell you cast each turn (as `parse_subject` reads it), as a filter.
fn first_spell(subject: &str) -> Option<Filter> {
    let subj = super::spell_cost_grammar::parse_subject(subject)?;
    if subj.who != PlayerRel::You
        || subj.ordinal != Some(super::spell_cost_grammar::Ordinal::First)
        || !subj.conds.is_empty()
        || subj.this_turn
    {
        return None;
    }
    let mut parts = subj.parts;
    parts.push(Filter::ControlledBy(PlayerRel::You));
    parts.push(Filter::Custom(
        crate::kw::first_spell_each_turn::FIRST_THIS_TURN.into(),
    ));
    Some(Filter::And(parts))
}

fn flash_permission(what: Filter, text: &str) -> Ability {
    AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::FlashPermission {
            who: PlayerRel::You,
            what,
        })),
        text,
    )
}

fn s_first_spell_options(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l.trim());
    // "You may cast the first creature spell you cast each turn as though it had flash."
    if let Some(subject) = l
        .strip_prefix("you may cast ")
        .and_then(|r| r.strip_suffix(" as though it had flash"))
    {
        if !subject.starts_with("the first ") {
            return None;
        }
        let what = first_spell(subject)?;
        return Some(vec![flash_permission(what, text)]);
    }
    // "The first creature spell you cast each turn costs {2} less to cast and can be cast
    // as though it had flash."
    if let Some(r) = l.strip_suffix(" and can be cast as though it had flash") {
        let (subject, _) = r.split_once(" costs ")?;
        let what = first_spell(subject)?;
        let mut out = crate::oracle::statics::parse_static(&format!("{r}."), ctx)?;
        out.push(flash_permission(what, text));
        return Some(out);
    }
    // "The first spell each player casts during each of their turns may be cast without
    // paying its mana cost."
    if l == "the first spell each player casts during each of their turns may be cast without paying its mana cost"
    {
        let s = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
            applies_to: CostTarget::Spells(Filter::And(vec![
                Filter::Spell,
                Filter::Custom(CASTER_IS_ACTIVE.into()),
                Filter::Custom(crate::kw::first_spell_each_turn::FIRST_THIS_TURN.into()),
            ])),
            who: PlayerRel::Any,
            change: CostChange::AlternativeCost(Cost::free()),
        }));
        return Some(vec![AbilityDef::new(AbilityKind::Static(s), text)]);
    }
    None
}

inventory::submit! { StaticPattern { name: "cast timing options: the first spell each turn (flash, free)", priority: 110, parse: s_first_spell_options } }
