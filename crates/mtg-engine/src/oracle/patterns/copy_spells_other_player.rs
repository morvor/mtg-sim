//! Another player copying a spell (CR 707.10): "Whenever a player casts an instant or
//! sorcery spell, that player copies it." (Izzet Steam Maze, Bonus Round) and "The
//! controller of target instant or sorcery spell copies it." (Meletis Charlatan). That
//! player performs the copying, so they control the copy and choose any new targets for
//! it (CR 707.10c): the copy is made as that player (`Effect::AsPlayer`). The follow-ups
//! "The player may choose new targets for the copy." / "That player may choose new targets
//! for the copy." and "... and may choose new targets for the copy" let them.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// The spell `what` copied by `who`.
fn copied_by(who: PlayerRef, what: Sel, new_targets: bool) -> Effect {
    Effect::AsPlayer {
        who,
        effect: Box::new(Effect::CopySpell {
            what,
            count: Value::c(1),
            new_targets,
        }),
    }
}

/// "that player copies it [and may choose new targets for the copy]", "the controller of
/// target instant or sorcery spell copies it".
fn other_player_copies(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (l, new_targets) = match l.strip_suffix(" and may choose new targets for the copy") {
        Some(r) => (r, true),
        None => (l, false),
    };
    if matches!(l, "that player copies it" | "that player copies that spell") {
        // The spell whose casting triggered the ability, copied by its caster.
        if !matches!(b.it, Sel::TriggerSpell) || matches!(b.it_player, PlayerRef::You) {
            return None;
        }
        return Some(copied_by(b.it_player.clone(), Sel::TriggerSpell, new_targets));
    }
    let r = l.strip_prefix("the controller of ")?;
    let r = r.strip_suffix(" copies it")?;
    let (spec, tail) = parse_target(r)?;
    if !end(tail).is_empty() || !matches!(spec.what, TargetKind::Spell(_)) {
        return None;
    }
    let slot = b.add_target(spec, r);
    let what = Sel::Target(slot);
    b.it_player = PlayerRef::ControllerOf(Box::new(what.clone()));
    Some(copied_by(
        PlayerRef::ControllerOf(Box::new(what.clone())),
        what,
        new_targets,
    ))
}

inventory::submit! { EffectPattern { name: "another player copies a spell", priority: 100, parse: other_player_copies } }

/// "The player may choose new targets for the copy." / "That player may choose new
/// targets for the copy." after another player copied a spell (CR 707.10c).
fn player_may_choose_new_targets(s: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = s.to_lowercase();
    if !matches!(
        end(&l),
        "the player may choose new targets for the copy"
            | "that player may choose new targets for the copy"
    ) {
        return false;
    }
    match prev {
        Effect::AsPlayer { effect, .. } => match effect.as_mut() {
            Effect::CopySpell { new_targets, .. } => {
                *new_targets = true;
                true
            }
            _ => false,
        },
        _ => false,
    }
}

inventory::submit! { FollowupPattern { name: "copy by another player: new targets", priority: 100, apply: player_may_choose_new_targets } }
