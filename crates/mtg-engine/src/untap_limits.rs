//! "Players can't untap more than one land during their untap steps" (Winter Orb,
//! Static Orb, Smoke, ...): limits on the untap step's turn-based action (CR 502.3).

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::*;

impl Game {
    /// Removes from `to_untap` the permanents the active player may not untap because of
    /// [`Restriction::MaxUntaps`] effects. The player chooses which ones to untap; each
    /// limit applies in turn.
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
        for (ctx, what, n) in limits {
            let matching: Vec<ObjectId> = to_untap
                .iter()
                .copied()
                .filter(|id| self.matches(*id, &what, &ctx))
                .collect();
            if matching.len() as u32 <= n {
                continue;
            }
            let keep = self.ask_objects(
                active,
                ctx.source,
                "Choose permanents to untap",
                matching.clone(),
                n,
                n,
            );
            to_untap.retain(|id| !matching.contains(id) || keep.contains(id));
        }
    }
}
