//! "Players can't untap more than one land during their untap steps" (Winter Orb,
//! Static Orb, Smoke, ...): limits on the untap step's turn-based action (CR 502.3).

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::*;

impl Game {
    /// Removes from `to_untap` the permanents the active player may not untap because of
    /// [`Restriction::MaxUntaps`] effects. The player chooses which ones untap, obeying
    /// every limit at once (CR 502.3); the rest untap as usual, and as many as the limits
    /// allow do untap.
    pub(crate) fn limit_untaps(&mut self, active: PlayerId, to_untap: &mut Vec<ObjectId>) {
        let limits: Vec<(Ctx, Filter, u32)> = self
            .statics
            .restrictions
            .iter()
            .map(|(s, c, r)| (Some(*s), *c, r))
            .chain(
                self.rule_effects
                    .iter()
                    .map(|e| (e.source, e.controller, &e.restriction)),
            )
            .filter_map(|(s, c, r)| match r {
                Restriction::MaxUntaps { who, what, n } => {
                    let ctx = Ctx::new(s, c);
                    self.player_filter_matches(who, active, &ctx)
                        .then(|| (ctx, what.clone(), *n))
                }
                _ => None,
            })
            .collect();
        // The permanents some limit applies to; the others aren't affected.
        let limited: Vec<ObjectId> = to_untap
            .iter()
            .copied()
            .filter(|id| {
                limits
                    .iter()
                    .any(|(ctx, what, _)| self.matches(*id, what, ctx))
            })
            .collect();
        let fits = |g: &Game, set: &[ObjectId]| {
            limits.iter().all(|(ctx, what, n)| {
                set.iter().filter(|id| g.matches(**id, what, ctx)).count() as u32 <= *n
            })
        };
        if fits(self, &limited) {
            return;
        }
        let max = limits
            .iter()
            .map(|(_, _, n)| *n)
            .sum::<u32>()
            .min(limited.len() as u32);
        let source = limits.first().and_then(|(ctx, _, _)| ctx.source);
        let mut chosen = self.ask_objects(
            active,
            source,
            "Choose permanents to untap",
            limited.clone(),
            0,
            max,
        );
        if !fits(self, &chosen) {
            chosen.clear();
        }
        // Permanents untap unless an effect stops them: fill up to the limits.
        for id in &limited {
            if chosen.contains(id) {
                continue;
            }
            chosen.push(*id);
            if !fits(self, &chosen) {
                chosen.pop();
            }
        }
        to_untap.retain(|id| !limited.contains(id) || chosen.contains(id));
    }
}
