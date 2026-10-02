//! Exiling a card and letting its owner play it, sometimes for a price:
//!
//! * "Look at target opponent's hand. You may exile a nonland card from it." (Elite
//!   Spellbinder): the card is chosen from the hand looked at, as from a revealed hand
//!   (see `card_flow_reveal_hand`).
//! * "For as long as that card remains exiled, its owner may play it.", "For each of those
//!   cards, its owner may play it for as long as it remains exiled." after exiling cards
//!   face up: a permission for each exiled card (a new object, CR 400.7) given to its
//!   owner, which ends when the card leaves exile. The owner plays it with the normal
//!   timing rules (CR 305.1, 307.1) and costs (CR 601.2f); the back face of a modal
//!   double-faced card may be played as a land (CR 712.11b).
//! * "Each opponent exiles a card from their hand and may play that card for as long as it
//!   remains exiled." (Lightstall Inquisitor): each opponent's own permission for the card
//!   they exiled.
//! * "A spell cast this way costs {2} more to cast.", "A spell cast by an opponent this way
//!   costs {2} more to cast.", "Each land played this way enters tapped.": terms of those
//!   permissions (see `kw/play_permission_terms.rs`).

use super::card_flow_reveal_hand::choose_from_it;
use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::kw::play_permission_terms::{parse_terms, terms_effect, Terms};
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

inventory::submit! {
    FollowupPattern { name: "card_flow: you may exile a [card] from it (a hand looked at)", priority: 85, apply: exile_from_hand_looked_at }
}
inventory::submit! {
    FollowupPattern { name: "card_flow: for as long as that card remains exiled, its owner may play it", priority: 85, apply: owner_may_play }
}
inventory::submit! {
    EffectPattern { name: "card_flow: each opponent exiles a card from their hand and may play that card for as long as it remains exiled", priority: 85, parse: each_exiles_and_may_play }
}
inventory::submit! {
    FollowupPattern { name: "card_flow: a spell cast this way costs {N} more to cast", priority: 85, apply: permission_terms }
}

/// The variable iterating over the exiled cards.
const CARD: Var = vars::USER + 1791;

/// "you may exile a nonland card from it", "exile a card from it" after looking at (or
/// revealing) a player's hand: the card is chosen from that hand and exiled.
fn exile_from_hand_looked_at(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let (may, r) = match l.strip_prefix("you may exile ") {
        Some(r) => (true, r),
        None => match l.strip_prefix("exile ") {
            Some(r) => (false, r),
            None => return false,
        },
    };
    // Only "a [card] from it" (a single card).
    let Some(desc) = r.strip_suffix(" from it") else {
        return false;
    };
    if !(desc.starts_with("a ") || desc.starts_with("an ")) {
        return false;
    }
    let chooser = if may { "you may choose" } else { "you choose" };
    let saved = (prev.clone(), b.it.clone(), b.it_player.clone());
    if !choose_from_it(
        &format!("{chooser} {desc} from it and exile that card"),
        prev,
        b,
    ) {
        (*prev, b.it, b.it_player) = saved;
        return false;
    }
    // "that card" now names the exiled card (CR 400.7).
    b.it = Sel::Var(vars::IT);
    true
}

/// Whether the effect ends by exiling cards face up.
fn ends_with_exile(e: &Effect) -> bool {
    match e {
        Effect::Exile {
            face_down: false, ..
        } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_with_exile),
        _ => false,
    }
}

/// "For as long as that card remains exiled, its owner may play it." after exiling cards
/// face up: each card's owner may play it while it remains exiled.
fn owner_may_play(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if !matches!(
        end(l),
        "for as long as that card remains exiled, its owner may play it"
            | "its owner may play it for as long as it remains exiled"
            | "for each of those cards, its owner may play it for as long as it remains exiled"
    ) {
        return false;
    }
    if !ends_with_exile(prev) {
        return false;
    }
    let card = Sel::Var(CARD);
    let each = Effect::ForEach {
        sel: Sel::Var(vars::IT),
        var: CARD,
        effect: Box::new(Effect::GrantPlayPermission {
            who: PlayerRef::OwnerOf(Box::new(card.clone())),
            what: card,
            // The permission is for that object: it ends when the card leaves exile.
            duration: Duration::Permanent,
            free: false,
        }),
    };
    *prev = Effect::seq(vec![std::mem::take(prev), each]);
    true
}

/// "each opponent exiles a card from their hand and may play that card for as long as it
/// remains exiled": each of them exiles a card and may play it.
fn each_exiles_and_may_play(l: &str, b: &mut Builder) -> Option<Effect> {
    let exile = end(l).strip_suffix(" and may play that card for as long as it remains exiled")?;
    if !exile.ends_with(" exiles a card from their hand") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let parsed = parse_clause(exile, b);
    let Some(Effect::ForEachPlayer { who, effect }) = parsed else {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    };
    if !matches!(*effect, Effect::Exile { face_down: false, .. }) {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    }
    let grant = Effect::GrantPlayPermission {
        who: PlayerRef::Iterated,
        what: Sel::Var(vars::IT),
        duration: Duration::Permanent,
        free: false,
    };
    Some(Effect::ForEachPlayer {
        who,
        effect: Box::new(Effect::seq(vec![*effect, grant])),
    })
}

/// "{N}" (generic mana).
fn generic(s: &str) -> Option<(u32, &str)> {
    let r = s.strip_prefix('{')?;
    let (n, r) = r.split_once('}')?;
    Some((n.parse().ok()?, r))
}

/// "a spell cast this way costs {2} more to cast", "each spell cast this way costs {1}
/// more to cast", "a spell cast by an opponent this way costs {2} more to cast", "each
/// land played this way enters tapped".
fn permission_terms(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l);
    let mut t = Terms::default();
    if l == "each land played this way enters tapped" {
        t.lands_enter_tapped = true;
    } else {
        let r = if let Some(r) = l
            .strip_prefix("a spell cast this way costs ")
            .or_else(|| l.strip_prefix("each spell cast this way costs "))
            .or_else(|| l.strip_prefix("spells cast this way cost "))
        {
            r
        } else if let Some(r) = l.strip_prefix("a spell cast by an opponent this way costs ") {
            t.opponents_only = true;
            r
        } else {
            return false;
        };
        let Some((n, " more to cast")) = generic(r) else {
            return false;
        };
        t.cost_increase = n;
    }
    add_after_grant(prev, t)
}

/// Puts the terms right after the permission the effect ends by giving, where the same
/// variable names the cards.
fn add_after_grant(e: &mut Effect, t: Terms) -> bool {
    match e {
        Effect::GrantPlayPermission {
            what: Sel::Var(v),
            ..
        } => {
            let terms = terms_effect(*v, t);
            *e = Effect::Seq(vec![std::mem::take(e), terms]);
            true
        }
        // (After the terms an earlier sentence gave.)
        Effect::Seq(v) => v
            .iter_mut()
            .rev()
            .find(|x| !matches!(x, Effect::Custom(n) if parse_terms(n).is_some()))
            .is_some_and(|x| add_after_grant(x, t)),
        Effect::ForEach { effect, .. } | Effect::ForEachPlayer { effect, .. } => {
            add_after_grant(effect, t)
        }
        _ => false,
    }
}
