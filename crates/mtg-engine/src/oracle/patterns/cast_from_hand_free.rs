//! "[You may] cast a[n] [quality] spell [with mana value N or less] from your hand without
//! paying its mana cost" (Sram's Expertise, I Am Duskmourn, Wildfire Eternal): as the
//! ability resolves, its controller chooses a card in their hand with that quality and
//! casts it (CR 608.2g) without paying its mana cost (CR 118.9) — or chooses none. Any
//! additional costs may (or, if mandatory, must) still be paid, and no alternative cost
//! can be chosen (CR 118.9a). "If you do" after it asks whether a spell was cast.
//! (The plain "a spell with mana value N or less" form is `r601_cast_from_hand_free.rs`'s,
//! which judges the mana value on each spell the card could be cast as, CR 601.3e.)

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::types::CardType;

fn cast_from_hand_free(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l)
        .strip_prefix("cast ")?
        .strip_suffix(" from your hand without paying its mana cost")?;
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    // "[quality] spell [with ...]": the card it's cast from has that quality.
    let (before, after) = r.split_once("spell")?;
    let desc = format!("{before}card{after}");
    let (quality, _, tail) = parse_object_phrase(&desc)?;
    if !end(tail).trim().is_empty() {
        return None;
    }
    Some(Effect::CastCard {
        who: PlayerRef::You,
        // Choosing none is not casting one.
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![
                Filter::InZone(ZoneKind::Hand),
                Filter::OwnedBy(PlayerRel::You),
                Filter::Not(Box::new(Filter::Type(CardType::Land))),
                quality,
            ]),
            count: Value::c(1),
            up_to: true,
            store: None,
        },
        free: true,
        optional: false,
    })
}

inventory::submit! { EffectPattern { name: "cast a spell from your hand without paying its mana cost", priority: 100, parse: cast_from_hand_free } }
