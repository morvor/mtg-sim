//! "You and that player each draw [N] cards." (Xyris, the Writhing Storm: "Whenever Xyris
//! deals combat damage to a player, you and that player each draw that many cards."):
//! both players draw, the active player first, then the other in turn order
//! (CR 121.2c). A player who has left the game draws nothing (CR 800.4a).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::patterns::oracle_hardening_referents::is_no_player_referent;
use crate::oracle::phrases::end;

fn you_and_that_player_each_draw(l: &str, b: &mut Builder) -> Option<Effect> {
    let rest = end(l).strip_prefix("you and that player each ")?;
    if !rest.starts_with("draw ") || is_no_player_referent(&b.it_player) {
        return None;
    }
    let that_player = b.it_player.clone();
    if matches!(that_player, PlayerRef::You) {
        return None;
    }
    let Effect::Draw {
        who: PlayerRef::You,
        n,
    } = parse_sentence(&format!("you {rest}"), b)?
    else {
        return None;
    };
    Some(Effect::Draw {
        who: PlayerRef::Each(PlayerFilter::Or(vec![
            PlayerFilter::You,
            PlayerFilter::Ref(Box::new(that_player)),
        ])),
        n,
    })
}

inventory::submit! { EffectPattern { name: "you and that player each draw", priority: 90, parse: you_and_that_player_each_draw } }
