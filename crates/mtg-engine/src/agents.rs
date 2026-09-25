//! Built-in agents for simulations.

use crate::decision::*;
use crate::game::Game;
use crate::types::*;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

/// Makes uniformly random legal choices, with a mild bias toward doing something
/// (playing lands, casting spells, attacking) over passing.
pub struct RandomAgent {
    rng: ChaCha8Rng,
    /// Probability of passing when other actions are available.
    pub pass_bias: f64,
}

impl RandomAgent {
    pub fn new(seed: u64) -> Self {
        RandomAgent {
            rng: ChaCha8Rng::seed_from_u64(seed),
            pass_bias: 0.2,
        }
    }
}

impl Agent for RandomAgent {
    fn name(&self) -> &str {
        "random"
    }

    fn decide(&mut self, _g: &Game, _p: PlayerId, d: &Decision) -> Answer {
        match d {
            Decision::Priority { actions } => {
                let non_pass: Vec<&Action> = actions
                    .iter()
                    .filter(|a| !matches!(a, Action::Pass | Action::Concede))
                    .collect();
                if non_pass.is_empty() || self.rng.gen_bool(self.pass_bias) {
                    return Answer::Action(Action::Pass);
                }
                // Prefer land drops.
                if let Some(l) = non_pass
                    .iter()
                    .find(|a| matches!(a, Action::PlayLand { .. }))
                {
                    return Answer::Action((*l).clone());
                }
                Answer::Action((*non_pass.choose(&mut self.rng).unwrap()).clone())
            }
            Decision::Mulligan { .. } => Answer::Bool(false),
            Decision::DeclareAttackers { options } => {
                let mut v = Vec::new();
                for (a, ts) in options {
                    if self.rng.gen_bool(0.5) {
                        if let Some(t) = ts.choose(&mut self.rng) {
                            v.push((*a, *t));
                        }
                    }
                }
                Answer::Attackers(v)
            }
            Decision::DeclareBlockers { options } => {
                let mut v = Vec::new();
                for (b, atts) in options {
                    if self.rng.gen_bool(0.4) {
                        if let Some(a) = atts.choose(&mut self.rng) {
                            v.push((*b, *a));
                        }
                    }
                }
                Answer::Blockers(v)
            }
            Decision::ChooseTargets {
                candidates,
                min,
                max,
                ..
            } => {
                let mut c = candidates.clone();
                c.shuffle(&mut self.rng);
                let n = if *max > *min {
                    self.rng.gen_range(*min..=*max)
                } else {
                    *min
                };
                Answer::Entities(c.into_iter().take(n.max(*min) as usize).collect())
            }
            Decision::ChooseEntities {
                candidates,
                min,
                max,
                ..
            } => {
                let mut c = candidates.clone();
                c.shuffle(&mut self.rng);
                let n = if *max > *min {
                    self.rng.gen_range(*min..=*max)
                } else {
                    *min
                };
                Answer::Entities(c.into_iter().take(n as usize).collect())
            }
            Decision::YesNo { .. } => Answer::Bool(self.rng.gen_bool(0.5)),
            Decision::ChooseOption { options, .. } => {
                Answer::Index(self.rng.gen_range(0..options.len().max(1)))
            }
            Decision::ChooseModes { modes, min, .. } => {
                let mut idx: Vec<usize> = (0..modes.len()).collect();
                idx.shuffle(&mut self.rng);
                Answer::Indices(idx.into_iter().take((*min).max(1) as usize).collect())
            }
            Decision::ChooseX { max, .. } => Answer::Number(if *max > 0 {
                self.rng.gen_range(0..=*max)
            } else {
                0
            }),
            _ => Answer::Default,
        }
    }
}
