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
inventory::submit! {
    FollowupPattern { name: "card_flow: if it's a nonland card, you may cast that card this turn", priority: 90, apply: may_cast_if_nonland }
}
inventory::submit! {
    FollowupPattern { name: "card_flow: you may cast that card (now)", priority: 90, apply: may_cast_it_now }
}
inventory::submit! {
    EffectPattern { name: "card_flow: you may play that card until end of turn (the card just moved)", priority: 90, parse: may_play_that_card }
}

/// "the top card of", "the top N cards of" + a library.
fn top_cards_of<'a>(s: &'a str) -> Option<(Value, &'a str)> {
    let r = s.strip_prefix("the top ")?;
    if let Some(r) = r.strip_prefix("card of ") {
        return Some((Value::c(1), r));
    }
    let (n, r) = parse_number(r)?;
    // "the top X cards", X being chosen as the spell is cast (Monastery Raid's freerunning
    // cost {X}{R}, CR 107.3).
    if !matches!(n, Value::X) {
        n.as_const()?;
    }
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
        // "exile the top card of that player's library" / "of their library": the player
        // the text is about (e.g. the player a creature dealt combat damage to).
        "that player's library" | "their library"
            if matches!(
                b.it_player,
                PlayerRef::Target(_) | PlayerRef::TriggerPlayer | PlayerRef::DefendingPlayer
            ) =>
        {
            b.it_player.clone()
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
        // "Exile the top two cards of your library. If ..., exile the top three cards
        // instead."
        Effect::If {
            then, otherwise, ..
        } => exiles_your_top_cards(then) && exiles_your_top_cards(otherwise),
        _ => false,
    }
}

/// "you may play that card this turn", "until end of turn, you may play those cards",
/// "until the end of your next turn, you may play that card", "you may play it until your
/// next end step" after exiling the top cards of your library.
fn may_play_them(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let (duration, r) = if let Some(r) = l.strip_prefix("until the end of your next turn, ") {
        (Some(Duration::UntilEndOfYourNextTurn), r)
    } else if let Some(r) = l.strip_prefix("until end of turn, ") {
        (Some(Duration::EndOfTurn), r)
    } else if let Some(r) = l.strip_prefix("until the beginning of your next upkeep, ") {
        // Elkin Bottle (CR 500.4: until that step next begins).
        (Some(Duration::UntilYourNextStep(TriggerStep::Upkeep)), r)
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
        // Haste Magic: until your next end step begins (CR 500.4).
        (None, "until your next end step") => Duration::UntilYourNextStep(TriggerStep::End),
        // Valakut Exploration, Rassilon: the permission is for those objects, so it ends
        // when they leave exile (CR 400.7).
        (None, "for as long as it remains exiled" | "for as long as they remain exiled") => {
            Duration::Permanent
        }
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

/// "If it's a nonland card, you may cast that card this turn." after exiling the top card
/// of your library (Vance's Blasting Cannons): a permission to cast it, with the normal
/// timing rules and costs; a land card gets none (CR 305.9).
fn may_cast_if_nonland(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let duration = match l {
        "if it's a nonland card, you may cast that card this turn"
        | "if it's a nonland card, you may cast it this turn" => Duration::EndOfTurn,
        _ => return false,
    };
    if !exiles_your_top_cards(prev) {
        return false;
    }
    let card = Sel::Var(vars::IT);
    let grant = Effect::If {
        cond: Condition::SelMatches(
            card.clone(),
            Filter::Not(Box::new(Filter::Type(crate::types::CardType::Land))),
        ),
        then: Box::new(Effect::GrantPlayPermission {
            who: PlayerRef::You,
            what: card,
            duration,
            free: false,
        }),
        otherwise: Box::new(Effect::Noop),
    };
    *prev = Effect::seq(vec![std::mem::take(prev), grant]);
    true
}

/// "You may cast that card." after exiling the top card of your library (Chandra, Torch of
/// Defiance): it may be cast as the effect resolves, paying its costs (CR 608.2g, 601.2b);
/// "If you don't, ..." then refers to whether it was cast.
fn may_cast_it_now(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if !matches!(l, "you may cast that card" | "you may cast it") || !exiles_your_top_cards(prev) {
        return false;
    }
    *prev = Effect::seq(vec![
        std::mem::take(prev),
        Effect::CastCard {
            who: PlayerRef::You,
            what: Sel::Var(vars::IT),
            free: false,
            optional: true,
        },
    ]);
    true
}

/// "You may play that card until end of turn." as an instruction of its own, "that card"
/// being the card the previous instruction moved ("If you don't, you may play that card
/// until end of turn.", Spark of Creativity): a permission for that object (CR 400.7).
fn may_play_that_card(l: &str, b: &mut Builder) -> Option<Effect> {
    let duration = match end(l) {
        "you may play that card until end of turn" | "you may play that card this turn" => {
            Duration::EndOfTurn
        }
        _ => return None,
    };
    if !matches!(&b.it, Sel::Var(v) if *v == vars::IT) {
        return None;
    }
    Some(Effect::GrantPlayPermission {
        who: PlayerRef::You,
        what: Sel::Var(vars::IT),
        duration,
        free: false,
    })
}
