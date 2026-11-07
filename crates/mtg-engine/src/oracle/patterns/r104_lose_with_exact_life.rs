//! "Each player with exactly N life loses the game" (Triskaidekaphobia; CR 104.3e): the
//! players whose life total is exactly N as the effect happens lose the game at once.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_number};

fn exact_life_loses(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("each player with exactly ")?;
    let (n, r) = parse_number(r)?;
    if end(r) != "life loses the game" {
        return None;
    }
    Some(Effect::LoseGame {
        who: PlayerRef::Each(PlayerFilter::Life(Cmp::Eq, Box::new(n))),
    })
}

inventory::submit! { EffectPattern { name: "r104 each player with exactly N life loses the game", priority: 100, parse: exact_life_loses } }
