//! "[You may] cast a[n] [quality] spell from your hand with mana value less than or equal
//! to that damage without paying its mana cost" (Glamdring, Buster Sword): as the ability
//! resolves, its controller may choose a card in their hand with that quality and mana
//! value (the damage dealt by the triggering event) and cast it (CR 608.2g) without paying
//! its mana cost (CR 118.9). See `cast_from_hand_free.rs` for the other word order ("a
//! spell with mana value N or less from your hand").

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::types::CardType;

fn cast_from_hand_free_mv(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l)
        .strip_prefix("cast ")?
        .strip_suffix(" without paying its mana cost")?;
    let (spell, mv) = r.split_once(" from your hand with mana value ")?;
    let (cmp, v) = match mv {
        "less than or equal to that damage" => (Cmp::Le, Value::EventAmount),
        _ => return None,
    };
    let spell = spell
        .strip_prefix("a ")
        .or_else(|| spell.strip_prefix("an "))?;
    let before = spell.strip_suffix("spell")?;
    let mut filter = vec![
        Filter::InZone(ZoneKind::Hand),
        Filter::OwnedBy(PlayerRel::You),
        Filter::Not(Box::new(Filter::Type(CardType::Land))),
        Filter::ManaValue(cmp, Box::new(v)),
    ];
    if !before.is_empty() {
        let desc = format!("{before}card");
        let (quality, _, tail) = parse_object_phrase(&desc)?;
        if !end(tail).trim().is_empty() {
            return None;
        }
        filter.push(quality);
    }
    Some(Effect::CastCard {
        who: PlayerRef::You,
        // Choosing none is not casting one.
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(filter),
            count: Value::c(1),
            up_to: true,
            store: None,
        },
        free: true,
        optional: false,
    })
}

inventory::submit! { EffectPattern { name: "cast a spell from your hand with mana value <= that damage without paying its mana cost", priority: 100, parse: cast_from_hand_free_mv } }
