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
inventory::submit! {
    FollowupPattern { name: "card_flow: you may cast that card without paying its mana cost (now)", priority: 90, apply: may_cast_found_now }
}
inventory::submit! {
    FollowupPattern { name: "card_flow: put the exiled cards that weren't cast on the bottom in a random order", priority: 90, apply: rest_to_bottom }
}

/// The card found, remembered before it may be cast ([`may_cast_found_now`]).
const FOUND: Var = vars::USER + 3132;

/// "You may cast that card without paying its mana cost." right after exiling cards until
/// one was found (Chaos Wand): it's cast as the effect resolves (CR 608.2g, 118.9; X is
/// 0, CR 107.3b).
fn may_cast_found_now(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if !matches!(
        end(l),
        "you may cast that card without paying its mana cost"
            | "you may cast it without paying its mana cost"
    ) || !matches!(prev, Effect::RevealUntil { .. })
    {
        return false;
    }
    *prev = Effect::seq(vec![
        std::mem::take(prev),
        Effect::Store {
            var: FOUND,
            sel: Sel::Var(vars::IT),
        },
        Effect::CastCard {
            who: PlayerRef::You,
            what: Sel::Var(vars::IT),
            free: true,
            optional: true,
        },
    ]);
    true
}

/// "Then put the exiled cards that weren't cast this way on the bottom of that library in a
/// random order." after [`may_cast_found_now`]: the card found if it wasn't cast, and the
/// other cards exiled, still in exile.
fn rest_to_bottom(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    if !matches!(
        l,
        "put the exiled cards that weren't cast this way on the bottom of that library in a random order"
            | "put the exiled cards that weren't cast this way on the bottom of their owner's library in a random order"
    ) {
        return false;
    }
    let remembers_found = match &*prev {
        Effect::Seq(v) => v
            .iter()
            .any(|e| matches!(e, Effect::Store { var, .. } if *var == FOUND)),
        _ => false,
    };
    if !remembers_found {
        return false;
    }
    let mut to = Destination::zone(ZoneKind::Library);
    to.position = LibraryPosition::BottomRandom;
    let rest = Sel::All(Filter::and(vec![
        Filter::In(Box::new(Sel::Union(vec![
            Sel::Var(FOUND),
            Sel::Var(vars::REVEALED),
        ]))),
        Filter::InZone(ZoneKind::Exile),
    ]));
    *prev = Effect::seq(vec![std::mem::take(prev), Effect::Move { what: rest, to }]);
    true
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
    // A quality relative to the trigger's spell ("with lesser mana value", Jodah, the
    // Unifier) is `r406_exile_until.rs`'s, with its own follow-up instructions.
    if super::filters_relational::mentions_referent(&filter) {
        return None;
    }
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

/// Whether the effect ends by exiling cards until one was found, or by exiling a card
/// from among the top cards looked at ("You may exile a legendary creature card from
/// among them.", Djeru and Hazoret).
fn ends_with_exile_until(e: &Effect) -> bool {
    match e {
        Effect::RevealUntil { found_to, .. } => found_to.zone == ZoneKind::Exile,
        Effect::Dig { take_to, .. } => take_to.zone == ZoneKind::Exile,
        Effect::Seq(v) => v.last().is_some_and(ends_with_exile_until),
        _ => false,
    }
}

/// "Until end of turn, you may cast that card [without paying its mana cost]." after
/// exiling cards until one was found.
fn may_cast_that_card(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let free = match end(l) {
        "until end of turn, you may cast that card without paying its mana cost"
        | "until end of turn, you may cast the exiled card without paying its mana cost" => {
            true
        }
        "until end of turn, you may cast that card" | "until end of turn, you may cast the exiled card" => {
            false
        }
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
