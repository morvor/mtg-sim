//! "Each player chooses any number of creatures they control with total power N or less,
//! then sacrifices all other creatures they control." (Slaughter the Strong, Destined
//! Confrontation): the players choose in APNAP order (CR 101.4), then all the other
//! creatures are sacrificed at the same time. The total is the sum of the chosen
//! creatures' powers, so a creature with negative power subtracts from it (CR 107.1b,
//! 208.3).
//!
//! The effect is an `Effect::Custom` named by [`effect_name`]; the oracle phrase is parsed
//! in `oracle/patterns/total_power_keep.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;
use std::collections::BTreeSet;

const PREFIX: &str = "each player keeps creatures with total power at most:";

/// The `Effect::Custom` name of "each player chooses any number of creatures they control
/// with total power `max` or less, then sacrifices all other creatures they control".
pub fn effect_name(max: i32) -> String {
    format!("{PREFIX}{max}")
}

pub struct TotalPowerKeep;

impl KeywordRules for TotalPowerKeep {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some(max) = name.strip_prefix(PREFIX).and_then(|n| n.parse::<i32>().ok()) else {
            return false;
        };
        keep_total_power(g, ctx, max);
        true
    }
}

inventory::submit! { KeywordRegistration(&TotalPowerKeep) }

/// The total power of `ids`.
fn total_power(g: &Game, ids: &[ObjectId]) -> i64 {
    ids.iter().map(|o| g.obj(*o).power() as i64).sum()
}

/// The default choice: as many creatures as possible, lowest power first (a creature
/// with negative power leaves room for more).
fn default_keep(g: &Game, creatures: &[ObjectId], max: i32) -> Vec<ObjectId> {
    let mut sorted = creatures.to_vec();
    sorted.sort_by_key(|o| g.obj(*o).power());
    let mut kept = Vec::new();
    let mut total = 0i64;
    for o in sorted {
        let p = g.obj(o).power() as i64;
        if total + p <= max as i64 {
            total += p;
            kept.push(o);
        }
    }
    kept
}

fn keep_total_power(g: &mut Game, ctx: &mut Ctx, max: i32) {
    g.recompute();
    let players = g.eval_players(&crate::ability::PlayerRef::EachPlayer, ctx);
    let round = g.apnap_choices.len();
    let source = ctx.source;
    let mut sacrifice: Vec<(ObjectId, PlayerId)> = Vec::new();
    let requests = players.into_iter().map(|p| (p, ())).collect();
    g.apnap_round(requests, |g, p, ()| {
        let creatures: Vec<ObjectId> = g
            .permanents()
            .filter(|o| o.controller == p && o.is_creature())
            .map(|o| o.id)
            .collect();
        let candidates: Vec<Entity> = creatures.iter().map(|o| Entity::Object(*o)).collect();
        let kept = if creatures.is_empty() {
            vec![]
        } else {
            let ans = g.ask(
                p,
                Decision::ChooseEntities {
                    source,
                    prompt: format!(
                        "Choose any number of creatures you control with total power {max} or less"
                    ),
                    candidates: candidates.clone(),
                    min: 0,
                    max: candidates.len() as u32,
                },
            );
            let chosen: Option<Vec<ObjectId>> = match ans {
                Answer::Entities(v) => {
                    let mut seen = BTreeSet::new();
                    let ids: Vec<ObjectId> = v.iter().filter_map(|e| e.object()).collect();
                    let valid = ids.len() == v.len()
                        && v.iter().all(|e| candidates.contains(e) && seen.insert(*e))
                        && total_power(g, &ids) <= max as i64;
                    valid.then_some(ids)
                }
                _ => None,
            };
            chosen.unwrap_or_else(|| default_keep(g, &creatures, max))
        };
        g.record_apnap_choice(p, kept.clone());
        sacrifice.extend(
            creatures
                .into_iter()
                .filter(|o| !kept.contains(o))
                .map(|o| (o, p)),
        );
        vec![]
    });
    // What makes them sacrifice (CR 701.21; see `rule_statics::sacrifice_causes`).
    let cause = crate::rule_statics::sacrifice_causes::cause_of(ctx);
    sacrifice.retain(|(o, _)| !g.sacrifice_forbidden(*o, cause.as_ref()));
    let res = g.sacrifice_simultaneously(&sacrifice);
    g.end_apnap_choices(round);
    ctx.prev_value = res.len() as i64;
    ctx.prev_happened = !res.is_empty();
}
