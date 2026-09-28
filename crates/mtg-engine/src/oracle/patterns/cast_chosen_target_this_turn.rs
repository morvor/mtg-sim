//! "Choose target artifact card in your graveyard. You may cast that card this turn."
//! (Emry, Lurker of the Loch; Silas Renn, Seeker Adept): a permission to cast the chosen
//! card until end of turn, paying its costs and following the timing rules. The
//! permission is for that object: it ends if the card leaves the graveyard (CR 400.7).
//! It lets the card be cast, never played as a land (CR 305.9): a land card gets none.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::types::CardType;

/// Whether the filter describes cards ("target artifact card in your graveyard").
fn is_card(f: &Filter) -> bool {
    match f {
        Filter::Card => true,
        Filter::And(v) => v.iter().any(is_card),
        _ => false,
    }
}

fn may_cast_that_card_this_turn(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if end(l) != "you may cast that card this turn" || !matches!(prev, Effect::Noop) {
        return false;
    }
    // "Choose target [card]": the sentence before chose a card as the only target.
    if b.targets.len() != 1 {
        return false;
    }
    let is_card = matches!(&b.targets[0].what, TargetKind::Object(f) if is_card(f));
    if !is_card {
        return false;
    }
    let card = Sel::Target(0);
    *prev = Effect::If {
        cond: Condition::SelMatches(
            card.clone(),
            Filter::Not(Box::new(Filter::Type(CardType::Land))),
        ),
        then: Box::new(Effect::GrantPlayPermission {
            who: PlayerRef::You,
            what: card,
            duration: Duration::EndOfTurn,
            free: false,
        }),
        otherwise: Box::new(Effect::Noop),
    };
    true
}

inventory::submit! { FollowupPattern { name: "choose target card. you may cast that card this turn", priority: 90, apply: may_cast_that_card_this_turn } }
