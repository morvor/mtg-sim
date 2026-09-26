//! Oracle patterns for exiling cards face down (CR 406.3): "exile the top card of your
//! library face down", "look at the top card of your library, then exile it face down",
//! and "you may look at [it] for as long as it remains exiled".

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_number};
use crate::zones::MAY_LOOK_AT_EXILED;

/// "the top card of " / "the top N cards of " + whose library.
fn top_of_library(r: &str, b: &mut Builder) -> Option<(Sel, String)> {
    let r = r.strip_prefix("the top ")?;
    let (n, r) = if let Some(r) = r.strip_prefix("card of ") {
        (Value::c(1), r)
    } else {
        let (n, r) = parse_number(r)?;
        n.as_const()?;
        (n, r.strip_prefix("cards of ")?)
    };
    for (whose, who) in [
        ("your library", PlayerRef::You),
        ("that player's library", b.it_player.clone()),
        ("each opponent's library", PlayerRef::EachOpponent),
        ("each player's library", PlayerRef::EachPlayer),
    ] {
        if let Some(rest) = r.strip_prefix(whose) {
            return Some((Sel::TopOfLibrary(who, n), rest.to_string()));
        }
    }
    None
}

fn exile_face_down(what: Sel, b: &mut Builder) -> Effect {
    b.it = Sel::Var(vars::IT);
    Effect::Exile {
        what,
        face_down: true,
        link: true,
    }
}

/// "exile the top [N] card[s] of your library face down" — no player may look at the
/// cards unless something allows it (CR 406.3).
fn exile_top_face_down(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("exile ")?;
    let (what, rest) = top_of_library(r, b)?;
    if rest != " face down" {
        return None;
    }
    Some(exile_face_down(what, b))
}

inventory::submit! { EffectPattern { name: "r406 exile the top card face down", priority: 80, parse: exile_top_face_down } }

/// "look at the top card of your library, then exile it face down": having looked at it,
/// the player may continue to look at it while it remains exiled (CR 406.3).
fn look_then_exile_face_down(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("look at ")?;
    let (what, rest) = top_of_library(r, b)?;
    if !matches!(
        rest.as_str(),
        ", then exile it face down" | " and exile it face down" | ", then exile them face down"
    ) {
        return None;
    }
    let exile = exile_face_down(what, b);
    Some(Effect::seq(vec![
        exile,
        Effect::Custom(MAY_LOOK_AT_EXILED.into()),
    ]))
}

inventory::submit! { EffectPattern { name: "r406 look at the top card, then exile it face down", priority: 80, parse: look_then_exile_face_down } }

/// Whether the effect ends by exiling cards face down.
fn ends_with_face_down_exile(e: &Effect) -> bool {
    match e {
        Effect::Exile {
            face_down: true, ..
        } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_with_face_down_exile),
        _ => false,
    }
}

/// "You may look at it for as long as it remains exiled." after exiling cards face down.
fn may_look_while_exiled(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if !matches!(
        end(l),
        "you may look at it for as long as it remains exiled"
            | "you may look at that card for as long as it remains exiled"
            | "you may look at those cards for as long as they remain exiled"
            | "you may look at them for as long as they remain exiled"
    ) {
        return false;
    }
    if !ends_with_face_down_exile(prev) {
        return false;
    }
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, Effect::Custom(MAY_LOOK_AT_EXILED.into())]);
    true
}

inventory::submit! { FollowupPattern { name: "r406 you may look at it while it remains exiled", priority: 80, apply: may_look_while_exiled } }

/// "until end of turn, you may play cards [you own] exiled with ~" — a linked ability
/// (CR 406.6, 607.2a); face-down cards are turned face up as they're played (CR 406.3a).
fn may_play_cards_exiled_with(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("until end of turn, you may play ")?;
    let owned = match r {
        "cards you own exiled with ~" => true,
        "cards exiled with ~" => false,
        _ => return None,
    };
    let mut f = vec![
        Filter::InZone(ZoneKind::Exile),
        Filter::In(Box::new(Sel::Linked)),
    ];
    if owned {
        f.push(Filter::OwnedBy(PlayerRel::You));
    }
    Some(Effect::GrantPlayPermission {
        who: PlayerRef::You,
        what: Sel::All(Filter::and(f)),
        duration: Duration::EndOfTurn,
        free: false,
    })
}

inventory::submit! { EffectPattern { name: "r406 play cards exiled with ~", priority: 80, parse: may_play_cards_exiled_with } }

/// "You may play that card for as long as it remains exiled." after exiling cards face
/// down: a permission for those cards (CR 406.3a).
fn may_play_while_exiled(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if !matches!(
        end(l),
        "you may play that card for as long as it remains exiled"
            | "you may play it for as long as it remains exiled"
            | "for as long as it remains exiled, you may play it"
            | "you may play those cards for as long as they remain exiled"
    ) {
        return false;
    }
    if !ends_with_face_down_exile(prev) && !ends_with_may_look(prev) {
        return false;
    }
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::GrantPlayPermission {
            who: PlayerRef::You,
            what: Sel::Var(vars::IT),
            // The permission is for those objects: it ends when they leave exile.
            duration: Duration::Permanent,
            free: false,
        },
    ]);
    true
}

fn ends_with_may_look(e: &Effect) -> bool {
    match e {
        Effect::Custom(n) => n.as_str() == MAY_LOOK_AT_EXILED,
        Effect::Seq(v) => v.last().is_some_and(ends_with_may_look),
        _ => false,
    }
}

inventory::submit! { FollowupPattern { name: "r406 you may play it while it remains exiled", priority: 80, apply: may_play_while_exiled } }
