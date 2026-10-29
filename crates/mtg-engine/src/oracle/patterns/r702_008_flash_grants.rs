//! Flash granted to cards (CR 702.8a: flash functions in any zone from which the card
//! could be played): "Creature cards you own that aren't on the battlefield have flash."
//! (Teferi, Mage of Zhalfir; Teferi, Druid of Argoth).

use super::StaticPattern;
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

/// "[Quality] cards you own that aren't on the battlefield have flash."
fn cards_off_the_battlefield_have_flash(
    l: &str,
    text: &str,
    _ctx: &CompileContext,
) -> Option<Vec<Ability>> {
    let subject = end(l)
        .trim()
        .strip_suffix(" cards you own that aren't on the battlefield have flash")?;
    let phrase = format!("{subject} card");
    let (quality, _, tail) = parse_object_phrase(&phrase)?;
    if !end(tail).trim().is_empty() {
        return None;
    }
    // A continuous effect applies to the objects of one zone (see `Filter::zone`): one
    // static ability for each zone a card can be played from other than the battlefield.
    let zones = [
        ZoneKind::Hand,
        ZoneKind::Graveyard,
        ZoneKind::Exile,
        ZoneKind::Library,
        ZoneKind::Command,
    ];
    Some(
        zones
            .into_iter()
            .map(|z| {
                let s = StaticAbility::new(StaticEffect::Continuous {
                    affected: Filter::and(vec![
                        Filter::InZone(z),
                        quality.clone(),
                        Filter::Card,
                        Filter::OwnedBy(PlayerRel::You),
                    ]),
                    mods: vec![Modification::AddKeyword(
                        Keyword::new(KeywordKind::Flash).text("flash"),
                    )],
                });
                AbilityDef::new(AbilityKind::Static(s), text)
            })
            .collect(),
    )
}

inventory::submit! { StaticPattern { name: "[quality] cards you own that aren't on the battlefield have flash", priority: 100, parse: cards_off_the_battlefield_have_flash } }
