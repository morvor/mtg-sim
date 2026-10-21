//! Combat grammar IV (cards3-combat-statics): objects put onto the battlefield attacking a
//! named player or blocking a named creature.
//!
//! * "put [cards] onto the battlefield tapped and attacking that player / that opponent /
//!   defending player [or a planeswalker they control]" (Kaalia of the Vast, Owlbear Cub,
//!   Zara, Hans Eriksson): the creature attacks that player (or one of their planeswalkers,
//!   as its controller chooses) rather than any player or planeswalker its controller
//!   chooses (CR 508.4). `Effect::TokensJoinCombat` around the move.
//! * "... It enters tapped and attacking that player." after a move (Shark Shredder).
//! * "put [a creature card] onto the battlefield blocking that creature" (Aetherplasm): it
//!   enters blocking that attacker (CR 509.4), never declared as a blocker.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::patterns::oracle_hardening_referents::{is_no_player_referent, is_no_referent};
use crate::oracle::phrases::end;

/// Whether `e` puts objects onto the battlefield attacking (any player): a move or a dig
/// whose destination says "attacking", possibly optional or conditional.
pub(crate) fn moves_attacking(e: &Effect) -> bool {
    match e {
        Effect::Move { to, .. } => to.zone == ZoneKind::Battlefield && to.attacking,
        Effect::Dig { take_to, .. } => {
            take_to.zone == ZoneKind::Battlefield && take_to.attacking
        }
        Effect::DigStep(s) => {
            matches!(&**s, DigStep::Take { to, .. } if to.zone == ZoneKind::Battlefield && to.attacking)
        }
        Effect::May { effect, .. } => moves_attacking(effect),
        Effect::Seq(v) => v.iter().any(moves_attacking) && !v.iter().any(creates_tokens),
        _ => false,
    }
}

/// Whether `e` puts objects onto the battlefield with a move or dig (not attacking).
fn moves_onto_battlefield(e: &Effect) -> bool {
    match e {
        Effect::Move { to, .. } => to.zone == ZoneKind::Battlefield && !to.attacking,
        Effect::Dig { take_to, .. } => {
            take_to.zone == ZoneKind::Battlefield && !take_to.attacking
        }
        Effect::May { effect, .. } => moves_onto_battlefield(effect),
        Effect::Seq(v) => v.iter().any(moves_onto_battlefield) && !v.iter().any(creates_tokens),
        _ => false,
    }
}

fn creates_tokens(e: &Effect) -> bool {
    matches!(
        e,
        Effect::CreateToken { .. }
            | Effect::CreateTokenWithPT { .. }
            | Effect::CreateTokenCopy { .. }
            | Effect::TokensJoinCombat { .. }
    )
}

/// The player a creature entering attacking attacks: "that player", "that opponent"
/// (the player the text refers to), "defending player".
fn named_defender(who: &str, b: &Builder) -> Option<PlayerRef> {
    match who {
        "that player" | "that opponent" => {
            let p = b.it_player.clone();
            if is_no_player_referent(&p) || matches!(p, PlayerRef::You) {
                return None;
            }
            Some(p)
        }
        "defending player" | "the defending player" => Some(PlayerRef::DefendingPlayer),
        _ => None,
    }
}

/// Splits "... attacking [defender][ or a planeswalker they control]" into the instruction
/// ending in "attacking", the defender's words and whether its planeswalkers may be
/// attacked instead.
pub(crate) fn split_defender(l: &str) -> Option<(&str, &str, bool)> {
    let (l, walkers) = match l
        .strip_suffix(" or a planeswalker they control")
        .or_else(|| l.strip_suffix(" or a planeswalker that player controls"))
    {
        Some(x) => (x, true),
        None => (l, false),
    };
    for who in [
        "that player",
        "that opponent",
        "defending player",
        "the defending player",
    ] {
        if let Some(base) = l.strip_suffix(who).and_then(|x| x.strip_suffix(" attacking ")) {
            return Some((&l[..base.len() + " attacking".len()], who, walkers));
        }
    }
    None
}

pub(crate) fn join_attacking(e: Effect, defender: PlayerRef, walkers: bool) -> Effect {
    let effect = if walkers {
        Effect::seq(vec![
            Effect::Custom(crate::kw::token_creation_rules::OR_THEIR_PLANESWALKERS.into()),
            e,
        ])
    } else {
        e
    };
    Effect::TokensJoinCombat {
        effect: Box::new(effect),
        attacking: Some(defender),
        blocking: None,
    }
}

/// "you may put an Angel, Demon, or Dragon creature card from your hand onto the battlefield
/// tapped and attacking that opponent", "put it onto the battlefield tapped and attacking
/// defending player or a planeswalker they control".
fn p_move_attacking_player(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    if !l.contains(" onto the battlefield ") || l.contains("create") {
        return None;
    }
    let (base, who, walkers) = split_defender(l)?;
    let defender = named_defender(who, b)?;
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let e = crate::oracle::effects::parse_clause(base, b);
    let Some(e) = e.filter(moves_attacking) else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        b.it_player = saved.2;
        return None;
    };
    Some(join_attacking(e, defender, walkers))
}

inventory::submit! { EffectPattern { name: "combat 4: put onto the battlefield attacking that player", priority: 89, parse: p_move_attacking_player } }

/// "you may put a creature card from your hand onto the battlefield blocking that creature"
/// (Aetherplasm).
fn p_move_blocking(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (base, blocked) = l.rsplit_once(" onto the battlefield blocking ")?;
    if l.contains("create") {
        return None;
    }
    let (attacker, rest) = object_ref(blocked, b)?;
    if !rest.trim().is_empty()
        || is_no_referent(&attacker)
        || !matches!(attacker, Sel::TriggerObject | Sel::Target(_) | Sel::Var(_))
    {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let e = crate::oracle::effects::parse_clause(&format!("{base} onto the battlefield"), b);
    let Some(e) = e.filter(moves_onto_battlefield) else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        b.it_player = saved.2;
        return None;
    };
    Some(Effect::TokensJoinCombat {
        effect: Box::new(e),
        attacking: None,
        blocking: Some(attacker),
    })
}

inventory::submit! { EffectPattern { name: "combat 4: put onto the battlefield blocking that creature", priority: 89, parse: p_move_blocking } }

/// The move instruction of `e` that puts objects onto the battlefield, last.
fn last_move(e: &mut Effect) -> Option<&mut Destination> {
    match e {
        Effect::Seq(v) => v.iter_mut().rev().find_map(last_move),
        Effect::May { effect, .. } => last_move(effect),
        Effect::Move { to, .. } if to.zone == ZoneKind::Battlefield && !to.attacking => Some(to),
        _ => None,
    }
}

/// "It enters tapped and attacking that player." after putting a card onto the
/// battlefield (Shark Shredder, Killer Clone).
fn f_it_enters_attacking_player(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l.trim());
    let Some(rest) = ["it enters ", "that creature enters ", "that card enters "]
        .iter()
        .find_map(|p| l.strip_prefix(p))
    else {
        return false;
    };
    let Some((base, who, walkers)) = split_defender(rest) else {
        return false;
    };
    if base != "tapped and attacking" {
        return false;
    }
    let Some(defender) = named_defender(who, b) else {
        return false;
    };
    if matches!(prev, Effect::TokensJoinCombat { .. }) {
        return false;
    }
    let Some(to) = last_move(prev) else {
        return false;
    };
    to.tapped = true;
    to.attacking = true;
    let inner = std::mem::take(prev);
    *prev = join_attacking(inner, defender, walkers);
    true
}

inventory::submit! { FollowupPattern { name: "combat 4: it enters tapped and attacking that player", priority: 44, apply: f_it_enters_attacking_player } }

/// "When you put a creature card onto the battlefield this way, it fights ~." (Hans
/// Eriksson): a reflexive triggered ability (CR 603.12) that triggers if the instruction
/// before put a card of that kind onto the battlefield ("it": that card) — not if it put
/// the card somewhere else instead ("Otherwise, put that card into your hand").
fn f_when_put_onto_battlefield_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l.trim());
    let Some((desc, text)) = l
        .strip_prefix("when you put ")
        .and_then(|r| r.split_once(" onto the battlefield this way, "))
    else {
        return false;
    };
    let Some(desc) = desc.strip_prefix("a ").or_else(|| desc.strip_prefix("an ")) else {
        return false;
    };
    let Some(f) = super::card_flow_search::card_filter(desc, b) else {
        return false;
    };
    if !puts_onto_battlefield(prev) {
        return false;
    }
    let Some(body) = super::r600_triggers::reflexive_body(text, b) else {
        return false;
    };
    let entered = Sel::Var(vars::IT);
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::If {
            cond: Condition::SelMatches(
                entered,
                Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]),
            ),
            then: Box::new(Effect::Reflexive {
                body: Box::new(body),
            }),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

/// Whether the last instruction of `e` may put a card onto the battlefield with a move.
fn puts_onto_battlefield(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => v.last().is_some_and(puts_onto_battlefield),
        Effect::If {
            then, otherwise, ..
        } => puts_onto_battlefield(then) || puts_onto_battlefield(otherwise),
        Effect::May { effect, .. } | Effect::TokensJoinCombat { effect, .. } => {
            puts_onto_battlefield(effect)
        }
        Effect::Move { to, .. } => to.zone == ZoneKind::Battlefield,
        _ => false,
    }
}

inventory::submit! { FollowupPattern { name: "combat 4: when you put a card onto the battlefield this way", priority: 60, apply: f_when_put_onto_battlefield_this_way } }
