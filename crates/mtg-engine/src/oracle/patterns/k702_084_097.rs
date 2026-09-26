//! Oracle text of the keywords of CR 702.84–702.97 that the generic keyword parser doesn't
//! handle, and phrases that go with them:
//!
//! * "Each [quality] card in your graveyard has unearth [cost]." (CR 702.84a);
//! * "Each creature card in your graveyard has scavenge. The scavenge cost is equal to its
//!   mana cost." (CR 702.97a);
//! * "Auras attached to permanents you control have umbra armor." (CR 702.89a);
//! * "[Quality] spells you cast [from your hand] have cascade." (CR 702.85a) and "As you
//!   cascade, you may put a land card from among the exiled cards onto the battlefield
//!   tapped." (CR 702.85b);
//! * "Instant and sorcery spells you control have rebound." (CR 702.88a);
//! * abilities that refer to paired creatures (CR 702.95b): "As long as ~ is paired with
//!   another creature, each of those creatures ...", "~ can't attack or block unless it's
//!   paired with a creature with soulbond", "If it's paired with a creature, that creature
//!   also gets +N/+N until end of turn", "Whenever ~ or a creature it's paired with is
//!   dealt damage".

use super::{FollowupPattern, StaticPattern, TriggerPattern};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::effects::Builder;
use crate::oracle::CompileContext;
use crate::types::CardType;

/// The filter for "[quality] card" in "Each [quality] card in your graveyard".
fn graveyard_card_quality(subject: &str) -> Option<Filter> {
    let phrase = format!("{subject} card");
    let (f, _, tail) = parse_object_phrase(&phrase)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Filter::and(vec![
        f,
        Filter::Card,
        Filter::InZone(ZoneKind::Graveyard),
        Filter::OwnedBy(PlayerRel::You),
    ]))
}

fn grant(affected: Filter, kw: Keyword, text: &str) -> Vec<Ability> {
    let s = StaticAbility::new(StaticEffect::Continuous {
        affected,
        mods: vec![Modification::AddKeyword(kw)],
    });
    vec![AbilityDef::new(AbilityKind::Static(s), text)]
}

/// "Each creature card in your graveyard has unearth {2}{B}." (Sedris, the Traitor King;
/// Dregscape Sliver): each of those cards has its own unearth ability (CR 702.84a).
fn graveyard_cards_have_unearth(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = l.strip_prefix("each ")?;
    let (subject, cost) = r.split_once(" card in your graveyard has unearth ")?;
    if cost.is_empty() {
        return None;
    }
    let at = text.to_lowercase().find(" has unearth ")? + " has unearth ".len();
    let cost = crate::oracle::keywords::parse_keyword_cost(&text[at..])?;
    let affected = graveyard_card_quality(subject)?;
    Some(grant(
        affected,
        Keyword::with_cost(KeywordKind::Unearth, cost).text("unearth"),
        text,
    ))
}

inventory::submit! { StaticPattern { name: "each [quality] card in your graveyard has unearth", priority: 100, parse: graveyard_cards_have_unearth } }

/// "Each creature card in your graveyard has scavenge. The scavenge cost is equal to its
/// mana cost." (Varolz, the Scar-Striped): a scavenge ability whose cost is the card's
/// mana cost (an {X} in it is 0).
fn graveyard_cards_have_scavenge(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = l.strip_prefix("each ")?;
    let subject = r.strip_suffix(
        " card in your graveyard has scavenge. the scavenge cost is equal to its mana cost",
    )?;
    let affected = graveyard_card_quality(subject)?;
    let cost = Cost {
        mana: None,
        parts: vec![CostPart::PayManaCostOf(Box::new(Sel::This))],
    };
    Some(grant(
        affected,
        Keyword::with_cost(KeywordKind::Scavenge, cost).text("scavenge"),
        text,
    ))
}

inventory::submit! { StaticPattern { name: "each [quality] card in your graveyard has scavenge", priority: 100, parse: graveyard_cards_have_scavenge } }

/// "Auras attached to permanents you control have umbra armor." (Umbra Mystic): whoever
/// controls the Auras (CR 702.89a).
fn auras_have_umbra_armor(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if l != "auras attached to permanents you control have umbra armor" {
        return None;
    }
    let affected = Filter::and(vec![
        Filter::Subtype("Aura".into()),
        Filter::Permanent,
        Filter::Custom(crate::kw::umbra_armor::ATTACHED_TO_YOURS.into()),
    ]);
    Some(grant(
        affected,
        Keyword::new(KeywordKind::UmbraArmor).text("umbra armor"),
        text,
    ))
}

inventory::submit! { StaticPattern { name: "auras attached to permanents you control have umbra armor", priority: 100, parse: auras_have_umbra_armor } }

/// "As you cascade, you may put a land card from among the exiled cards onto the
/// battlefield tapped." (Averna, the Chaos Bloom; CR 702.85b).
fn as_you_cascade(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if l != "as you cascade, you may put a land card from among the exiled cards onto the battlefield tapped" {
        return None;
    }
    let s = StaticAbility::new(StaticEffect::Custom(
        crate::kw::cascade::AS_YOU_CASCADE_LAND.into(),
    ));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "as you cascade, you may put a land card onto the battlefield", priority: 100, parse: as_you_cascade } }

/// "Sliver spells you cast have cascade." (The First Sliver), "Instant and sorcery spells
/// you cast from your hand have cascade." (Quandrix, the Proof), "Commander spells you
/// cast have cascade." (Flamekin Herald), "Spells you cast with mana value 6 or greater
/// have cascade." (Imoti): the spells have cascade as they're cast, so it triggers
/// (CR 702.85a).
fn spells_have_cascade(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = l.strip_suffix(" have cascade")?;
    let (subject, rest) = match r.strip_prefix("spells you cast") {
        Some(rest) => ("", rest),
        None => r.split_once(" spells you cast")?,
    };
    let mut parts: Vec<Filter> = Vec::new();
    let rest = match rest.strip_prefix(" from your hand") {
        Some(x) => {
            parts.push(Filter::CastFrom(ZoneKind::Hand));
            x
        }
        None => rest,
    };
    if let Some(n) = rest
        .strip_prefix(" with mana value ")
        .and_then(|x| x.strip_suffix(" or greater"))
    {
        let n: i32 = n.trim().parse().ok()?;
        parts.push(Filter::ManaValue(Cmp::Ge, Box::new(Value::c(n))));
    } else if !rest.is_empty() {
        return None;
    }
    match subject {
        "" => {}
        "commander" => parts.push(Filter::Commander),
        _ => {
            let mut alts = Vec::new();
            for word in subject.split(" and ") {
                let phrase = format!("{word} card");
                let (f, _, tail) = parse_object_phrase(&phrase)?;
                if !end(tail).is_empty() {
                    return None;
                }
                alts.push(f);
            }
            parts.push(if alts.len() == 1 {
                alts.pop()?
            } else {
                Filter::Or(alts)
            });
        }
    }
    parts.push(Filter::Spell);
    parts.push(Filter::ControlledBy(PlayerRel::You));
    Some(grant(
        Filter::and(parts),
        Keyword::new(KeywordKind::Cascade).text("cascade"),
        text,
    ))
}

inventory::submit! { StaticPattern { name: "[quality] spells you cast have cascade", priority: 100, parse: spells_have_cascade } }

/// "Instant and sorcery spells you control have rebound." (Cast Through Time): the spells
/// have rebound while they're on the stack (CR 702.88a).
fn spells_have_rebound(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if l != "instant and sorcery spells you control have rebound" {
        return None;
    }
    let affected = Filter::and(vec![
        Filter::Or(vec![
            Filter::Type(CardType::Instant),
            Filter::Type(CardType::Sorcery),
        ]),
        Filter::Spell,
        Filter::ControlledBy(PlayerRel::You),
    ]);
    Some(grant(
        affected,
        Keyword::new(KeywordKind::Rebound).text("rebound"),
        text,
    ))
}

inventory::submit! { StaticPattern { name: "instant and sorcery spells you control have rebound", priority: 100, parse: spells_have_rebound } }

/// "As long as ~ is paired with another creature, each of those creatures gets +1/+1."
/// (Trusted Forcemage), "..., both creatures have flying." (Wingcrafter), "..., each of
/// those creatures has "[ability]"." (Doom Weaver): the soulbond creature and the creature
/// it's paired with (CR 702.95b).
fn while_paired(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    const HEAD: &str = "as long as ~ is paired with another creature, ";
    let r = l.strip_prefix(HEAD)?;
    // Quoted abilities keep their original case.
    let orig = text.get(HEAD.len()..).filter(|o| o.to_lowercase().starts_with(r))?;
    let clause = if let Some(x) = orig.strip_prefix("each of those creatures ") {
        x.to_string()
    } else if let Some(x) = r.strip_prefix("both creatures ") {
        if let Some(y) = x.strip_prefix("have ") {
            format!("has {y}")
        } else {
            format!("gets {}", x.strip_prefix("get ")?)
        }
    } else {
        return None;
    };
    let parsed = crate::oracle::statics::parse_static(&format!("~ {clause}"), ctx)?;
    let mut out = Vec::new();
    for a in parsed {
        let AbilityKind::Static(s) = &a.kind else {
            return None;
        };
        let StaticEffect::Continuous {
            affected: Filter::Source,
            mods,
        } = &s.effect
        else {
            return None;
        };
        if s.condition.is_some() {
            return None;
        }
        let mut ns = StaticAbility::new(StaticEffect::Continuous {
            affected: Filter::Custom(crate::kw::soulbond::THE_PAIR.into()),
            mods: mods.clone(),
        });
        ns.condition = Some(Condition::Custom(crate::kw::soulbond::IS_PAIRED.into()));
        out.push(AbilityDef::with_link(AbilityKind::Static(ns), text, a.link));
    }
    (!out.is_empty()).then_some(out)
}

inventory::submit! { StaticPattern { name: "as long as ~ is paired with another creature", priority: 100, parse: while_paired } }

/// "~ can't attack or block unless it's paired with a creature with soulbond." (Flowering
/// Lumberknot).
fn cant_attack_unless_paired(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if l != "~ can't attack or block unless it's paired with a creature with soulbond" {
        return None;
    }
    let mut s = StaticAbility::new(StaticEffect::Restriction(Restriction::CantAttackOrBlock(
        Filter::Source,
    )));
    s.condition = Some(Condition::Not(Box::new(Condition::Custom(
        crate::kw::soulbond::PAIRED_WITH_SOULBOND.into(),
    ))));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "~ can't attack or block unless it's paired with a creature with soulbond", priority: 100, parse: cant_attack_unless_paired } }

/// "+N/+N" → (N, N).
fn pt_mod(s: &str) -> Option<(i32, i32)> {
    let (p, t) = s.split_once('/')?;
    let n = |x: &str| -> Option<i32> {
        match x.strip_prefix('+') {
            Some(v) => v.parse().ok(),
            None => x.parse().ok(),
        }
    };
    Some((n(p)?, n(t)?))
}

/// "If it's paired with a creature, that creature also gets +2/+2 until end of turn."
/// (Joint Assault), after an effect on a target creature: the creature the target is
/// paired with as the spell resolves.
fn that_paired_creature_also_gets(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Some(pt) = l
        .strip_prefix("if it's paired with a creature, that creature also gets ")
        .and_then(|r| r.strip_suffix(" until end of turn"))
    else {
        return false;
    };
    let Some((p, t)) = pt_mod(pt) else {
        return false;
    };
    let about_target = matches!(prev, Effect::Modify { what: Sel::Target(0), .. });
    if !about_target {
        return false;
    }
    let extra = Effect::Modify {
        what: Sel::All(Filter::Custom(
            crate::kw::soulbond::PAIRED_WITH_TARGET.into(),
        )),
        mods: vec![Modification::ModifyPT(Value::c(p), Value::c(t))],
        duration: Duration::EndOfTurn,
    };
    *prev = Effect::Seq(vec![std::mem::take(prev), extra]);
    true
}

inventory::submit! { FollowupPattern { name: "if it's paired with a creature, that creature also gets", priority: 100, apply: that_paired_creature_also_gets } }

/// "~ or a creature it's paired with is dealt damage" (Donna Noble): once for each of them
/// dealt damage at the same time.
fn this_or_paired_dealt_damage(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    if r != "~ or a creature it's paired with is dealt damage" {
        return None;
    }
    Some((
        TriggerCond::Batched {
            trigger: Box::new(TriggerCond::IsDealtDamage {
                filter: Filter::Or(vec![
                    Filter::Source,
                    Filter::Custom(crate::kw::soulbond::PAIRED_WITH_THIS.into()),
                ]),
                combat_only: false,
            }),
            per: BatchPer::Object,
        },
        Sel::TriggerObject,
        PlayerRef::ControllerOf(Box::new(Sel::TriggerObject)),
    ))
}

inventory::submit! { TriggerPattern { name: "~ or a creature it's paired with is dealt damage", priority: 100, parse: this_or_paired_dealt_damage } }
