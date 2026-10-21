//! Effects that apply only during a player's next turn or next combat phase ("During
//! target opponent's next turn, creatures that player controls attack Gideon Jura if
//! able.", "that creature can't attack during its controller's next turn", "Target
//! opponent skips all combat phases of their next turn.", "target creature an opponent
//! controls attacks during its controller's next combat phase if able").
//!
//! The resolving spell or ability creates the effect (CR 611.2a), but it applies only
//! during that turn (Gideon Jura's rulings: it "applies during each combat phase of the
//! affected player's next turn" and "doesn't lock in what it applies to"). The engine
//! keeps the effect waiting with the context it was created with (its targets and
//! choices) and creates it, lasting "this turn" (or until end of combat), as that turn
//! (or combat phase) begins. Only rule-modifying effects (restrictions, requirements and
//! skips, CR 613.11, 614.10) are created this way, so the later timestamp changes nothing
//! (see the interpretation `next-turn-effects-start-with-the-turn`).
//!
//! "During its controller's next turn" waits for the next turn of whoever controls the
//! object then: if it changes controllers first, the effect applies during its new
//! controller's next turn (Gideon, Battle-Forged's rulings).

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::*;
use serde::{Deserialize, Serialize};

/// Whose next turn an effect waits for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Whose {
    Player(PlayerId),
    /// Whoever controls the object as that turn begins.
    ControllerOf(ObjectId),
}

/// An effect waiting for a player's next turn or combat phase.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Scheduled {
    /// The player whose turn (combat phase) it waits for.
    pub whose: Whose,
    pub part: NextTurnPart,
    /// The resolving spell's or ability's context, with "that player"
    /// ([`PlayerRef::Iterated`]) the player it waits for.
    pub ctx: Ctx,
    pub effect: Effect,
    /// The turn it was created in, and how many combat phases that turn had begun then.
    pub created_turn: u32,
    pub created_combats: u32,
}

/// Performs [`Effect::DuringNext`]: one waiting effect for each player `who` names.
pub fn schedule(g: &mut Game, who: &PlayerRef, part: NextTurnPart, effect: &Effect, ctx: &Ctx) {
    let whose: Vec<Whose> = match who {
        PlayerRef::ControllerOf(sel) => {
            let mut c = ctx.clone();
            g.resolve_objects(sel, &mut c)
                .into_iter()
                .filter(|o| g.is_live(*o))
                .map(Whose::ControllerOf)
                .collect()
        }
        _ => g
            .eval_players(who, ctx)
            .into_iter()
            .map(Whose::Player)
            .collect(),
    };
    for w in whose {
        g.scheduled.push(Scheduled {
            whose: w,
            part,
            ctx: ctx.clone(),
            effect: effect.clone(),
            created_turn: g.turn.number,
            created_combats: g.turn.combat_phases,
        });
    }
}

/// As a turn begins: creates the effects waiting for the turn of one of its active
/// players. Effects for players who left the game are dropped (CR 800.4a).
pub fn turn_began(g: &mut Game) {
    start(g, |g, s| {
        s.part == NextTurnPart::Turn && g.turn.number > s.created_turn
    });
}

/// As a combat phase begins (its beginning of combat step, CR 507): creates the effects
/// waiting for the next combat phase of one of the turn's active players.
pub fn combat_began(g: &mut Game) {
    start(g, |g, s| {
        s.part == NextTurnPart::CombatPhase
            && (g.turn.number > s.created_turn || g.turn.combat_phases > s.created_combats)
    });
}

/// The player an effect waits for now: `None` once it can't apply any more (the player
/// left the game, CR 800.4a, or the object left the battlefield, CR 400.7).
fn waiting_for(g: &Game, w: Whose) -> Option<PlayerId> {
    let p = match w {
        Whose::Player(p) => p,
        Whose::ControllerOf(o) => {
            if !g.is_live(o) || g.obj(o).zone != crate::object::Zone::Battlefield {
                return None;
            }
            g.obj(o).controller
        }
    };
    g.players_in_game().contains(&p).then_some(p)
}

fn start(g: &mut Game, due: impl Fn(&Game, &Scheduled) -> bool) {
    if g.scheduled.is_empty() {
        return;
    }
    let actives = g.active_players();
    let mut now = Vec::new();
    for s in std::mem::take(&mut g.scheduled) {
        match waiting_for(g, s.whose) {
            None => {}
            Some(p) if actives.contains(&p) && due(g, &s) => now.push((p, s)),
            Some(_) => g.scheduled.push(s),
        }
    }
    for (p, s) in now {
        let mut ctx = s.ctx;
        ctx.iter_player = Some(p);
        g.exec(&s.effect, &mut ctx);
    }
    g.dirty = true;
}
