//! The complete and exact list of a player's legal priority actions.
//!
//! The engine's own [`Game::legal_actions`] is optimistic: it checks timing, targets and
//! (optimistically) costs, but an action it lists can still turn out to be impossible once
//! the player starts it (CR 733: such an action is reversed). This module tries each
//! listed action on a copy of the game, so that every action offered to an external agent
//! can really be taken, and adds the actions the engine accepts at any time with priority
//! but doesn't list: activating mana abilities (CR 117.1d, 605.3a) and conceding
//! (CR 104.3a).

use mtg_engine::ability::AbilityKind;
use mtg_engine::agents::RandomAgent;
use mtg_engine::decision::{Action, Agent, Answer, Decision, PassiveAgent};
use mtg_engine::object::Zone;
use mtg_engine::{Game, PlayerId};

/// How many differently-seeded random choosers [`can_take`] tries after the default
/// chooser before deciding that an action can't be completed.
pub const RANDOM_TRIALS: u64 = 3;

/// A copy of `g` that can be changed without affecting `g`: its agents are replaced (the
/// copy shares the original's otherwise, and they may be busy deciding) and it records no
/// events.
pub fn sandbox(g: &Game, agent: impl Fn(PlayerId) -> Box<dyn Agent>) -> Game {
    let mut copy = g.clone();
    copy.set_agents(
        (0..g.players.len())
            .map(|i| agent(PlayerId(i as u8)))
            .collect(),
    );
    copy.event_feed.disable();
    copy.logging = false;
    copy
}

/// A chooser for trying an action: the cheapest choices where the engine's default may
/// be too ambitious (X = 0, no optional costs, as few targets as allowed but at least
/// one), the engine's default otherwise.
struct CheapestAgent;

impl Agent for CheapestAgent {
    fn decide(&mut self, _g: &Game, _p: PlayerId, d: &Decision) -> Answer {
        match d {
            Decision::ChooseX { min, .. } => Answer::Number((*min).max(0)),
            Decision::OptionalCost { repeatable, .. } => {
                if *repeatable {
                    Answer::Number(0)
                } else {
                    Answer::Bool(false)
                }
            }
            Decision::ChooseTargets {
                candidates,
                min,
                max,
                ..
            } => {
                let n = (*min).max(1).min(*max) as usize;
                if n > candidates.len() {
                    return Answer::Default;
                }
                Answer::Entities(candidates[..n].to_vec())
            }
            _ => Answer::Default,
        }
    }
}

/// Whether `p` can take `action` now and complete it: tried on a copy of the game, first
/// with the engine's default choices, then with the cheapest ones, then with a few random
/// ones.
pub fn can_take(g: &Game, p: PlayerId, action: &Action) -> bool {
    match action {
        Action::Pass | Action::Concede => return true,
        _ => {}
    }
    let mut copy = sandbox(g, |_| Box::new(PassiveAgent));
    if copy.perform_action(p, action.clone()).is_ok() {
        return true;
    }
    let mut copy = sandbox(g, |q| {
        if q == p {
            Box::new(CheapestAgent)
        } else {
            Box::new(PassiveAgent)
        }
    });
    if copy.perform_action(p, action.clone()).is_ok() {
        return true;
    }
    (1..=RANDOM_TRIALS).any(|seed| {
        let mut copy = sandbox(g, |q| Box::new(RandomAgent::new(seed * 31 + q.0 as u64)));
        copy.perform_action(p, action.clone()).is_ok()
    })
}

/// The mana abilities `p` may activate now (CR 605.3a), as priority actions.
pub fn mana_ability_actions(g: &Game, p: PlayerId) -> Vec<Action> {
    let mut out = Vec::new();
    for (i, o) in g.objects.iter().enumerate() {
        let id = mtg_engine::ObjectId(i as u32);
        if o.zone == Zone::Nowhere || !g.is_live(id) {
            continue;
        }
        for a in &o.chars.abilities {
            let AbilityKind::Activated(act) = &a.kind else {
                continue;
            };
            if act.is_mana_ability && g.can_activate(p, id, a, act) {
                out.push(Action::Activate {
                    source: id,
                    ability: a.uid,
                });
            }
        }
    }
    out
}

/// What [`priority_options`] includes besides the engine's list.
#[derive(Clone, Debug)]
pub struct PriorityOptions {
    /// Drop listed actions that can't be completed (see [`can_take`]).
    pub verify: bool,
    /// Offer activating mana abilities.
    pub mana_abilities: bool,
    /// Offer conceding.
    pub concede: bool,
}

impl Default for PriorityOptions {
    fn default() -> Self {
        PriorityOptions {
            verify: true,
            mana_abilities: true,
            concede: true,
        }
    }
}

/// The priority actions to offer `p`, given the engine's list `listed` (from
/// `Decision::Priority`): pass first, then the listed actions that can be completed, then
/// mana abilities, then conceding.
pub fn priority_options(
    g: &Game,
    p: PlayerId,
    listed: &[Action],
    opts: &PriorityOptions,
) -> Vec<Action> {
    let mut out = vec![Action::Pass];
    for a in listed {
        if matches!(a, Action::Pass | Action::Concede) || out.contains(a) {
            continue;
        }
        if !opts.verify || can_take(g, p, a) {
            out.push(a.clone());
        }
    }
    if opts.mana_abilities {
        for a in mana_ability_actions(g, p) {
            if !out.contains(&a) && (!opts.verify || can_take(g, p, &a)) {
                out.push(a);
            }
        }
    }
    if opts.concede {
        out.push(Action::Concede);
    }
    out
}
