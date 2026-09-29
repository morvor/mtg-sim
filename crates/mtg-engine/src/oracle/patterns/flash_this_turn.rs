//! "You may cast sorcery spells this turn as though they had flash." (Complete the
//! Circuit), "You may cast spells this turn as though they had flash." (Borne Upon a
//! Wind, Emergence Zone): a resolved effect that lets its controller cast those spells
//! any time they could cast an instant until the turn ends (CR 601.3b, 611.2a).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn flash_this_turn(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (r, duration) = if let Some(r) = l.strip_prefix("until end of turn, you may cast ") {
        (r.strip_suffix(" as though they had flash")?, Duration::EndOfTurn)
    } else {
        let r = l.strip_prefix("you may cast ")?;
        (
            r.strip_suffix(" this turn as though they had flash")?,
            Duration::EndOfTurn,
        )
    };
    let what = super::k702_001_010::spells_phrase(r)?;
    Some(Effect::AddPlayerEffect {
        who: PlayerRef::You,
        effect: PlayerModification::FlashPermission(what),
        duration,
    })
}

inventory::submit! { EffectPattern { name: "you may cast spells this turn as though they had flash", priority: 100, parse: flash_this_turn } }
