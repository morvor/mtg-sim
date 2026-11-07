//! Targets chosen for each player (`TargetSpec::per_player`): "For each opponent, gain
//! control of up to one target creature that player controls", "for each player, exile up
//! to one target nonland permanent that player controls", "for each opponent, you may
//! cast up to one target instant or sorcery card from that player's graveyard".
//!
//! As the spell or ability is put on the stack (CR 601.2c), its controller chooses, for
//! each player the phrase names, that many targets that match the object phrase with
//! "that player" being that player (`PlayerRel::Iterated`). A player with no legal choice
//! gets no target ("You can cast Mass Mutiny even if an opponent doesn't control any
//! creatures"); if a player has one, a required target ("for each opponent, destroy target
//! noncreature permanent that player controls") must be chosen for them. All the targets
//! are one instance of the word "target", so they're all different (CR 115.3), and the
//! effect affects each of them ("untap those creatures").
//!
//! The player each target was chosen for is remembered (`ChosenMode::target_players`): as
//! the spell or ability resolves, a target is legal only if it still matches for that
//! player (CR 608.2b) — a creature that changed controllers since it was targeted isn't
//! "a creature that player controls" any more, even if its new controller is another
//! opponent. Changing a target (CR 115.7) keeps the player it's for.

use crate::ability::TargetSpec;
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::*;

/// The players a per-player slot chooses targets for, in turn order starting with the
/// active player (CR 101.4).
pub fn players_for(g: &Game, spec: &TargetSpec, ctx: &Ctx) -> Vec<PlayerId> {
    let Some(f) = &spec.per_player else {
        return vec![];
    };
    g.apnap()
        .into_iter()
        .filter(|p| g.player_filter_matches(f, *p, ctx))
        .collect()
}

/// The context for evaluating the slot's object phrase for player `p` ("that player").
pub fn ctx_for(ctx: &Ctx, p: PlayerId) -> Ctx {
    let mut c = ctx.clone();
    c.iter_player = Some(p);
    c
}

/// Whether `e` is a legal target of the slot for some player it names (used where no
/// particular player is meant, e.g. a copy that targets each object it could target).
pub fn legal_for_any_player(
    g: &Game,
    spec: &TargetSpec,
    e: Entity,
    ctx: &Ctx,
    stack_obj: ObjectId,
) -> bool {
    players_for(g, spec, ctx)
        .into_iter()
        .any(|p| g.is_legal_target(spec, e, &ctx_for(ctx, p), stack_obj))
}

/// Chooses the slot's targets, player by player. `exclude` are entities that can't be
/// chosen ("another target"). Returns the targets and, for each, the player it's for.
pub fn choose(
    g: &mut Game,
    spec: &TargetSpec,
    ctx: &Ctx,
    stack_obj: ObjectId,
    chooser: PlayerId,
    exclude: &[Entity],
) -> (Vec<Entity>, Vec<PlayerId>) {
    let mut targets: Vec<Entity> = Vec::new();
    let mut players: Vec<PlayerId> = Vec::new();
    for p in players_for(g, spec, ctx) {
        let pctx = ctx_for(ctx, p);
        let cands: Vec<Entity> = g
            .legal_target_candidates(spec, &pctx, stack_obj)
            .into_iter()
            .filter(|e| !targets.contains(e) && !exclude.contains(e))
            .collect();
        let max = (g.eval_value(&spec.max, &pctx).max(0) as usize).min(cands.len());
        // A required target is chosen for this player only if they have one (Sylvan
        // Primordial: "If an opponent has any legal permanents to target, you must
        // target one of them").
        let min = (g.eval_value(&spec.min, &pctx).max(0) as usize).min(max);
        if max == 0 {
            continue;
        }
        let ans = g.ask(
            chooser,
            Decision::ChooseTargets {
                source: stack_obj,
                text: format!("{} (for {p})", spec.text),
                candidates: cands.clone(),
                min: min as u32,
                max: max as u32,
            },
        );
        let picked: Vec<Entity> = match ans {
            Answer::Entities(v)
                if v.len() >= min
                    && v.len() <= max
                    && v.iter().all(|e| cands.contains(e))
                    && v.iter().enumerate().all(|(i, e)| !v[..i].contains(e)) =>
            {
                v
            }
            _ => cands.iter().copied().take(min).collect(),
        };
        for e in picked {
            targets.push(e);
            players.push(p);
        }
    }
    (targets, players)
}
