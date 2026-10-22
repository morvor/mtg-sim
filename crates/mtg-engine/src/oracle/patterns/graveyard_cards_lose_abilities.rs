//! "Cards in graveyards lose all abilities." (Yixlid Jailer), "Creature cards in
//! graveyards lose all abilities.": a layer 6 effect (CR 613.1f) on cards in graveyards,
//! so abilities that function from a graveyard (CR 113.6) — and triggered abilities that
//! would trigger as a card is put into one — stop working.

use crate::ability::*;
use crate::oracle::phrases::parse_object_phrase;
use crate::oracle::CompileContext;

fn graveyard_cards_lose_all_abilities(
    l: &str,
    text: &str,
    _ctx: &CompileContext,
) -> Option<Vec<Ability>> {
    let subject = l.strip_suffix(" in graveyards lose all abilities")?;
    let quality = if subject == "cards" {
        Filter::Card
    } else {
        let (f, _, tail) = parse_object_phrase(subject.strip_suffix('s')?)?;
        if !tail.trim().is_empty() {
            return None;
        }
        Filter::and(vec![f, Filter::Card])
    };
    let s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::and(vec![quality, Filter::InZone(ZoneKind::Graveyard)]),
        mods: vec![Modification::RemoveAllAbilities],
    });
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { crate::oracle::patterns::StaticPattern { name: "cards in graveyards lose all abilities", priority: 100, parse: graveyard_cards_lose_all_abilities } }
