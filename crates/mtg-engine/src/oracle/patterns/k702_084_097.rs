//! Oracle text of the keywords of CR 702.84–702.97 that the generic keyword parser doesn't
//! handle, and phrases that go with them:
//!
//! * "Each [quality] card in your graveyard has unearth [cost]." (CR 702.84a);
//! * "Each creature card in your graveyard has scavenge. The scavenge cost is equal to its
//!   mana cost." (CR 702.97a);
//! * "Auras attached to permanents you control have umbra armor." (CR 702.89a).

use super::StaticPattern;
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

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
