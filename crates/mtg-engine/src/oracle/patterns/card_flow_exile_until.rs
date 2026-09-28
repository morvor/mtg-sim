//! Exiling cards from the top of a library until a card with some quality is exiled:
//! "Target opponent exiles cards from the top of their library until they exile a nonland
//! card." (Nicol Bolas, God-Pharaoh), "Exile cards from the top of your library until you
//! exile a nonland card." Every card exiled this way stays in exile; "that card" is the
//! last one (`vars::IT`). Followed by a permission to cast it: "Until end of turn, you may
//! cast that card without paying its mana cost." (CR 118.9). The permission is for that
//! object in exile (CR 400.7) and only lets it be cast, never played as a land (CR 305.9).

use super::card_flow_search::card_filter;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::{EffectPattern, FollowupPattern};
use crate::oracle::phrases::end;
use crate::types::CardType;

inventory::submit! {
    EffectPattern { name: "card_flow: exile cards from the top of a library until exiling a [card]", priority: 90, parse: exile_until }
}
inventory::submit! {
    FollowupPattern { name: "card_flow: until end of turn, you may cast that card [without paying its mana cost]", priority: 90, apply: may_cast_that_card }
}

/// "exile cards from the top of your library until you exile a nonland card", "target
/// opponent exiles cards from the top of their library until they exile a nonland card",
/// "that player exiles ...".
fn exile_until(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    const YOU: &str = "exile cards from the top of your library until you exile ";
    const THEY: &str = "exiles cards from the top of their library until they exile ";
    let (who, r) = if let Some(r) = l.strip_prefix(YOU) {
        (PlayerRef::You, r)
    } else if let Some(r) = l.strip_prefix("that player ").and_then(|r| r.strip_prefix(THEY)) {
        match b.it_player {
            PlayerRef::Target(_) | PlayerRef::TriggerPlayer | PlayerRef::DefendingPlayer => {
                (b.it_player.clone(), r)
            }
            _ => return None,
        }
    } else {
        let (pf, text, r) = if let Some(r) = l.strip_prefix("target opponent ") {
            (PlayerFilter::Opponent, "target opponent", r)
        } else if let Some(r) = l.strip_prefix("target player ") {
            (PlayerFilter::Any, "target player", r)
        } else {
            return None;
        };
        let r = r.strip_prefix(THEY)?;
        let desc = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
        let filter = card_filter(desc, b)?;
        let slot = b.add_target(TargetSpec::player(pf, text), text);
        b.it_player = PlayerRef::Target(slot);
        b.it = Sel::Var(vars::IT);
        return Some(effect(PlayerRef::Target(slot), filter));
    };
    let desc = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let filter = card_filter(desc, b)?;
    b.it = Sel::Var(vars::IT);
    Some(effect(who, filter))
}

fn effect(who: PlayerRef, filter: Filter) -> Effect {
    Effect::RevealUntil {
        who,
        filter: Filter::And(vec![Filter::Card, filter]),
        found_to: Destination::zone(ZoneKind::Exile),
        rest_to: Destination::zone(ZoneKind::Exile),
    }
}

/// Whether the effect ends by exiling cards until one was found.
fn ends_with_exile_until(e: &Effect) -> bool {
    match e {
        Effect::RevealUntil { found_to, .. } => found_to.zone == ZoneKind::Exile,
        Effect::Seq(v) => v.last().is_some_and(ends_with_exile_until),
        _ => false,
    }
}

/// "Until end of turn, you may cast that card [without paying its mana cost]." after
/// exiling cards until one was found.
fn may_cast_that_card(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let free = match end(l) {
        "until end of turn, you may cast that card without paying its mana cost" => true,
        "until end of turn, you may cast that card" => false,
        _ => return false,
    };
    if !ends_with_exile_until(prev) {
        return false;
    }
    let card = Sel::Var(vars::IT);
    let grant = Effect::If {
        cond: Condition::SelMatches(
            card.clone(),
            Filter::Not(Box::new(Filter::Type(CardType::Land))),
        ),
        then: Box::new(Effect::GrantPlayPermission {
            who: PlayerRef::You,
            what: card,
            duration: Duration::EndOfTurn,
            free,
        }),
        otherwise: Box::new(Effect::Noop),
    };
    *prev = Effect::seq(vec![std::mem::take(prev), grant]);
    true
}
