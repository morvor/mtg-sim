//! Glimmervoid Basin:
//!
//! * "Whenever a player casts an instant or sorcery spell with a single target, ..."
//!   (CR 115.9a): a cast trigger for spells with exactly one target.
//! * "that player copies that spell for each other spell, permanent, card not on the
//!   battlefield, and/or player the spell could target. Each copy targets a different one
//!   of them." (CR 707.10d): the caster copies it once for each other legal target.
//! * "Each player except that creature's controller creates a token that's a copy of that
//!   creature." (CR 707.2, 111.1).

use super::{EffectPattern, FollowupPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::effects::{object_ref, player_ref, Builder};
use crate::oracle::phrases::*;

/// "a player casts an instant or sorcery spell with a single target".
fn cast_with_single_target(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = end(r);
    let (who, rest) = if let Some(x) = r.strip_prefix("you cast ") {
        (PlayerRel::You, x)
    } else if let Some(x) = r.strip_prefix("an opponent casts ") {
        (PlayerRel::Opponent, x)
    } else if let Some(x) = r.strip_prefix("a player casts ") {
        (PlayerRel::Any, x)
    } else {
        return None;
    };
    let rest = rest.strip_suffix(" with a single target")?;
    let rest = rest.strip_prefix("a ").or_else(|| rest.strip_prefix("an "))?;
    let kind = if rest == "spell" {
        Filter::Any
    } else {
        let (f, _, tail) = parse_object_phrase(rest.strip_suffix(" spell")?)?;
        if !end(tail).is_empty() {
            return None;
        }
        f
    };
    let filter = Filter::and(vec![
        kind,
        Filter::StackTargets(Box::new(TargetsFilter::Count(1))),
    ]);
    Some((
        TriggerCond::CastSpell { who, filter },
        Sel::TriggerSpell,
        PlayerRef::TriggerPlayer,
    ))
}

inventory::submit! { TriggerPattern { name: "casts a spell with a single target", priority: 100, parse: cast_with_single_target } }

/// "that player copies that spell for each other spell, permanent, card not on the
/// battlefield, and/or player the spell could target": every other legal target.
fn copies_for_each_other_target(l: &str, b: &mut Builder) -> Option<Effect> {
    if !matches!(b.it, Sel::TriggerSpell) {
        return None;
    }
    let l = end(l);
    let (who, r) = if let Some(r) = l.strip_prefix("that player copies that spell for each other ")
    {
        (b.it_player.clone(), r)
    } else {
        (
            PlayerRef::You,
            l.strip_prefix("copy that spell for each other ")?,
        )
    };
    if r != "spell, permanent, card not on the battlefield, and/or player the spell could target"
    {
        return None;
    }
    let copy = Effect::CopySpellRetargeted {
        what: Sel::TriggerSpell,
        target: None,
    };
    Some(match who {
        PlayerRef::You => copy,
        who => Effect::AsPlayer {
            who,
            effect: Box::new(copy),
        },
    })
}

inventory::submit! { EffectPattern { name: "copies that spell for each other target it could have", priority: 100, parse: copies_for_each_other_target } }

/// "Each copy targets a different one of them." after copying a spell for each other
/// object or player it could target: how those copies are made (CR 707.10d).
fn each_copy_targets_a_different_one(s: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = s.to_lowercase();
    if end(&l) != "each copy targets a different one of them" {
        return false;
    }
    let retargeted = |e: &Effect| matches!(e, Effect::CopySpellRetargeted { target: None, .. });
    match prev {
        Effect::AsPlayer { effect, .. } => retargeted(effect),
        e => retargeted(e),
    }
}

inventory::submit! { FollowupPattern { name: "each copy targets a different one of them", priority: 100, apply: each_copy_targets_a_different_one } }

/// "each player [except that creature's controller] creates a token that's a copy of
/// [object]".
fn each_player_creates_copy(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("each player ")?;
    // The object named in the exception ("that creature"), which the copied object's
    // phrase may name again.
    let mut named: Option<(&str, Sel)> = None;
    let (players, r) = match r.strip_prefix("except ") {
        Some(x) => {
            let (who, rest) = x.split_once(" creates ")?;
            // "that creature's controller", or another player reference.
            let p = match who.strip_suffix("'s controller") {
                Some(obj) => {
                    let saved = b.targets.len();
                    let (sel, tail) = object_ref(obj, b)?;
                    if !tail.trim().is_empty() || b.targets.len() != saved {
                        b.targets.truncate(saved);
                        return None;
                    }
                    named = Some((obj, sel.clone()));
                    PlayerRef::ControllerOf(Box::new(sel))
                }
                None => {
                    let (p, tail) = player_ref(who, b)?;
                    if !tail.trim().is_empty() {
                        return None;
                    }
                    p
                }
            };
            (
                PlayerRef::Each(PlayerFilter::Not(Box::new(PlayerFilter::Ref(Box::new(p))))),
                rest,
            )
        }
        None => (PlayerRef::EachPlayer, r.strip_prefix("creates ")?),
    };
    let what = r
        .strip_prefix("a token that's a copy of ")
        .or_else(|| r.strip_prefix("a token that is a copy of "))?;
    let of = match named {
        Some((phrase, sel)) if end(what) == phrase => sel,
        _ => {
            let saved = b.targets.len();
            let (of, tail) = object_ref(what, b)?;
            if !end(&tail).trim().is_empty() || b.targets.len() != saved {
                b.targets.truncate(saved);
                return None;
            }
            of
        }
    };
    Some(Effect::CreateTokenCopy {
        of,
        count: Value::c(1),
        controller: players,
        tapped: false,
        attacking: false,
        mods: vec![],
    })
}

inventory::submit! { EffectPattern { name: "each player creates a token that's a copy of", priority: 100, parse: each_player_creates_copy } }
