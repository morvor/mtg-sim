//! "You may cast a[n] [card type] spell from your hand [or graveyard] by paying life equal
//! to its mana value rather than paying its mana cost." (Anrakyr the Traveller): a spell
//! cast as the ability resolves for an alternative cost of life. See
//! `kw/cast_paying_life.rs`.

use super::EffectPattern;
use crate::ability::*;
use crate::kw::cast_paying_life::effect_name;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::types::CardType;

fn cast_paying_life(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let l = l.strip_prefix("you may ").unwrap_or(l);
    let r = l.strip_prefix("cast ")?.strip_suffix(
        " by paying life equal to its mana value rather than paying its mana cost",
    )?;
    let (spell, zones) = r.split_once(" from ")?;
    let zones = match zones {
        "your hand" => vec![ZoneKind::Hand],
        "your graveyard" => vec![ZoneKind::Graveyard],
        "your hand or graveyard" | "your hand or your graveyard" => {
            vec![ZoneKind::Hand, ZoneKind::Graveyard]
        }
        _ => return None,
    };
    let kind = match spell {
        "a spell" => None,
        _ => {
            let k = spell
                .strip_prefix("a ")
                .or_else(|| spell.strip_prefix("an "))?
                .strip_suffix(" spell")?;
            Some(CardType::from_word(k)?)
        }
    };
    Some(Effect::Custom(effect_name(&zones, kind).into()))
}

inventory::submit! { EffectPattern { name: "r601 you may cast a spell by paying life equal to its mana value", priority: 100, parse: cast_paying_life } }
