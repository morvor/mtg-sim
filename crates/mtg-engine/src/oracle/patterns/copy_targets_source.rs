//! "Whenever a player casts a spell that targets only a single creature other than ~, you
//! may copy that spell. The copy targets ~." (Ivy, Gleeful Spellthief):
//!
//! * a cast trigger for spells whose only target is one object of a kind (CR 115.9c);
//! * "The copy targets ~.": a copy with a specified new target (CR 707.10e).

use super::{FollowupPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// "[a player casts] a spell that targets only a single creature other than ~".
pub(crate) fn cast_targeting_only_a_single(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
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
    let (spell, target) = rest.split_once(" that targets only a single ")?;
    let spell = spell
        .strip_prefix("a ")
        .or_else(|| spell.strip_prefix("an "))?;
    let kind = if spell == "spell" {
        Filter::Any
    } else {
        let (f, _, tail) = parse_object_phrase(spell.strip_suffix(" spell")?)?;
        if !end(tail).is_empty() {
            return None;
        }
        f
    };
    let (objects, plural, tail) = parse_object_phrase(target)?;
    if plural || !end(tail).is_empty() {
        return None;
    }
    let filter = Filter::and(vec![
        kind,
        Filter::StackTargets(Box::new(TargetsFilter::Only {
            objects: Some(objects),
            players: None,
        })),
    ]);
    Some((
        TriggerCond::CastSpell { who, filter },
        Sel::TriggerSpell,
        PlayerRef::TriggerPlayer,
    ))
}

inventory::submit! { TriggerPattern { name: "casts a spell that targets only a single [object]", priority: 100, parse: cast_targeting_only_a_single } }

/// "The copy targets ~." after "[you may] copy that spell".
fn the_copy_targets_source(s: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = s.to_lowercase();
    // "Create a 1/1 red Soldier creature token with haste, then copy that spell. The copy
    // targets that token." (Frontline Heroism): the token just created.
    let target = match end(&l) {
        "the copy targets ~" => Sel::This,
        "the copy targets that token" | "the copy targets the token" => Sel::Var(vars::CREATED),
        _ => return false,
    };
    fn retarget(e: &mut Effect, target: &Sel, created: bool) -> bool {
        match e {
            Effect::May { effect, .. } => retarget(effect, target, created),
            // The token is created earlier in the same instruction.
            Effect::Seq(v) if matches!(target, Sel::Var(_)) => {
                let created = created
                    || v.iter().any(|x| matches!(x, Effect::CreateToken { .. }));
                v.last_mut().is_some_and(|x| retarget(x, target, created))
            }
            Effect::CopySpell {
                what,
                count: Value::Const(1),
                ..
            } if created || matches!(target, Sel::This) => {
                *e = Effect::CopySpellRetargeted {
                    what: what.clone(),
                    target: Some(target.clone()),
                };
                true
            }
            _ => false,
        }
    }
    retarget(prev, &target, false)
}

inventory::submit! { FollowupPattern { name: "the copy targets ~", priority: 100, apply: the_copy_targets_source } }
