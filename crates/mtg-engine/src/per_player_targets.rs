//! Targets chosen for each player (`TargetSpec::per_player`): "For each opponent, gain
//! control of up to one target creature that player controls", "for each player, exile up
//! to one target nonland permanent that player controls", "for each opponent, you may
//! cast up to one target instant or sorcery card from that player's graveyard".
//!
//! As the spell or ability is put on the stack (CR 601.2c), its controller chooses, for
//! each player the phrase names, that many targets that match the object phrase with
//! "that player" being that player (`PlayerRel::Iterated`). The text defines the number of
//! targets (CR 601.2c: "In some cases, the number of targets will be defined by the
//! spell's text"): one for each such player, or up to one.
//! - "Up to one target" allows none for a player (CR 115.6), so a player with no legal
//!   choice gets none ("You can cast Mass Mutiny even if an opponent doesn't control any
//!   creatures").
//! - A required target ("for each opponent, destroy target noncreature permanent that
//!   player controls") must be chosen for each of those players. If a player has one, it
//!   must be chosen (Sylvan Primordial: "If an opponent has any legal permanents to
//!   target, you must target one of them"). If any of them has none, the requirement
//!   can't be met: the spell can't be cast, an ability can't be activated, and a
//!   triggered ability is removed from the stack (CR 601.2, 602.2, 603.3d; Vaevictis
//!   Asmadi, the Dire: "If any player controls no permanents, Vaevictis's ability won't
//!   have sufficient legal targets, so it's removed from the stack without any effect").
//!   Register entry `per-player-required-targets`.
//!
//! All the targets are one instance of the word "target", so they're all different (CR
//! 115.3), and the effect affects each of them ("untap those creatures").
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

/// What the slot asks for one player it names.
struct ForPlayer {
    player: PlayerId,
    /// The legal targets for that player.
    cands: Vec<Entity>,
    /// How many targets must be chosen for that player: the slot's minimum, but never
    /// more than its maximum ("X target creatures" with X = 0 has none, CR 601.2c). A lack
    /// of legal targets doesn't lower it.
    need: usize,
    /// How many may be.
    max: usize,
}

/// The slot's requirements, player by player. `exclude` are entities that can't be
/// chosen ("another target").
fn requirements(
    g: &Game,
    spec: &TargetSpec,
    ctx: &Ctx,
    stack_obj: ObjectId,
    exclude: &[Entity],
) -> Vec<ForPlayer> {
    players_for(g, spec, ctx)
        .into_iter()
        .map(|p| {
            let pctx = ctx_for(ctx, p);
            let cands: Vec<Entity> = g
                .legal_target_candidates(spec, &pctx, stack_obj)
                .into_iter()
                .filter(|e| !exclude.contains(e))
                .collect();
            let max = g.eval_value(&spec.max, &pctx).max(0) as usize;
            let need = (g.eval_value(&spec.min, &pctx).max(0) as usize).min(max);
            ForPlayer {
                player: p,
                cands,
                need,
                max,
            }
        })
        .collect()
}

/// Whether every player the slot names can get the targets it requires for them, with no
/// object or player chosen twice (one instance of "target", CR 115.3). If not, the spell
/// can't be cast and a triggered ability is removed from the stack (CR 601.2c, 603.3d).
pub fn possible(g: &Game, spec: &TargetSpec, ctx: &Ctx, stack_obj: ObjectId) -> bool {
    let needs: Vec<(usize, Vec<Entity>)> = requirements(g, spec, ctx, stack_obj, &[])
        .into_iter()
        .map(|r| (r.need, r.cands))
        .collect();
    assign(&needs).is_some()
}

/// Chooses the slot's targets, player by player. `exclude` are entities that can't be
/// chosen ("another target"). Returns the targets and, for each, the player it's for;
/// None if the players can't all get the targets the slot requires for them (CR 601.2c,
/// 603.3d).
pub fn choose(
    g: &mut Game,
    spec: &TargetSpec,
    ctx: &Ctx,
    stack_obj: ObjectId,
    chooser: PlayerId,
    exclude: &[Entity],
) -> Option<(Vec<Entity>, Vec<PlayerId>)> {
    let reqs = requirements(g, spec, ctx, stack_obj, exclude);
    // An assignment of their required targets to the players from `k` on, `own` of them
    // for player `k`, leaving out the targets already `chosen` and `also`.
    let rest = |k: usize, own: usize, chosen: &[Entity], also: &[Entity]| {
        let needs: Vec<(usize, Vec<Entity>)> = reqs[k..]
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let cands = r
                    .cands
                    .iter()
                    .copied()
                    .filter(|e| !chosen.contains(e) && !also.contains(e))
                    .collect();
                (if i == 0 { own } else { r.need }, cands)
            })
            .collect();
        assign(&needs)
    };
    rest(0, reqs.first().map_or(0, |r| r.need), &[], &[])?;
    let mut targets: Vec<Entity> = Vec::new();
    let mut players: Vec<PlayerId> = Vec::new();
    for (k, r) in reqs.iter().enumerate() {
        // The targets after which this player and the later ones can still get the
        // targets they require.
        let cands: Vec<Entity> = r
            .cands
            .iter()
            .copied()
            .filter(|e| {
                !targets.contains(e) && rest(k, r.need.saturating_sub(1), &targets, &[*e]).is_some()
            })
            .collect();
        if cands.len() < r.need {
            return None;
        }
        let max = r.max.min(cands.len());
        let min = r.need;
        if max == 0 {
            continue;
        }
        let ans = g.ask(
            chooser,
            Decision::ChooseTargets {
                source: stack_obj,
                text: format!("{} (for {})", spec.text, r.player),
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
                    && v.iter().enumerate().all(|(i, e)| !v[..i].contains(e))
                    && rest(k + 1, 0, &targets, &v).is_some() =>
            {
                v
            }
            // The fewest targets, leaving the later players theirs.
            _ => rest(k, min, &targets, &[])
                .and_then(|a| a.into_iter().next())
                .unwrap_or_default(),
        };
        for e in picked {
            targets.push(e);
            players.push(r.player);
        }
    }
    Some((targets, players))
}

/// An assignment of `n` different entities among `cands` to each `(n, cands)`, with no
/// entity assigned twice, if there is one (each required target matched to a different
/// legal target).
fn assign(needs: &[(usize, Vec<Entity>)]) -> Option<Vec<Vec<Entity>>> {
    // One seat for each required target, numbered by the requirement it's for.
    let seats: Vec<usize> = needs
        .iter()
        .enumerate()
        .flat_map(|(i, (n, _))| std::iter::repeat_n(i, *n))
        .collect();
    let mut held: Vec<(Entity, usize)> = Vec::new();
    for s in 0..seats.len() {
        let mut seen = Vec::new();
        if !seat(s, &seats, needs, &mut held, &mut seen) {
            return None;
        }
    }
    let mut out = vec![vec![]; needs.len()];
    for (e, s) in held {
        out[seats[s]].push(e);
    }
    Some(out)
}

/// Finds an entity for seat `s`, moving the seat that holds one it could take to another
/// (an augmenting path). `held` are the entities taken, with their seats.
fn seat(
    s: usize,
    seats: &[usize],
    needs: &[(usize, Vec<Entity>)],
    held: &mut Vec<(Entity, usize)>,
    seen: &mut Vec<Entity>,
) -> bool {
    for e in &needs[seats[s]].1 {
        if seen.contains(e) {
            continue;
        }
        seen.push(*e);
        let free = match held.iter().find(|(x, _)| x == e) {
            None => true,
            Some((_, other)) => {
                let other = *other;
                seat(other, seats, needs, held, seen)
            }
        };
        if free {
            held.retain(|(x, _)| x != e);
            held.push((*e, s));
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn o(n: u32) -> Entity {
        Entity::Object(ObjectId(n))
    }

    #[test]
    fn required_targets_are_matched_to_different_entities() {
        // The first two requirements can only have object 1: not both.
        assert!(assign(&[(1, vec![o(1)]), (1, vec![o(1)]), (1, vec![o(1), o(2)])]).is_none());
        // With object 3 for the second: 1, 3 and 2 (the third gives 1 up).
        let a = assign(&[
            (1, vec![o(1)]),
            (1, vec![o(1), o(3)]),
            (1, vec![o(1), o(2)]),
        ])
        .expect("possible");
        assert_eq!(a, vec![vec![o(1)], vec![o(3)], vec![o(2)]]);
        // A requirement of none ("up to one") needs nothing.
        assert_eq!(
            assign(&[(0, vec![]), (1, vec![o(4)])]),
            Some(vec![vec![], vec![o(4)]])
        );
        // Two targets for one requirement need two entities.
        assert!(assign(&[(2, vec![o(5)])]).is_none());
        assert_eq!(
            assign(&[(2, vec![o(5), o(6)])]).map(|a| a[0].len()),
            Some(2)
        );
    }
}
