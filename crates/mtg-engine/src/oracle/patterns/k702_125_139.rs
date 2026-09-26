//! Oracle text that goes with the keywords of CR 702.125–702.139:
//!
//! * embalm and eternalize: "Whenever you activate an eternalize or embalm ability";
//! * embalm (CR 702.128b): "You may have ~ enter as a copy of any creature on the
//!   battlefield, except if ~ was embalmed, the token has no mana cost, it's white, and
//!   it's a Zombie in addition to its other types.";
//! * ascend on an instant or sorcery (CR 702.131a): the keyword and its spell ability,
//!   which comes first;
//! * companion conditions (CR 702.139a) the general companion pattern doesn't read: different
//!   names, even or odd mana values, activated abilities, a shared card type, a minimum
//!   deck size, and repeated mana symbols;
//! * granted keywords: "[Quality] spells you cast have improvise", "The next spell you
//!   cast this turn has improvise", "... gains embalm until end of turn. The embalm cost is
//!   equal to its mana cost.", "... gains escape until end of turn. The escape cost is
//!   equal to its mana cost plus [cost].";
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
    // "instead only you draw a card": "only" restates who's affected.
    let x = x.strip_prefix("only ").unwrap_or(x);
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

// ---------------------------------------------------------------------------
// Embalm (CR 702.128)
// ---------------------------------------------------------------------------

/// "You may have ~ enter as a copy of any creature on the battlefield, except if ~ was
/// embalmed, the token has no mana cost, it's white, and it's a Zombie in addition to its
/// other types." (Vizier of Many Faces): the exceptions apply only if the entering
/// permanent is an embalmed token (CR 702.128b, 707.9f).
fn enter_as_copy_if_embalmed(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() {
        return None;
    }
    let lower = block.to_lowercase();
    let r = end(&lower).strip_prefix("you may have ~ enter as a copy of ")?;
    let (what, exc) = r.split_once(", except if ~ was embalmed, ")?;
    let what = what.strip_prefix("any ")?;
    let what = what.strip_suffix(" on the battlefield").unwrap_or(what);
    let (filter, _, tail) = parse_object_phrase(what)?;
    if !end(tail).is_empty() {
        return None;
    }
    let mut mods = Vec::new();
    for part in exc.replace(", and ", ", ").split(", ") {
        let part = part.trim();
        let part = part
            .strip_prefix("the token ")
            .map(|p| format!("it {p}"))
            .unwrap_or_else(|| part.to_string());
        match part.as_str() {
            "it has no mana cost" => mods.push(Modification::NoManaCost),
            _ => {
                if let Some(c) = part
                    .strip_prefix("it's ")
                    .and_then(|c| crate::types::Color::from_word(c))
                {
                    mods.push(Modification::SetColors(crate::types::ColorSet::single(c)));
                } else if let Some(t) = part
                    .strip_prefix("it's a ")
                    .and_then(|t| t.strip_suffix(" in addition to its other types"))
                {
                    let st = crate::oracle::phrases::subtype_word(t)?;
                    mods.push(Modification::AddSubtypes(vec![st]));
                } else {
                    return None;
                }
            }
        }
    }
    let rep = |action, self_replacement| {
        AbilityDef::new(
            AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
                ReplacementDef {
                    event: ReplacementEvent::EntersBattlefield(Filter::Source),
                    action,
                    self_replacement,
                    optional: false,
                },
            ))),
            block,
        )
    };
    Some(vec![
        rep(
            ReplacementAction::EnterAsCopy {
                filter,
                optional: true,
            },
            false,
        ),
        rep(
            ReplacementAction::AsEnters(Box::new(Effect::If {
                cond: Condition::Custom(SmolStr::new(crate::kw::embalm::EMBALMED)),
                then: Box::new(Effect::EnterCopyExceptions(mods)),
                otherwise: Box::new(Effect::Noop),
            })),
            // Applied first, while the entering permanent still has this ability; the
            // exceptions belong to the copy effect it then enters with.
            true,
        ),
    ])
}

inventory::submit! { AbilityPattern { name: "k702.128 enter as a copy, except if ~ was embalmed", priority: 90, parse: enter_as_copy_if_embalmed } }

// ---------------------------------------------------------------------------
// Ascend (CR 702.131)
// ---------------------------------------------------------------------------

/// "Ascend" on an instant or sorcery: the keyword plus the spell ability it represents
/// (CR 702.131a), in the keyword's place, so it's performed before the rest of the spell.
fn ascend_spell(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = block.trim().trim_end_matches('.').to_lowercase();
    if l != "ascend" {
        return None;
    }
    let tl = &ctx.type_line.card_types;
    if !tl.contains(crate::types::CardType::Instant)
        && !tl.contains(crate::types::CardType::Sorcery)
    {
        return None;
    }
    Some(vec![
        AbilityDef::new(
            AbilityKind::Keyword(Keyword::new(KeywordKind::Ascend)),
            block.trim(),
        ),
        AbilityDef::new(
            AbilityKind::Spell(SpellAbility {
                body: Body::effect(Effect::Custom(SmolStr::new(crate::kw::ascend::ASCEND))),
            }),
            block.trim(),
        ),
    ])
}

inventory::submit! { AbilityPattern { name: "k702.131 ascend on an instant or sorcery", priority: 100, parse: ascend_spell } }

// ---------------------------------------------------------------------------
// Companion (CR 702.139)
// ---------------------------------------------------------------------------

/// "Companion — [condition]" for the conditions of Lutri, Gyruda, Obosh, Zirda, Umori,
/// Yorion, and Jegantha: the keyword plus its condition on the starting deck, which
/// functions outside the game (CR 103.2b, 702.139a).
fn companion_conditions(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    use crate::kw::companion::{EVEN_MANA_VALUE, HAS_ACTIVATED_ABILITY, ODD_MANA_VALUE};
    use crate::start::DeckCondition;
    let t = block.trim();
    let l = t.to_lowercase();
    let cond = l
        .strip_prefix("companion — ")
        .or_else(|| l.strip_prefix("companion—"))?;
    let cond = cond.split_once(" (").map_or(cond, |(c, _)| c);
    let nonland = || Filter::not(Filter::Type(crate::types::CardType::Land));
    let custom = |n: &str| Filter::Custom(SmolStr::new(n));
    let dc = match end(cond) {
        "each nonland card in your starting deck has a different name" => {
            DeckCondition::DifferentNames { each: nonland() }
        }
        "your starting deck contains only cards with even mana values" => DeckCondition::Each {
            each: Filter::Any,
            must: custom(EVEN_MANA_VALUE),
        },
        "your starting deck contains only cards with odd mana values and land cards" => {
            DeckCondition::Each {
                each: nonland(),
                must: custom(ODD_MANA_VALUE),
            }
        }
        "each permanent card in your starting deck has an activated ability" => {
            DeckCondition::Each {
                each: Filter::PermanentCard,
                must: custom(HAS_ACTIVATED_ABILITY),
            }
        }
        "each nonland card in your starting deck shares a card type" => {
            DeckCondition::ShareACardType { each: nonland() }
        }
        "your starting deck contains at least twenty cards more than the minimum deck size" => {
            DeckCondition::MoreThanMinimumSize(20)
        }
        "no card in your starting deck has more than one of the same mana symbol in its mana cost" => {
            DeckCondition::NoRepeatedManaSymbol
        }
        _ => return None,
    };
    let mut kw = Keyword::new(KeywordKind::Companion);
    kw.text = Some(t.into());
    let mut s = StaticAbility::new(StaticEffect::Companion(dc));
    s.zone = FunctionZone::Anywhere;
    Some(vec![
        AbilityDef::new(AbilityKind::Keyword(kw), t),
        AbilityDef::new(AbilityKind::Static(s), t),
    ])
}

inventory::submit! { AbilityPattern { name: "k702.139 companion conditions", priority: -2, parse: companion_conditions } }

// ---------------------------------------------------------------------------
// Granted keywords
// ---------------------------------------------------------------------------

/// "Nonartifact spells you cast have improvise.", "Noncreature spells you cast have
/// improvise.": the spells have improvise as they're cast (CR 702.126a).
fn spells_you_cast_have_improvise(
    l: &str,
    text: &str,
    _ctx: &CompileContext,
) -> Option<Vec<Ability>> {
    let subject = end(l)
        .strip_suffix("spells you cast have improvise")?
        .trim();
    let mut parts = vec![Filter::Spell, Filter::ControlledBy(PlayerRel::You)];
    if !subject.is_empty() {
        let phrase = format!("{subject} spell");
        let (f, _, tail) = parse_object_phrase(&phrase)?;
        if !end(tail).is_empty() {
            return None;
        }
        parts.insert(0, f);
    }
    let s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::and(parts),
        mods: vec![Modification::AddKeyword(Keyword::new(
            KeywordKind::Improvise,
        ))],
    });
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "k702.126 spells you cast have improvise", priority: 90, parse: spells_you_cast_have_improvise } }

/// "The next spell you cast this turn has improvise." (CR 611.2f): it has improvise as
/// it's cast.
fn next_spell_has_improvise(l: &str, _b: &mut Builder) -> Option<Effect> {
    (end(l) == "the next spell you cast this turn has improvise").then(|| Effect::NextSpell {
        filter: Filter::Spell,
        mods: vec![Modification::AddKeyword(Keyword::new(
            KeywordKind::Improvise,
        ))],
        expires: Duration::EndOfTurn,
    })
}

inventory::submit! { EffectPattern { name: "k702.126 the next spell you cast this turn has improvise", priority: 90, parse: next_spell_has_improvise } }

/// The keyword of `kind` without a cost of its own that `e` grants, to be given one.
fn costless_granted_keyword(e: &mut Effect, kind: KeywordKind) -> Option<&mut Keyword> {
    match e {
        Effect::Modify { mods, .. } => mods.iter_mut().find_map(|m| match m {
            Modification::AddKeyword(k) if k.kind == kind && k.cost.is_none() => Some(k),
            _ => None,
        }),
        Effect::Seq(v) => v.iter_mut().find_map(|x| costless_granted_keyword(x, kind)),
        Effect::If { then, .. } => costless_granted_keyword(then, kind),
        Effect::May { effect, .. } | Effect::ForEach { effect, .. } => {
            costless_granted_keyword(effect, kind)
        }
        _ => None,
    }
}

/// "[card] gains embalm until end of turn. The embalm cost is equal to its mana cost."
/// (a granted embalm ability without a cost of its own costs the card's mana cost, see
/// `kw/embalm.rs`); "[card] gains escape until end of turn. The escape cost is equal to
/// its mana cost plus exile three other cards from your graveyard." (see `kw/escape.rs`).
fn granted_keyword_cost(s: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(s);
    if matches!(
        l,
        "the embalm cost is equal to its mana cost"
            | "the embalm cost is equal to that card's mana cost"
    ) {
        return costless_granted_keyword(prev, KeywordKind::Embalm).is_some();
    }
    let Some(plus) = l
        .strip_prefix("the escape cost is equal to its mana cost plus ")
        .or_else(|| l.strip_prefix("the escape cost is equal to that card's mana cost plus "))
    else {
        return false;
    };
    let Some((cost, _)) = crate::oracle::costs::parse_cost(plus) else {
        return false;
    };
    if cost.mana.is_some() || cost.parts.is_empty() {
        return false;
    }
    match costless_granted_keyword(prev, KeywordKind::Escape) {
        Some(k) => {
            k.cost = Some(cost);
            k.text = Some(SmolStr::new(crate::kw::escape::MANA_COST_PLUS));
            true
        }
        None => false,
    }
}

inventory::submit! { FollowupPattern { name: "k702.128/138 granted embalm or escape cost", priority: 90, apply: granted_keyword_cost } }

/// "Whenever you activate an eternalize or embalm ability" (Vizier of the Anointed).
fn activate_embalm_or_eternalize(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    if !matches!(
        end(r),
        "you activate an eternalize or embalm ability"
            | "you activate an embalm or eternalize ability"
    ) {
        return None;
    }
    Some((
        TriggerCond::Where {
            trigger: Box::new(TriggerCond::AbilityActivated {
                who: PlayerRel::You,
                source: Filter::Any,
                include_mana: false,
            }),
            cond: Condition::Custom(SmolStr::new(
                crate::kw::embalm::EMBALM_OR_ETERNALIZE_ACTIVATED,
            )),
        },
        Sel::This,
        PlayerRef::You,
    ))
}

inventory::submit! { TriggerPattern { name: "k702.128 you activate an eternalize or embalm ability", priority: 100, parse: activate_embalm_or_eternalize } }
