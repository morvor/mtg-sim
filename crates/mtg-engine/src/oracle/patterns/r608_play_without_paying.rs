//! "You may play that card without paying its mana cost." (Djinn of Wishes, Mind's
//! Desire-style effects): as the effect resolves, the player may play the card — a land
//! is played (only during their turn with a land play left, CR 305.2b, 305.3), any other
//! card is cast without paying its mana cost (CR 608.2g, 118.9). "If you don't" reads
//! whether it was played.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::end;

fn play_without_paying(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (optional, r) = match l.strip_prefix("you may ") {
        Some(r) => (true, r),
        None => (false, l),
    };
    let r = r.strip_prefix("play ")?;
    let saved_targets = b.targets.len();
    let saved_it = b.it.clone();
    if let Some((what, rest)) = object_ref(r, b) {
        if rest.trim() == "without paying its mana cost" && !matches!(what, Sel::None) {
            return Some(Effect::PlayCard {
                who: PlayerRef::You,
                what,
                free: true,
                optional,
            });
        }
    }
    b.targets.truncate(saved_targets);
    b.it = saved_it;
    None
}

inventory::submit! { EffectPattern { name: "r608 play it without paying its mana cost", priority: 100, parse: play_without_paying } }
