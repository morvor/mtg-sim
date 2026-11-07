//! "If that player has more cards in hand than you, draw cards equal to the difference."
//! (Sandstone Oracle, after "choose an opponent"): the number of cards is determined once,
//! then that many cards are drawn (CR 121.2, 121.6).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn draw_the_difference(s: &str, b: &mut Builder) -> Option<Effect> {
    if end(s)
        != "if that player has more cards in hand than you, draw cards equal to the difference"
    {
        return None;
    }
    let them = Value::HandSize(b.it_player.clone());
    let you = Value::HandSize(PlayerRef::You);
    Some(Effect::If {
        cond: Condition::Compare(them.clone(), Cmp::Gt, you.clone()),
        then: Box::new(Effect::Draw {
            who: PlayerRef::You,
            n: Value::Diff(Box::new(them), Box::new(you)),
        }),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "if that player has more cards in hand than you, draw the difference", priority: 100, parse: draw_the_difference } }
