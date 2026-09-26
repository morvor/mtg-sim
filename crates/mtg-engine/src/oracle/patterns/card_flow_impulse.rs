//! Exiling the top cards of a library, and "impulse draw" permissions to play them:
//! "Exile the top card of your library. You may play that card this turn.", "Exile the
//! top two cards of your library. Until the end of your next turn, you may play those
//! cards.", "Exile the top three cards of target opponent's library."

use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::{EffectPattern, FollowupPattern};
use crate::oracle::phrases::*;

inventory::submit! {
    EffectPattern { name: "card_flow: exile the top N cards of a library", priority: 90, parse: exile_top }
}
inventory::submit! {
    FollowupPattern { name: "card_flow: you may play those cards this turn", priority: 90, apply: may_play_them }
}

/// "the top card of", "the top N cards of" + a library.
fn top_cards_of<'a>(s: &'a str) -> Option<(Value, &'a str)> {
    let r = s.strip_prefix("the top ")?;
    if let Some(r) = r.strip_prefix("card of ") {
        return Some((Value::c(1), r));
    }
    let (n, r) = parse_number(r)?;
    n.as_const()?;
    Some((n, r.strip_prefix("cards of ")?))
}

/// "exile the top N cards of your library", "... of target opponent's library", "... of
/// each opponent's library", "... of each player's library".
fn exile_top(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("exile ")?;
    let (n, r) = top_cards_of(r)?;
    let who = match r {
        "your library" => PlayerRef::You,
        "each opponent's library" => PlayerRef::EachOpponent,
        "each player's library" => PlayerRef::EachPlayer,
        "target opponent's library" | "target player's library" => {
            let (pf, text) = if r.starts_with("target opponent") {
                (PlayerFilter::Opponent, "target opponent")
            } else {
                (PlayerFilter::Any, "target player")
            };
            let slot = b.add_target(TargetSpec::player(pf, text), text);
            b.it_player = PlayerRef::Target(slot);
            PlayerRef::Target(slot)
        }
        _ => return None,
    };
    b.it = Sel::Var(vars::IT);
    Some(Effect::Exile {
        what: Sel::TopOfLibrary(who, n),
        face_down: false,
        link: false,
    })
}

/// Whether the effect ends by exiling the top cards of your own library.
fn exiles_your_top_cards(e: &Effect) -> bool {
    match e {
        Effect::Exile {
            what: Sel::TopOfLibrary(PlayerRef::You, _),
            face_down: false,
            ..
        } => true,
        Effect::Seq(v) => v.last().is_some_and(exiles_your_top_cards),
        _ => false,
    }
}

/// "you may play that card this turn", "until end of turn, you may play those cards",
/// "until the end of your next turn, you may play that card" after exiling the top cards
/// of your library.
fn may_play_them(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let (duration, r) = if let Some(r) = l.strip_prefix("until the end of your next turn, ") {
        (Some(Duration::UntilEndOfYourNextTurn), r)
    } else if let Some(r) = l.strip_prefix("until end of turn, ") {
        (Some(Duration::EndOfTurn), r)
    } else {
        (None, l)
    };
    let Some(r) = r.strip_prefix("you may play ") else {
        return false;
    };
    let Some(r) = ["that card", "those cards", "them", "it", "the exiled card", "the exiled cards"]
        .iter()
        .find_map(|p| {
            r.strip_prefix(p)
                .filter(|x| x.is_empty() || x.starts_with(' '))
        })
    else {
        return false;
    };
    let duration = match (duration, r.trim()) {
        (Some(d), "") => d,
        (None, "this turn") => Duration::EndOfTurn,
        (None, "until the end of your next turn") => Duration::UntilEndOfYourNextTurn,
        _ => return false,
    };
    if !exiles_your_top_cards(prev) {
        return false;
    }
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::GrantPlayPermission {
            who: PlayerRef::You,
            what: Sel::Var(vars::IT),
            duration,
            free: false,
        },
    ]);
    true
}
