//! Effects that reselect which player or permanent attacking creatures are attacking
//! (CR 508.7): "you may reselect which player or permanent target attacking creature is
//! attacking" (Portal Mage, Misleading Signpost) and "for each attacking creature, you may
//! reselect which player or permanent that creature is attacking" (Windshaper Planetar).
//! The text is parsed in `oracle/patterns/r508_reselect_attack.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::Sel;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::types::*;

/// `Effect::Custom` name prefix: reselect for the attacking creature in the target slot
/// that follows (`"reselect attack of target:0"`).
pub const RESELECT_TARGET: &str = "reselect attack of target:";
/// `Effect::Custom` name: reselect for each attacking creature.
pub const RESELECT_EACH: &str = "reselect attack of each attacking creature";
/// `Effect::Custom` name: "you may reselect which player ~ is attacking" (Capricopian):
/// the source, among players only.
pub const RESELECT_SELF_PLAYER: &str = "reselect which player the source is attacking";

/// What `attacker` could be made to attack instead of what it's attacking now: an opponent
/// of its controller, or a planeswalker or battle of one, within the limits of CR
/// 508.7c–e (requirements and restrictions on attacking don't apply, CR 508.7b).
pub fn reselect_options(g: &Game, attacker: ObjectId) -> Vec<Entity> {
    let current = g
        .combat
        .as_ref()
        .and_then(|c| c.attackers.iter().find(|a| a.id == attacker))
        .and_then(|a| a.target);
    let mut cands: Vec<Entity> = g.players.iter().map(|p| Entity::Player(p.id)).collect();
    cands.extend(
        g.permanents()
            .filter(|o| o.is(CardType::Planeswalker) || o.is(CardType::Battle))
            .map(|o| Entity::Object(o.id)),
    );
    cands
        .into_iter()
        .filter(|e| Some(*e) != current)
        .filter(|e| {
            // The primitive checks every rule of CR 508.7; try it on a copy of the game.
            let mut trial = g.clone();
            trial.observer = None;
            crate::combat::reselect_attack_target(&mut trial, attacker, *e)
        })
        .collect()
}

fn reselect(g: &mut Game, attacker: ObjectId, ctx: &Ctx, players_only: bool) {
    if !g.is_attacking(attacker) || g.obj(attacker).zone != Zone::Battlefield {
        return;
    }
    let mut options = reselect_options(g, attacker);
    if players_only {
        options.retain(|e| matches!(e, Entity::Player(_)));
    }
    if options.is_empty() {
        return;
    }
    // Choosing nothing leaves it attacking what it's attacking ("you may").
    let chosen = g.ask_entities(
        ctx.controller,
        ctx.source,
        "Reselect what this creature is attacking (choose nothing to leave it)",
        options,
        0,
        1,
    );
    if let Some(e) = chosen.first() {
        crate::combat::reselect_attack_target(g, attacker, *e);
    }
}

pub struct ReselectAttack;

impl KeywordRules for ReselectAttack {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let players_only = name == RESELECT_SELF_PLAYER;
        let attackers: Vec<ObjectId> = if name == RESELECT_EACH {
            g.attackers()
        } else if players_only {
            ctx.source.map(|s| g.current(s)).into_iter().collect()
        } else if let Some(slot) = name
            .strip_prefix(RESELECT_TARGET)
            .and_then(|s| s.parse::<u8>().ok())
        {
            g.resolve_objects(&Sel::Target(slot), ctx)
        } else {
            return false;
        };
        for a in attackers {
            reselect(g, a, ctx, players_only);
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&ReselectAttack) }
