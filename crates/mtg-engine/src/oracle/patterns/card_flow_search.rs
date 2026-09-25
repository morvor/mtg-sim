//! Library searches (CR 701.23): "search your library for [cards], [reveal them,] put
//! them [into your hand | onto the battlefield (tapped) | into your graveyard], then
//! shuffle", "exile them", "then shuffle and put that card on top", searches by another
//! player ("its controller may search their library for a basic land card, put it onto
//! the battlefield tapped, then shuffle"), and searches of another player's library
//! ("search target opponent's library for ... Then that player shuffles.").

use crate::ability::*;
use crate::oracle::effects::{player_ref, Builder};
use crate::oracle::patterns::{EffectPattern, FollowupPattern};
use crate::oracle::phrases::*;

inventory::submit! {
    EffectPattern { name: "card_flow: search a library", priority: 90, parse: search_library }
}
inventory::submit! {
    FollowupPattern { name: "card_flow: then that player shuffles", priority: 100, apply: that_player_shuffles }
}

/// Who searches, whose library, and whether it's optional ("may search").
struct Searcher {
    who: PlayerRef,
    whose: PlayerRef,
    may: bool,
}

fn searcher(l: &str, b: &mut Builder) -> Option<(Searcher, String)> {
    for p in ["search your library for ", "you search your library for "] {
        if let Some(r) = l.strip_prefix(p) {
            let s = Searcher {
                who: PlayerRef::You,
                whose: PlayerRef::You,
                may: false,
            };
            return Some((s, r.to_string()));
        }
    }
    // "search target opponent's library for", "search target player's library for".
    if let Some(r) = l.strip_prefix("search ") {
        for (p, pf, text) in [
            (
                "target opponent's library for ",
                PlayerFilter::Opponent,
                "target opponent",
            ),
            (
                "target player's library for ",
                PlayerFilter::Any,
                "target player",
            ),
        ] {
            if let Some(r) = r.strip_prefix(p) {
                let slot = b.add_target(TargetSpec::player(pf, text), text);
                b.it_player = PlayerRef::Target(slot);
                let s = Searcher {
                    who: PlayerRef::You,
                    whose: PlayerRef::Target(slot),
                    may: false,
                };
                return Some((s, r.to_string()));
            }
        }
        return None;
    }
    // "[player] may search their library for", "[player] searches their library for".
    let (who, rest) = player_ref(l, b)?;
    let rest = rest.trim_start();
    let (may, r) = if let Some(r) = rest.strip_prefix("may search their library for ") {
        (true, r)
    } else if let Some(r) = rest.strip_prefix("searches their library for ") {
        (false, r)
    } else if let Some(r) = rest.strip_prefix("may search your library for ") {
        // "you may search your library for" (when "you" is parsed as a player phrase).
        (true, r)
    } else {
        return None;
    };
    if matches!(
        who,
        PlayerRef::EachPlayer | PlayerRef::EachOpponent | PlayerRef::EachOtherPlayer
    ) {
        // Each player searching (and shuffling once each) isn't expressed here.
        return None;
    }
    let s = Searcher {
        whose: who.clone(),
        who,
        may,
    };
    Some((s, r.to_string()))
}

/// "a", "an", "up to N", "any number of", "N" before the card description.
fn search_count(s: &str) -> Option<(Value, &str)> {
    if let Some(r) = s.strip_prefix("any number of ") {
        return Some((Value::c(999), r));
    }
    if let Some(r) = s.strip_prefix("up to ") {
        let (n, r) = parse_number(r)?;
        return Some((n, r));
    }
    let (n, r) = parse_number(s)?;
    // "X" alone would need the value of X from the spell; keep that to constants.
    n.as_const()?;
    Some((n, r))
}

/// The description of the cards searched for: "basic land card", "Mercenary permanent
/// card with mana value 3 or less", "card named ~", "instant card or a card with flash",
/// "basic land cards and/or Gate cards".
fn card_filter(s: &str, b: &Builder) -> Option<Filter> {
    let s = s.trim();
    // Alternatives spelled out with their own articles: "an instant card or a card with
    // flash", "a basic land card or a Desert card", "basic land cards and/or Gate cards".
    for sep in [" or a ", " or an ", " and/or "] {
        if let Some((a, c)) = s.split_once(sep) {
            let a_card = a.ends_with(" card") || a.ends_with(" cards");
            if a_card {
                let fa = card_filter(a, b)?;
                let fc = card_filter(c, b)?;
                return Some(Filter::Or(vec![fa, fc]));
            }
        }
    }
    // "card named ~", "cards named ~".
    for p in ["card named ~", "cards named ~"] {
        if s == p {
            let name = if b.ctx.card_name.is_empty() {
                return None;
            } else {
                b.ctx.card_name
            };
            return Some(Filter::and(vec![Filter::Card, Filter::Named(name.into())]));
        }
    }
    let (f, _, rest) = parse_object_phrase(s)?;
    if !rest.trim().is_empty() {
        return None;
    }
    // The description must name cards ("basic land card", "creature cards").
    if !(s.contains("card")) {
        return None;
    }
    Some(f)
}

const PRONOUNS: [&str; 6] = [
    "it",
    "them",
    "that card",
    "those cards",
    "the card",
    "the cards",
];

fn strip_pronoun(s: &str) -> Option<&str> {
    PRONOUNS.iter().find_map(|p| {
        let r = s.strip_prefix(p)?;
        (r.is_empty() || r.starts_with(' ') || r.starts_with(',')).then_some(r)
    })
}

/// Where the found cards go.
fn destination(s: &str, searcher: &PlayerRef) -> Option<Destination> {
    let (s, tapped) = match s.strip_suffix(" tapped") {
        Some(x) => (x, true),
        None => match s.strip_prefix("onto the battlefield tapped") {
            Some(r) => {
                return battlefield(r, true, searcher);
            }
            None => (s, false),
        },
    };
    if let Some(r) = s.strip_prefix("onto the battlefield") {
        return battlefield(r, tapped, searcher);
    }
    if tapped {
        return None;
    }
    Some(match s {
        "into your hand" | "into their hand" | "into that player's hand" => {
            Destination::zone(ZoneKind::Hand)
        }
        "into your graveyard" | "into their graveyard" | "into that player's graveyard" => {
            Destination::zone(ZoneKind::Graveyard)
        }
        _ => return None,
    })
}

fn battlefield(r: &str, tapped: bool, searcher: &PlayerRef) -> Option<Destination> {
    let mut d = Destination::battlefield();
    d.tapped = tapped;
    match r.trim() {
        // CR 110.2a: a permanent enters under the control of the player who put it there.
        "" | "under their control" => d.controller = Some(searcher.clone()),
        "under your control" => d.controller = Some(PlayerRef::You),
        _ => return None,
    }
    Some(d)
}

fn search_library(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (s, r) = searcher(l, b)?;
    let (count, r) = search_count(&r)?;
    // The card description runs to the first action.
    const ACTIONS: [&str; 8] = [
        ", reveal ",
        " and reveal ",
        ", put ",
        " and put ",
        ", exile ",
        " and exile ",
        ", then shuffle and put ",
        ", then shuffle",
    ];
    let cut = ACTIONS.iter().filter_map(|a| r.find(a)).min()?;
    let filter = card_filter(&r[..cut], b)?;
    let mut t = &r[cut..];
    // Optional reveal.
    for p in [", reveal ", " and reveal "] {
        if let Some(x) = t.strip_prefix(p) {
            t = strip_pronoun(x)?;
            break;
        }
    }
    let (to, shuffle) = if let Some(x) = t
        .strip_prefix(", then shuffle and put that card on top")
        .or_else(|| t.strip_prefix(" then shuffle and put that card on top"))
    {
        if !x.is_empty() {
            return None;
        }
        // CR 701.23: the card is put on top after the library is shuffled.
        (Destination::library_top(), true)
    } else {
        let x = [", and put ", ", put ", " and put ", " put "]
            .iter()
            .find_map(|p| t.strip_prefix(p).map(|r| (r, true)))
            .or_else(|| {
                [", and exile ", ", exile ", " and exile "]
                    .iter()
                    .find_map(|p| t.strip_prefix(p).map(|r| (r, false)))
            });
        let (x, put) = x?;
        let x = strip_pronoun(x)?.trim_start();
        let (dest, shuffle) = match x
            .strip_suffix(", then shuffle")
            .or_else(|| x.strip_suffix(" then shuffle"))
        {
            Some(d) => (d, true),
            None => (x, false),
        };
        let to = if put {
            destination(dest.trim(), &s.who)?
        } else if dest.trim().is_empty() {
            Destination::zone(ZoneKind::Exile)
        } else {
            return None;
        };
        (to, shuffle)
    };
    // Searching another player's library shuffles it only with "then that player
    // shuffles" (a follow-up sentence), and "then shuffle" names the searcher's own.
    if shuffle && !matches!(s.whose, PlayerRef::You) && matches!(s.who, PlayerRef::You) {
        return None;
    }
    let search = Effect::Search {
        who: s.who.clone(),
        whose: s.whose.clone(),
        filter: Filter::and(vec![filter, Filter::InZone(ZoneKind::Library)]),
        count,
        to,
        reveal: true,
        shuffle,
    };
    b.it = Sel::Var(vars::IT);
    Some(if s.may {
        Effect::May {
            who: s.who,
            effect: Box::new(search),
        }
    } else {
        search
    })
}

/// "Then that player shuffles." after a search of another player's library.
fn that_player_shuffles(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if !matches!(l, "then that player shuffles" | "that player shuffles") {
        return false;
    }
    match prev {
        Effect::Search {
            whose: PlayerRef::Target(_),
            shuffle,
            ..
        } if !*shuffle => {
            *shuffle = true;
            true
        }
        _ => false,
    }
}
