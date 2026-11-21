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
        self.untap_only_chosen_types(active, to_untap);
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

    /// "Each player chooses artifact, creature, or land during their untap step. That
    /// player can untap only permanents of the chosen type this step." (Storage Matrix,
    /// [`UNTAP_ONLY_CHOSEN_TYPE`]): for each such effect the active player chooses one of
    /// the card types as the untap step's untap action begins (CR 502.3); only permanents
    /// with a chosen type untap, a permanent with several card types if any of them was
    /// chosen. The choice is made before other limits on untapping apply, which then
    /// combine with it.
    fn untap_only_chosen_types(&mut self, active: PlayerId, to_untap: &mut Vec<ObjectId>) {
        let effects: Vec<(Option<ObjectId>, Vec<CardType>)> = self
            .statics
            .restrictions
            .iter()
            .map(|(s, _, r)| (Some(*s), r))
            .chain(self.rule_effects.iter().map(|e| (e.source, &e.restriction)))
            .filter_map(|(s, r)| match r {
                Restriction::Custom(n) => Some((s, untap_only_types(n)?)),
                _ => None,
            })
            .collect();
        for (source, types) in effects {
            let opts: Vec<String> = types.iter().map(|t| t.word().to_string()).collect();
            let i = self.ask_option(active, source, "Choose a type to untap", opts);
            let Some(t) = types.get(i).or(types.first()).copied() else {
                continue;
            };
            self.log(|_| format!("{active:?} chooses {} to untap", t.word()));
            to_untap.retain(|id| self.obj(*id).chars.card_types.contains(t));
        }
    }
}

/// `Restriction::Custom` name prefix: each player chooses one of the card types listed
/// after it (comma-separated words) during their untap step and can untap only
/// permanents of that type that step.
pub const UNTAP_ONLY_CHOSEN_TYPE: &str = "untap:only chosen type:";

/// The card types an [`UNTAP_ONLY_CHOSEN_TYPE`] restriction lets the player choose from.
pub fn untap_only_types(name: &str) -> Option<Vec<CardType>> {
    name.strip_prefix(UNTAP_ONLY_CHOSEN_TYPE)?
        .split(',')
        .map(CardType::from_word)
        .collect()
}
