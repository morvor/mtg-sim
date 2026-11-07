//! Discarding, then drawing that many: "Discard up to two cards, then draw that many
//! cards.", "Discard any number of cards, then draw that many cards plus one.", "Discard
//! all the cards in your hand, then draw that many cards.", "defending player discards all
//! the cards in their hand, then draws that many cards".
//!
//! The player chooses which cards to discard (CR 701.9a); "that many" is the number of
//! cards actually discarded (CR 701.9), so a discard that doesn't happen draws nothing.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{player_ref, Builder};
use crate::oracle::phrases::*;

/// The cards chosen to be discarded.
const CHOSEN: Var = vars::USER + 1401;

inventory::submit! {
    EffectPattern { name: "sweep: discard, then draw that many", priority: 95, parse: discard_then_draw }
}

/// ", then draw that many cards[ plus one]" (`draws` for a third-person subject): the
/// number of cards to draw beyond "that many".
fn draw_that_many(r: &str, draws: bool) -> Option<i32> {
    let verb = if draws { "draws" } else { "draw" };
    let r = r.strip_prefix(", then ")?.strip_prefix(verb)?;
    match r {
        " that many cards" => Some(0),
        " that many cards plus one" => Some(1),
        _ => None,
    }
}

fn discard_then_draw(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, r, third_person) = if let Some(r) = l.strip_prefix("discard ") {
        (PlayerRef::You, r.to_string(), false)
    } else {
        if l.starts_with("each ") || l.starts_with("you ") {
            return None;
        }
        let saved = b.targets.len();
        let (who, rest) = player_ref(l, b)?;
        match rest.trim_start().strip_prefix("discards ") {
            // Only a single player ("defending player", "target opponent").
            Some(r)
                if matches!(
                    who,
                    PlayerRef::Target(_) | PlayerRef::DefendingPlayer | PlayerRef::TriggerPlayer
                ) =>
            {
                (who, r.to_string(), true)
            }
            _ => {
                b.targets.truncate(saved);
                return None;
            }
        }
    };
    let r = r.as_str();
    let hand_words = if third_person {
        ["all the cards in their hand", "their hand"]
    } else {
        ["all the cards in your hand", "your hand"]
    };
    let (discard, rest) = if let Some(rest) = hand_words.iter().find_map(|w| r.strip_prefix(w)) {
        (Effect::DiscardHand { who: who.clone() }, rest)
    } else if third_person {
        return None;
    } else {
        let (count, rest) = if let Some(x) = r.strip_prefix("any number of cards") {
            (Value::HandSize(PlayerRef::You), x)
        } else {
            let x = r.strip_prefix("up to ")?;
            let (n, x) = parse_number(x)?;
            n.as_const()?;
            let x = x
                .trim_start()
                .strip_prefix("cards")
                .or_else(|| x.trim_start().strip_prefix("card"))?;
            (n, x)
        };
        let chosen = Effect::Store {
            var: CHOSEN,
            sel: Sel::Choose {
                chooser: PlayerRef::You,
                filter: Filter::and(vec![
                    Filter::Card,
                    Filter::InZone(ZoneKind::Hand),
                    Filter::OwnedBy(PlayerRel::You),
                ]),
                count,
                up_to: true,
                store: None,
            },
        };
        let discard = Effect::Discard {
            who: PlayerRef::You,
            n: Value::CountSel(Box::new(Sel::Var(CHOSEN))),
            random: false,
            filter: Filter::In(Box::new(Sel::Var(CHOSEN))),
        };
        (Effect::seq(vec![chosen, discard]), rest)
    };
    let extra = draw_that_many(rest, third_person)?;
    let n = if extra == 0 {
        Value::Prev
    } else {
        Value::Sum(vec![Value::Prev, Value::Const(extra)])
    };
    Some(Effect::seq(vec![discard, Effect::Draw { who, n }]))
}
