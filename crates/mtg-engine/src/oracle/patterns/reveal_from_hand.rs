//! "Reveal three cards from your hand", "reveal X cards from your hand", "reveal a number
//! of cards from your hand equal to ..." — usually performed by another player ("Target
//! player reveals three cards from their hand", see `player_subjects`): that player
//! chooses the cards (CR 701.20a; see `kw/reveal_from_hand.rs`). Then "You choose one of
//! them." / "you choose two of those cards": the ability's controller chooses among the
//! revealed cards, which "that player discards that card" (`card_flow_reveal_hand`) and
//! "that player exiles it" then refer to.

use super::card_flow_reveal_hand::CHOSEN;
use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::kw::reveal_from_hand::{REVEALED, REVEAL_CHOSEN};
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use smol_str::SmolStr;

/// "reveal N cards from your hand" / "reveal a number of cards from your hand equal to V".
fn reveal_from_hand(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("reveal ")?;
    let n = if let Some(r) = r.strip_prefix("a number of cards from your hand equal to ") {
        let (v, tail) = crate::oracle::statics::parse_value_phrase(r, b)?;
        if !end(&tail).is_empty() {
            return None;
        }
        v
    } else {
        let (n, rest) = parse_card_count(r)?;
        if end(rest) != "from your hand" || matches!(n, Value::Const(1)) && !r.starts_with("a ") {
            return None;
        }
        // "reveal a card from your hand" alone isn't this (it's usually a cost or a
        // choice with a follow-up the patterns read elsewhere).
        if matches!(n, Value::Const(1)) {
            return None;
        }
        n
    };
    b.named.push((REVEALED_NAME.to_string(), Sel::Var(REVEALED)));
    Some(Effect::seq(vec![
        Effect::Store {
            var: REVEALED,
            sel: Sel::Choose {
                chooser: PlayerRef::You,
                filter: Filter::and(vec![
                    Filter::InZone(ZoneKind::Hand),
                    Filter::OwnedBy(PlayerRel::You),
                ]),
                count: n,
                up_to: false,
                store: None,
            },
        },
        Effect::Custom(SmolStr::new(REVEAL_CHOSEN)),
    ]))
}

inventory::submit! { EffectPattern { name: "reveal N cards from your hand", priority: 120, parse: reveal_from_hand } }

/// Marks, in [`Builder::named`], that the text had a player reveal cards from their hand.
const REVEALED_NAME: &str = "\u{1}the cards revealed from a hand";

fn revealed_earlier(b: &Builder) -> bool {
    b.named.iter().any(|(p, _)| p == REVEALED_NAME)
}

/// The chooser's "you choose one of them" / "you choose two of those cards": the choice.
fn choose_among_revealed(r: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(r).strip_prefix("you choose ")?;
    let (n, rest) = parse_number(r)?;
    n.as_const()?;
    if !matches!(rest.trim(), "of them" | "of those cards") {
        return None;
    }
    b.it = Sel::Var(CHOSEN);
    Some(Effect::Store {
        var: CHOSEN,
        sel: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::In(Box::new(Sel::Var(REVEALED))),
            count: n,
            up_to: false,
            store: None,
        },
    })
}

/// "You choose one of them." after a player revealed cards from their hand.
fn you_choose_of_them(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !revealed_earlier(b) {
        return false;
    }
    let Some(e) = choose_among_revealed(l, b) else {
        return false;
    };
    let p = std::mem::take(prev);
    *prev = Effect::seq(vec![p, e]);
    true
}

inventory::submit! { FollowupPattern { name: "you choose one of them (revealed from hand)", priority: 90, apply: you_choose_of_them } }

/// "[player] reveals three cards from their hand and you choose one of them" (one
/// sentence).
fn reveal_and_you_choose(l: &str, b: &mut Builder) -> Option<Effect> {
    let (first, choose) = end(l).rsplit_once(" and you choose ")?;
    let saved = (b.targets.len(), b.it_player.clone());
    let Some(e) = crate::oracle::effects::parse_clause(first, b) else {
        b.targets.truncate(saved.0);
        b.it_player = saved.1;
        return None;
    };
    if !revealed_earlier(b) {
        return None;
    }
    let c = choose_among_revealed(&format!("you choose {choose}"), b)?;
    Some(Effect::seq(vec![e, c]))
}

inventory::submit! { EffectPattern { name: "[player] reveals N cards from their hand and you choose one of them", priority: 85, parse: reveal_and_you_choose } }
