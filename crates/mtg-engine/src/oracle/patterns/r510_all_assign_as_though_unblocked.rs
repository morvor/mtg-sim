//! "You may have creatures you control assign their combat damage this turn as though
//! they weren't blocked." (Predatory Focus): the choice is made as the spell resolves; if
//! its controller chooses to, for the rest of the turn every creature they control
//! assigns all its combat damage to the player, planeswalker, or battle it's attacking,
//! none to creatures blocking it (CR 510.1c), including creatures that weren't on the
//! battlefield as the spell resolved (an effect that changes the rules of the game, not
//! characteristics, CR 611.2c). See `kw::assign_as_though_unblocked`.

use super::EffectPattern;
use crate::ability::*;
use crate::kw::assign_as_though_unblocked::ALL_ASSIGN_UNBLOCKED;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn all_assign_unblocked(l: &str, _b: &mut Builder) -> Option<Effect> {
    const REST: &str =
        "have creatures you control assign their combat damage this turn as though they weren't blocked";
    let l = end(l);
    // "You may" is usually split off by the caller, which wraps the rest in `Effect::May`.
    let may = match l.strip_prefix("you may ") {
        Some(r) if r == REST => true,
        _ if l == REST => false,
        _ => return None,
    };
    let grant = Effect::AddPlayerEffect {
        who: PlayerRef::You,
        effect: PlayerModification::Custom(ALL_ASSIGN_UNBLOCKED.into()),
        duration: Duration::EndOfTurn,
    };
    Some(if may {
        Effect::May {
            who: PlayerRef::You,
            effect: Box::new(grant),
        }
    } else {
        grant
    })
}

inventory::submit! { EffectPattern { name: "r510 creatures you control assign combat damage as though unblocked this turn", priority: 60, parse: all_assign_unblocked } }
