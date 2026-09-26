//! Oracle text of the keywords of CR 702.84–702.97 that the generic keyword parser doesn't
//! handle, and phrases that go with them:
//!
//! * "Each [quality] card in your graveyard has unearth [cost]." (CR 702.84a);
//! * "Each creature card in your graveyard has scavenge. The scavenge cost is equal to its
//!   mana cost." (CR 702.97a);
//! * "Auras attached to permanents you control have umbra armor." (CR 702.89a);
//! * "[Quality] spells you cast [from your hand] have cascade." (CR 702.85a) and "As you
//!   cascade, you may put a land card from among the exiled cards onto the battlefield
//!   tapped." (CR 702.85b).

use super::StaticPattern;
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::oracle::phrases::{end, parse_object_phrase};
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
