//! Spell copies and target changes II (CR 115.7, 707.10).
//!
//! * "The new target must be a player." after "Change the target of target spell that
//!   targets only a player." (Rebound): a restriction on the new target (CR 115.7).
//! * "If target spell has only one target and that target is a creature, change that
//!   spell's target to another creature." (Meddle; Quicksilver Dragon: "... and that
//!   target is ~ ...").
//! * "if it has a single target" (Captured by the Consulate's intervening "if" clause,
//!   CR 115.9a) and "change the target to enchanted creature if able".

use super::{ConditionPattern, EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// "player", "creature": what a new target must be.
fn new_target_kind(s: &str) -> Option<Sel> {
    let s = end(s.trim());
    let s = s
        .strip_prefix("a ")
        .or_else(|| s.strip_prefix("an "))
        .unwrap_or(s);
    if s == "player" {
        return Some(Sel::Players(PlayerRef::EachPlayer));
    }
    let (f, plural, tail) = parse_object_phrase(s)?;
    (!plural && end(tail).is_empty()).then(|| Sel::All(f))
}

/// "The new target must be a player." after changing a spell's target.
fn new_target_must_be(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let lower = l.to_lowercase();
    let Some(r) = end(&lower).strip_prefix("the new target must be ") else {
        return false;
    };
    let Some(sel) = new_target_kind(r) else {
        return false;
    };
    let slot = match prev {
        Effect::ChangeTargets { to, .. } => to,
        Effect::If { then, .. } => match then.as_mut() {
            Effect::ChangeTargets { to, .. } => to,
            _ => return false,
        },
        _ => return false,
    };
    if slot.is_some() {
        return false;
    }
    *slot = Some(sel);
    true
}

inventory::submit! { FollowupPattern { name: "copy spell grammar 2: the new target must be", priority: 45, apply: new_target_must_be } }

/// "if target spell has only one target and that target is a creature, change that
/// spell's target to another creature", "... and that target is ~, ...".
fn change_only_target(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("if target spell has only one target and that target is ")?;
    let (what, rest) = r.split_once(", ")?;
    let rest = rest.strip_prefix("change that spell's target to another ")?;
    let only = if what == "~" {
        Filter::Source
    } else {
        let w = what
            .strip_prefix("a ")
            .or_else(|| what.strip_prefix("an "))?;
        let (f, plural, tail) = parse_object_phrase(w)?;
        if plural || !end(tail).is_empty() {
            return None;
        }
        f
    };
    let to = new_target_kind(rest)?;
    let mut spec = TargetSpec::any_target();
    spec.what = TargetKind::Spell(Filter::Spell);
    let slot = b.add_target(spec, "target spell");
    let spell = Sel::Target(slot);
    b.it = spell.clone();
    Some(Effect::If {
        cond: Condition::SelMatches(
            spell.clone(),
            Filter::and(vec![
                Filter::StackTargets(Box::new(TargetsFilter::Count(1))),
                Filter::StackTargets(Box::new(TargetsFilter::Only {
                    objects: Some(only),
                    players: None,
                })),
            ]),
        ),
        then: Box::new(Effect::ChangeTargets {
            what: spell,
            who: PlayerRef::You,
            how: TargetChange::All,
            to: Some(to),
        }),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "copy spell grammar 2: if target spell has only one target, change it", priority: 80, parse: change_only_target } }

/// "it has a single target" (about the spell a cast trigger is about, CR 115.9a).
fn has_a_single_target(c: &str) -> Option<Condition> {
    match end(c) {
        "it has a single target" | "that spell has a single target" => Some(Condition::SelMatches(
            Sel::TriggerSpell,
            Filter::StackTargets(Box::new(TargetsFilter::Count(1))),
        )),
        _ => None,
    }
}

inventory::submit! { ConditionPattern { name: "copy spell grammar 2: it has a single target", priority: 80, parse: has_a_single_target } }

/// "if it has a single target" as an intervening "if" clause of an ability that triggers
/// on a spell: that spell (`b.it`).
pub(crate) fn it_has_a_single_target(c: &str, b: &Builder) -> Option<Condition> {
    if !matches!(b.it, Sel::TriggerSpell) || end(c) != "it has a single target" {
        return None;
    }
    has_a_single_target(c)
}

/// "change the target to enchanted creature if able" in an ability that triggers on a
/// spell (Captured by the Consulate): the spell's target becomes that creature, if it's a
/// legal target for it (CR 115.7).
fn change_the_target_to(l: &str, b: &mut Builder) -> Option<Effect> {
    if !matches!(b.it, Sel::TriggerSpell) {
        return None;
    }
    let r = end(l)
        .strip_prefix("change the target to ")?
        .strip_suffix(" if able")?;
    let to = match r {
        "enchanted creature" | "enchanted permanent" => Sel::AttachedTo,
        "~" => Sel::This,
        _ => return None,
    };
    Some(Effect::ChangeTargets {
        what: Sel::TriggerSpell,
        who: PlayerRef::You,
        how: TargetChange::All,
        to: Some(to),
    })
}

inventory::submit! { EffectPattern { name: "copy spell grammar 2: change the target to [object] if able", priority: 80, parse: change_the_target_to } }

/// "each other player may copy that spell and may choose new targets for the copy they
/// control" (Curse of Echoes), "each opponent may copy that spell and ..." (Tempt with
/// Mayhem): each of those players in turn may copy it; they control their copy and choose
/// its new targets (CR 707.10, 707.10c).
fn each_player_may_copy(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (players, r) = if let Some(r) = l.strip_prefix("each other player may copy ") {
        // Other than the player the ability is about (the caster), or than you.
        let other = match &b.it_player {
            p @ (PlayerRef::TriggerPlayer | PlayerRef::Target(_)) => p.clone(),
            _ => PlayerRef::You,
        };
        (
            PlayerRef::Each(PlayerFilter::Not(Box::new(PlayerFilter::Ref(Box::new(
                other,
            ))))),
            r,
        )
    } else if let Some(r) = l.strip_prefix("each opponent may copy ") {
        (PlayerRef::Each(PlayerFilter::Opponent), r)
    } else {
        return None;
    };
    let (what, new_targets) =
        match r.strip_suffix(" and may choose new targets for the copy they control") {
            Some(w) => (w, true),
            None => (r, false),
        };
    let spell = match (what, &b.it) {
        ("that spell" | "it", s @ (Sel::TriggerSpell | Sel::Target(_))) => s.clone(),
        _ => return None,
    };
    Some(Effect::ForEachPlayer {
        who: players,
        effect: Box::new(Effect::AsPlayer {
            who: PlayerRef::Iterated,
            effect: Box::new(Effect::May {
                who: PlayerRef::You,
                effect: Box::new(Effect::CopySpell {
                    what: spell,
                    count: Value::c(1),
                    new_targets,
                }),
            }),
        }),
    })
}

inventory::submit! { EffectPattern { name: "copy spell grammar 2: each other player may copy that spell", priority: 80, parse: each_player_may_copy } }
