//! Cards from outside the game (CR 400.11b, 108.3b), beyond the wishes of
//! `r108_cards.rs`: "reveal an artifact card you own from outside the game or choose a
//! face-up artifact card you own in exile. Put that card into your hand." (Karn, the
//! Great Creator; Coax from the Blind Eternities), "Shuffle up to four cards you own from
//! outside the game into your library." (Research), "put a card you own from outside the
//! game on top of your library" (The Raven's Warning).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::kw::reveal_from_hand::{REVEALED, REVEAL_CHOSEN};
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// "a [filter] card you own" (singular) and the rest.
fn owned_card(s: &str) -> Option<(Filter, &str)> {
    let r = s.strip_prefix("a ").or_else(|| s.strip_prefix("an "))?;
    let (f, false, rest) = parse_object_phrase(r)? else {
        return None;
    };
    Some((f, rest.trim_start()))
}

fn outside(f: Filter) -> Filter {
    Filter::and(vec![
        f,
        Filter::Card,
        Filter::InZone(ZoneKind::Outside),
        Filter::OwnedBy(PlayerRel::You),
    ])
}

/// "reveal an artifact card you own from outside the game or choose a face-up artifact
/// card you own in exile": one card from either place, chosen and (if it's from outside
/// the game) revealed; "that card" is it.
fn reveal_outside_or_exiled(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("reveal ")?;
    let (f, rest) = owned_card(r)?;
    let rest = rest.strip_prefix("from outside the game or choose a face-up ")?;
    let (g, rest) = owned_card(&format!("a {rest}"))
        .map(|(g, rest)| (g, rest.to_string()))?;
    // ("in exile" may be read as part of the phrase.)
    if !(rest == "in exile" || (rest.is_empty() && g.zone() == Some(ZoneKind::Exile))) {
        return None;
    }
    let exiled = Filter::and(vec![
        g,
        Filter::Card,
        Filter::not(Filter::FaceDown),
        Filter::InZone(ZoneKind::Exile),
        Filter::OwnedBy(PlayerRel::You),
    ]);
    b.it = Sel::Var(REVEALED);
    Some(Effect::seq(vec![
        Effect::Store {
            var: REVEALED,
            sel: Sel::Choose {
                chooser: PlayerRef::You,
                filter: Filter::Or(vec![outside(f), exiled]),
                count: Value::c(1),
                up_to: false,
                store: None,
            },
        },
        Effect::Custom(REVEAL_CHOSEN.into()),
    ]))
}

inventory::submit! { EffectPattern { name: "game rules: reveal a card from outside the game or choose a face-up exiled card", priority: 100, parse: reveal_outside_or_exiled } }

/// "Put that card into your hand." after [`reveal_outside_or_exiled`].
fn put_that_card_into_hand(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    fn ends_with_reveal(e: &Effect) -> bool {
        match e {
            Effect::Seq(v) => v.last().is_some_and(ends_with_reveal),
            Effect::May { effect, .. } => ends_with_reveal(effect),
            Effect::Custom(n) => n == REVEAL_CHOSEN,
            _ => false,
        }
    }
    if end(l) != "put that card into your hand" || !ends_with_reveal(prev) {
        return false;
    }
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![
        old,
        Effect::Move {
            what: Sel::Var(REVEALED),
            to: Destination::zone(ZoneKind::Hand),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "game rules: put that card into your hand (from outside the game or exile)", priority: 100, apply: put_that_card_into_hand } }

/// "Shuffle up to four cards you own from outside the game into your library.", "put a
/// card you own from outside the game on top of your library".
fn outside_to_library(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if let Some(r) = l.strip_prefix("shuffle up to ") {
        let (n, r) = parse_number(r)?;
        n.as_const()?;
        let r = r
            .trim_start()
            .strip_suffix(" you own from outside the game into your library")?;
        let (f, true, rest) = parse_object_phrase(r)? else {
            return None;
        };
        if !end(rest).is_empty() {
            return None;
        }
        return Some(Effect::ShuffleIntoLibrary {
            what: Sel::Choose {
                chooser: PlayerRef::You,
                filter: outside(f),
                count: n,
                up_to: true,
                store: None,
            },
            library: PlayerRef::You,
        });
    }
    let r = l.strip_prefix("put ")?;
    let (f, rest) = owned_card(r)?;
    if rest != "from outside the game on top of your library" {
        return None;
    }
    Some(Effect::Move {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: outside(f),
            count: Value::c(1),
            up_to: false,
            store: None,
        },
        to: Destination::library_top(),
    })
}

inventory::submit! { EffectPattern { name: "game rules: cards from outside the game into your library", priority: 100, parse: outside_to_library } }
