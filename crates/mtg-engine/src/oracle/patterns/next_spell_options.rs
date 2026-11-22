//! Casting options for "the next spell you cast this turn" (CR 611.2f, 118.9, 601.3b): a
//! waiting effect (`Effect::NextSpell`) that offers the next matching spell its
//! controller casts a way to be cast, used up by that spell however it's cast.
//!
//! * "The next red or green creature spell you cast this turn can be cast without paying
//!   its mana cost." (World War Hulk), "The next spell you cast this turn of the chosen
//!   type can be cast without paying its mana cost." (Apex Observatory): an alternative
//!   cost of nothing for that spell (CR 118.9), offered like the alternative costs other
//!   objects offer (`kw/offered_costs.rs`);
//! * "Rather than pay the mana cost of the next spell you cast this turn, you may pay life
//!   equal to that spell's mana value." (Marshland Bloodcaster): that alternative cost;
//! * "The next spell of the chosen type you cast this turn can be cast as though it had
//!   flash." (Progenitor's Icon);
//! * after such an effect, "That spell can't be countered." and "That creature enters with
//!   an additional +1/+1 counter on it." (Savage Summoning): the spell gains those
//!   abilities as it's cast; the second applies as the permanent it becomes enters
//!   (CR 400.7a, 614.1c).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// "[quality] spell you cast this turn" (or "spell you cast this turn of the chosen type",
/// "spell of the chosen type you cast this turn") followed by `tail`: the spell's
/// qualities.
fn next_spell(r: &str, tail: &str) -> Option<Filter> {
    let head = r.strip_suffix(tail)?;
    let quality = if let Some(q) = head.strip_suffix(" you cast this turn of the chosen type") {
        format!("{q} of the chosen type")
    } else {
        head.strip_suffix(" you cast this turn")?.to_string()
    };
    let parts = super::spell_cost_grammar::spell_noun(&quality, true)?;
    let f = Filter::and(parts);
    if serde_json::to_string(&f).is_ok_and(|s| s.contains("\"X\"")) {
        return None;
    }
    Some(f)
}

/// The ability "You may pay [cost] rather than pay this spell's mana cost." the next spell
/// gains (CR 118.9).
fn alternative_cost(cost: Cost, text: &str) -> Modification {
    let mut s = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::ThisSpell,
        who: PlayerRel::You,
        change: CostChange::AlternativeCost(cost),
    }));
    s.zone = FunctionZone::Anywhere;
    Modification::AddAbility(AbilityDef::new(AbilityKind::Static(s), text))
}

fn p_next_spell_option(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    // "Rather than pay the mana cost of the next spell you cast this turn, you may pay
    // life equal to that spell's mana value."
    if let Some(r) = l.strip_prefix("rather than pay the mana cost of the next ") {
        let filter = next_spell(
            r,
            ", you may pay life equal to that spell's mana value",
        )?;
        let cost = Cost {
            mana: None,
            parts: vec![CostPart::PayLife(Value::ManaValueOf(Box::new(Sel::This)))],
        };
        return Some(Effect::NextSpell {
            filter,
            mods: vec![alternative_cost(
                cost,
                "You may pay life equal to this spell's mana value rather than pay its mana cost.",
            )],
            expires: Duration::EndOfTurn,
        });
    }
    let r = l.strip_prefix("the next ")?;
    if let Some(filter) = next_spell(r, " can be cast without paying its mana cost") {
        return Some(Effect::NextSpell {
            filter,
            mods: vec![alternative_cost(
                Cost::free(),
                "You may cast this spell without paying its mana cost.",
            )],
            expires: Duration::EndOfTurn,
        });
    }
    // "The next spell of the chosen type you cast this turn can be cast as though it had
    // flash." (the other wordings are `permission_grammar::next_spell_flash`'s).
    if r.starts_with("spell of the chosen type ") || r.contains(" this turn of the chosen type ")
    {
        let filter = next_spell(r, " can be cast as though it had flash")?;
        return Some(Effect::NextSpell {
            filter,
            mods: vec![Modification::AddKeyword(
                Keyword::new(KeywordKind::Flash).text("flash"),
            )],
            expires: Duration::EndOfTurn,
        });
    }
    None
}

inventory::submit! { EffectPattern { name: "next spell options: the next spell can be cast without paying its mana cost / for life", priority: 110, parse: p_next_spell_option } }

/// "the next instant spell and the next sorcery spell you cast this turn each have
/// cascade": a waiting effect for each (CR 611.2f).
fn p_next_two_spells(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let r = l.strip_prefix("the next ")?;
    let (a, rest) = r.split_once(" and the next ")?;
    let (b_kind, granted) = rest.split_once(" you cast this turn each have ")?;
    let first = crate::oracle::effects::parse_effect_text(
        &format!("The next {a} you cast this turn has {granted}."),
        b,
    )?;
    let second = crate::oracle::effects::parse_effect_text(
        &format!("The next {b_kind} you cast this turn has {granted}."),
        b,
    )?;
    if !matches!(first, Effect::NextSpell { .. }) || !matches!(second, Effect::NextSpell { .. }) {
        return None;
    }
    Some(Effect::seq(vec![first, second]))
}

inventory::submit! { EffectPattern { name: "next spell options: the next A and the next B each have [keyword]", priority: 110, parse: p_next_two_spells } }

/// "as you cast spells from your hand this turn, they gain cascade": each such spell this
/// turn gains the keyword as it's cast (CR 611.2f, 601.2a).
fn p_as_you_cast_this_turn(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let r = l.strip_prefix("as you cast ")?;
    let (subject, granted) = r.split_once(" this turn, they gain ")?;
    // "spells from your hand" → "spells you cast from your hand".
    let (noun, zone) = match subject.split_once(" from ") {
        Some((n, z)) => (n, format!(" from {z}")),
        None => (subject, String::new()),
    };
    let subj = super::spell_cost_grammar::parse_subject(&format!("{noun} you cast{zone}"))?;
    if subj.who != PlayerRel::You || subj.ordinal.is_some() || subj.singular || !subj.conds.is_empty() {
        return None;
    }
    let _ = b;
    let mods = crate::oracle::effects::keyword_mods(granted)?;
    let mut parts = subj.parts;
    parts.push(Filter::Custom(crate::next_spell::EACH_SPELL_THIS_TURN.into()));
    Some(Effect::NextSpell {
        filter: Filter::and(parts),
        mods,
        expires: Duration::EndOfTurn,
    })
}

inventory::submit! { EffectPattern { name: "next spell options: as you cast spells this turn, they gain [keyword]", priority: 110, parse: p_as_you_cast_this_turn } }

/// The "next spell" effect an effect ends with.
fn last_next_spell(e: &mut Effect) -> Option<&mut Vec<Modification>> {
    match e {
        Effect::NextSpell { mods, .. } => Some(mods),
        Effect::Seq(v) => v.last_mut().and_then(last_next_spell),
        _ => None,
    }
}

/// "That spell can't be countered.", "That creature enters with an additional +1/+1
/// counter on it." after a "next spell" effect: abilities that spell gains.
fn f_next_spell_also(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l.trim());
    let Some(mods) = last_next_spell(prev) else {
        return false;
    };
    if matches!(l, "that spell can't be countered" | "it can't be countered") {
        mods.push(Modification::AddAbility(
            super::restriction_grammar::cant_be_countered_ability(),
        ));
        return true;
    }
    let Some(r) = l
        .strip_prefix("that creature enters with ")
        .or_else(|| l.strip_prefix("that permanent enters with "))
    else {
        return false;
    };
    let Some((kind, n)) = super::etb_replacement_grammar::counters_on(r) else {
        return false;
    };
    if n.as_const().is_none() {
        return false;
    }
    let s = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
        event: ReplacementEvent::EntersBattlefield(Filter::Source),
        action: ReplacementAction::EnterWithCounters(kind, n),
        self_replacement: false,
        optional: false,
    }));
    mods.push(Modification::AddAbility(AbilityDef::new(
        AbilityKind::Static(s),
        "This permanent enters with additional counters on it.",
    )));
    true
}

inventory::submit! { FollowupPattern { name: "next spell options: that spell can't be countered / enters with counters", priority: 90, apply: f_next_spell_also } }
