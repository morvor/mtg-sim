//! Dispatch to pluggable oracle patterns registered in `oracle/patterns/`.

use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::*;
use crate::oracle::CompileContext;

pub fn parse_effect_ext(l: &str, b: &mut Builder) -> Option<Effect> {
    for p in effect_patterns() {
        let saved_targets = b.targets.len();
        let saved_it = b.it.clone();
        let saved_player = b.it_player.clone();
        if let Some(e) = (p.parse)(l, b) {
            return Some(e);
        }
        b.targets.truncate(saved_targets);
        b.it = saved_it;
        b.it_player = saved_player;
    }
    None
}

/// Tries the registered [`FollowupPattern`]s on sentence `s` against the previous effect.
pub fn apply_followup_ext(s: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let lower = s.to_lowercase();
    let l = crate::oracle::phrases::end(&lower);
    followup_patterns().iter().any(|p| {
        let saved = prev.clone();
        let saved_targets = b.targets.len();
        let saved_it = b.it.clone();
        let saved_player = b.it_player.clone();
        if (p.apply)(l, prev, b) {
            true
        } else {
            *prev = saved;
            b.targets.truncate(saved_targets);
            b.it = saved_it;
            b.it_player = saved_player;
            false
        }
    })
}

pub fn parse_trigger_ext(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    trigger_patterns().iter().find_map(|p| (p.parse)(r))
}

pub fn parse_static_ext(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    static_patterns()
        .iter()
        .find_map(|p| (p.parse)(l, text, ctx))
}

pub fn parse_condition_ext(c: &str) -> Option<Condition> {
    condition_patterns().iter().find_map(|p| (p.parse)(c))
}

pub fn parse_ability_ext(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    ability_patterns()
        .iter()
        .find_map(|p| (p.parse)(block, ctx))
}
