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

/// `Effect::Custom` prefix: choose a player at random among some players and store them
/// in a variable (JSON of `(Var, PlayerRef)`).
pub const RANDOM_PLAYER: &str = "choice_grammar:random player:";
/// `Effect::Custom` prefix: choose one of several numbers at random and store it as a
/// number variable, and as the previous instruction's value (JSON of `(Var, Vec<i32>)`).
pub const RANDOM_NUMBER: &str = "choice_grammar:random number:";

/// "choose an opponent at random": the effect storing the player chosen in `var`.
pub fn random_player_effect(
    var: crate::ability::Var,
    from: crate::ability::PlayerRef,
) -> Option<crate::ability::Effect> {
    let json = serde_json::to_string(&(var, from)).ok()?;
    Some(crate::ability::Effect::Custom(
        format!("{RANDOM_PLAYER}{json}").into(),
    ))
}

/// "choose 1, 2, or 3 at random": the effect storing the number chosen in `var`.
pub fn random_number_effect(var: crate::ability::Var, nums: &[i32]) -> Option<crate::ability::Effect> {
    let json = serde_json::to_string(&(var, nums)).ok()?;
    Some(crate::ability::Effect::Custom(
        format!("{RANDOM_NUMBER}{json}").into(),
    ))
}

/// `Effect::Custom` prefix: the opponent who makes a choice "an opponent" makes (CR
/// 801.5a, see `Game::deciding_opponent`), stored in a variable (JSON of the `Var`).
pub const DECIDING_OPPONENT: &str = "choice_grammar:deciding opponent:";

/// "An opponent chooses ...": the effect storing that opponent in `var`.
pub fn deciding_opponent_effect(var: crate::ability::Var) -> Option<crate::ability::Effect> {
    let json = serde_json::to_string(&var).ok()?;
    Some(crate::ability::Effect::Custom(
        format!("{DECIDING_OPPONENT}{json}").into(),
    ))
}

pub struct ChoiceGrammar;

impl KeywordRules for ChoiceGrammar {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        use rand::seq::SliceRandom;
        if let Some(json) = name.strip_prefix(RANDOM_PLAYER) {
            let Ok((var, from)) =
                serde_json::from_str::<(crate::ability::Var, crate::ability::PlayerRef)>(json)
            else {
                return true;
            };
            let players = g.eval_players(&from, ctx);
            let pick: Vec<Entity> = players
                .choose(&mut g.rng)
                .map(|p| Entity::Player(*p))
                .into_iter()
                .collect();
            ctx.prev_happened = !pick.is_empty();
            ctx.chosen_player = pick.first().and_then(|e| e.player());
            ctx.set_var(var, pick);
            return true;
        }
        if let Some(json) = name.strip_prefix(DECIDING_OPPONENT) {
            let Ok(var) = serde_json::from_str::<crate::ability::Var>(json) else {
                return true;
            };
            let Some(obj) = ctx.stack_obj.or(ctx.source) else {
                return true;
            };
            let p = g.deciding_opponent(ctx.controller, obj, ctx);
            ctx.set_var(var, vec![Entity::Player(p)]);
            return true;
        }
        if let Some(json) = name.strip_prefix(RANDOM_NUMBER) {
            let Ok((var, nums)) = serde_json::from_str::<(crate::ability::Var, Vec<i32>)>(json)
            else {
                return true;
            };
            if let Some(n) = nums.choose(&mut g.rng).copied() {
                ctx.nums.insert(var, n as i64);
                ctx.prev_value = n as i64;
            }
            return true;
        }
        false
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
