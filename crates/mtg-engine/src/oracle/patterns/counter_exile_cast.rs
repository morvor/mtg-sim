//! "Counter target spell. If [that / a permanent] spell is countered this way, exile it
//! instead of putting it into its owner's graveyard. You may cast that card without
//! paying its mana cost for as long as it remains exiled." (Thranduil's Decree, Kheru
//! Spellsnatcher; Spelljack: "You may play it ..."): the countered card, found in exile
//! (CR 400.7j), may be cast by the ability's controller while it stays there, only
//! without paying its mana cost — no other alternative cost, such as casting it face
//! down, can be used (CR 118.9a). A land card is played, never cast (CR 305.9).

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::types::CardType;

/// Whether the effect counters a spell and exiles it instead of putting it into a
/// graveyard.
fn counters_into_exile(e: &Effect) -> bool {
    match e {
        Effect::SelfReplace {
            replacement:
                ReplacementDef {
                    action: ReplacementAction::MoveInstead(d),
                    ..
                },
            effect,
        } => d.zone == ZoneKind::Exile && matches!(**effect, Effect::CounterSpell { .. }),
        Effect::Seq(v) => v.last().is_some_and(counters_into_exile),
        _ => false,
    }
}

fn may_cast_countered_card(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let lands = match end(l) {
        "you may cast that card without paying its mana cost for as long as it remains exiled"
        | "you may cast it without paying its mana cost for as long as it remains exiled" => {
            false
        }
        "you may play it without paying its mana cost for as long as it remains exiled"
        | "you may play that card without paying its mana cost for as long as it remains exiled" => {
            true
        }
        _ => return false,
    };
    if !counters_into_exile(prev) {
        return false;
    }
    let card = Sel::Var(CARD);
    // The countered card, if it's in exile (a spell that wasn't exiled — e.g. not a
    // permanent spell — gets nothing).
    let mut filter = vec![Filter::InZone(ZoneKind::Exile)];
    if !lands {
        filter.push(Filter::Not(Box::new(Filter::Type(CardType::Land))));
    }
    let grant = Effect::GrantPlayPermission {
        who: PlayerRef::You,
        what: card.clone(),
        // The permission is for that object: it ends when the card leaves exile.
        duration: Duration::Permanent,
        free: true,
    };
    // "You may cast": not to play a land (CR 305.9).
    let grant = if lands { grant } else { grant.cast_only() };
    let each = Effect::ForEach {
        sel: Sel::Var(vars::IT),
        var: CARD,
        effect: Box::new(Effect::If {
            cond: Condition::SelMatches(card, Filter::and(filter)),
            then: Box::new(grant),
            otherwise: Box::new(Effect::Noop),
        }),
    };
    *prev = Effect::seq(vec![std::mem::take(prev), each]);
    true
}

/// The variable iterating over the countered cards.
const CARD: Var = vars::USER + 1773;

inventory::submit! { FollowupPattern { name: "counter, exile instead: you may cast that card without paying its mana cost while exiled", priority: 45, apply: may_cast_countered_card } }
