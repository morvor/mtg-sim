//! Engine support for the choice grammar (see `oracle/patterns/choice_grammar*.rs`):
//! named filters, values and effects for choices made as a spell or ability resolves
//! (CR 608.2d) and the qualifiers that describe what may be chosen.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// `Filter::Custom`: a creature that dealt combat damage to the ability's controller this
/// turn ("a creature of their choice that dealt combat damage to you this turn").
pub const DEALT_COMBAT_DAMAGE_TO_YOU: &str = "choice_grammar:dealt combat damage to you this turn";

/// `Filter::Custom` prefix: a permanent that, chosen for one of several kinds a player must
/// choose one permanent of each of ("sacrifices an artifact, a creature, and a land"),
/// still lets them choose one for as many of the remaining kinds as possible: each
/// permanent fills one kind, and the player must fill as many as they can (Catch //
/// Release ruling). Followed by the JSON of [`OneOfEach`].
pub const ONE_OF_EACH: &str = "choice_grammar:one of each:";

/// The kinds a player chooses one permanent of each of, which one is being chosen now,
/// and the variable holding the permanents chosen for the earlier ones.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct OneOfEach {
    pub kinds: Vec<crate::ability::Filter>,
    pub slot: usize,
    pub picked: Option<crate::ability::Var>,
}

/// The most kinds that can each get a different object of the pool (bipartite matching
/// by augmenting paths; the lists are small).
fn max_matching(g: &Game, kinds: &[crate::ability::Filter], pool: &[ObjectId], ctx: &Ctx) -> usize {
    let fits: Vec<Vec<usize>> = kinds
        .iter()
        .map(|k| {
            (0..pool.len())
                .filter(|&j| g.matches(pool[j], k, ctx))
                .collect()
        })
        .collect();
    let mut owner: Vec<Option<usize>> = vec![None; pool.len()];
    fn augment(
        i: usize,
        fits: &[Vec<usize>],
        seen: &mut [bool],
        owner: &mut [Option<usize>],
    ) -> bool {
        for &j in &fits[i] {
            if seen[j] {
                continue;
            }
            seen[j] = true;
            if owner[j].is_none_or(|k| augment(k, fits, seen, owner)) {
                owner[j] = Some(i);
                return true;
            }
        }
        false
    }
    (0..kinds.len())
        .filter(|&i| augment(i, &fits, &mut vec![false; pool.len()], &mut owner))
        .count()
}

pub struct ChoiceGrammar;

impl KeywordRules for ChoiceGrammar {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name == DEALT_COMBAT_DAMAGE_TO_YOU {
            let you = ctx.controller;
            return Some(g.turn_events.iter().any(|e| {
                matches!(e, Event::Damage { source, target: Entity::Player(p), amount, combat: true }
                    if *source == id && *p == you && *amount > 0)
            }));
        }
        if let Some(json) = name.strip_prefix(ONE_OF_EACH) {
            let spec: OneOfEach = serde_json::from_str(json).ok()?;
            let p = ctx.iter_player.unwrap_or(ctx.controller);
            let picked: Vec<ObjectId> = spec
                .picked
                .and_then(|v| ctx.vars.get(&v))
                .map(|v| v.iter().filter_map(|e| e.object()).collect())
                .unwrap_or_default();
            let pool: Vec<ObjectId> = g
                .permanents()
                .filter(|o| o.controller == p && !picked.contains(&o.id))
                .map(|o| o.id)
                .collect();
            let kind = spec.kinds.get(spec.slot)?;
            if !pool.contains(&id) || !g.matches(id, kind, ctx) {
                return Some(false);
            }
            let rest = &spec.kinds[spec.slot + 1..];
            let all = max_matching(g, &spec.kinds[spec.slot..], &pool, ctx);
            let without: Vec<ObjectId> = pool.iter().copied().filter(|o| *o != id).collect();
            return Some(max_matching(g, rest, &without, ctx) + 1 == all);
        }
        None
    }
}

inventory::submit! { KeywordRegistration(&ChoiceGrammar) }
