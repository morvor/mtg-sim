//! Controlling another player (CR 723): Mindslaver, Worst Fears, Sorin Markov, Emrakul,
//! the Promised End ("control target player during that player's next turn"), Secret of
//! Bloodbending ("during their next combat phase") and Opposition Agent ("while they're
//! searching their libraries").
//!
//! * Decisions of a controlled player are made by the agent of the player controlling
//!   them ([`decider`], used by `Game::ask`, CR 723.5); only control of the player
//!   changes: objects keep their controllers and the controlled player stays the active
//!   player (CR 723.3). The controller keeps making their own decisions (CR 723.8).
//! * The controller can't make the controlled player concede (CR 723.6) or make
//!   decisions called for by the tournament rules ([`own_decisions`], CR 723.5b), and
//!   can't see or choose cards outside the game (CR 723.4).
//! * "During that player's next turn" applies to the next turn the player actually takes
//!   and lasts until the beginning of the following turn (CR 723.1, 723.1b); effects
//!   affecting the same player overwrite each other (CR 723.1a). An effect may give a
//!   player control of themselves (CR 723.9).

use crate::ability::*;
use crate::decision::{Action, Answer, Decision};
use crate::eval::Ctx;
use crate::game::{Game, Variant};
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;
use crate::turn::Step;
use crate::types::*;
use smol_str::SmolStr;

/// For how long a player-controlling effect gives control of the player.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlSpan {
    /// "During that player's next turn" (CR 723.1).
    NextTurn,
    /// "During their next combat phase".
    NextCombatPhase,
}

/// A player-controlling effect.
#[derive(Clone, Debug)]
pub struct ControlEffect {
    /// The player who controls `player`.
    pub controller: PlayerId,
    /// The controlled player.
    pub player: PlayerId,
    /// When the effect was created (the last one created wins, CR 723.1a).
    pub timestamp: Timestamp,
    pub span: ControlSpan,
    /// "After that turn, that player takes an extra turn" (Emrakul, the Promised End).
    pub extra_turn_after: bool,
}

/// Player-controlling state, in `Game::player_control`.
#[derive(Clone, Debug, Default)]
pub struct PlayerControlState {
    /// Effects waiting for the affected player's next turn or combat phase (CR 723.1b).
    pub pending: Vec<ControlEffect>,
    /// Effects currently giving control of a player.
    pub active: Vec<ControlEffect>,
    /// While positive, each player makes their own decisions: decisions called for by the
    /// tournament rules (CR 723.5b).
    pub own_decisions: u32,
    /// The player searching their own library right now (for "while they're searching
    /// their libraries").
    pub searching: Vec<PlayerId>,
}

/// `StaticEffect::Custom` name: "You control your opponents while they're searching
/// their libraries." (Opposition Agent, CR 723.2).
pub const CONTROL_WHILE_SEARCHING: &str = "control opponents while searching";
/// `Effect::Custom` prefix: "player control:<turn|combat>:<target slot>[:extra]".
pub const CONTROL_EFFECT: &str = "player control:";
/// `Condition::Custom` name: "this spell's additional cost was paid" (an optional
/// additional cost of the spell whose ability this is was paid).
pub const ADDITIONAL_COST_PAID: &str = "this spell's additional cost was paid";

/// The effect "you control the players in target slot `slot` during their next turn (or
/// combat phase)".
pub fn control_effect(span: ControlSpan, slot: u8, extra_turn_after: bool) -> Effect {
    let span = match span {
        ControlSpan::NextTurn => "turn",
        ControlSpan::NextCombatPhase => "combat",
    };
    let extra = if extra_turn_after { ":extra" } else { "" };
    Effect::Custom(SmolStr::new(format!(
        "{CONTROL_EFFECT}{span}:{slot}{extra}"
    )))
}

/// Parses a [`control_effect`] name: (span, slot, extra turn after).
pub fn parse_control_effect(name: &str) -> Option<(ControlSpan, u8, bool)> {
    let rest = name.strip_prefix(CONTROL_EFFECT)?;
    let mut parts = rest.split(':');
    let span = match parts.next()? {
        "turn" => ControlSpan::NextTurn,
        "combat" => ControlSpan::NextCombatPhase,
        _ => return None,
    };
    let slot = parts.next()?.parse().ok()?;
    let extra = parts.next() == Some("extra");
    Some((span, slot, extra))
}

/// `controller` gains control of `player` during `player`'s next turn or combat phase.
/// In Two-Headed Giant, gaining control of a player gains control of each player on that
/// team.
pub fn gain_control(
    g: &mut Game,
    controller: PlayerId,
    player: PlayerId,
    span: ControlSpan,
    extra_turn_after: bool,
) {
    let players = if g.config.variant == Variant::TwoHeadedGiant {
        g.team_members(player)
    } else {
        vec![player]
    };
    for p in players {
        let timestamp = g.new_timestamp();
        g.log(|_| format!("{controller} will control {p}"));
        g.player_control.pending.push(ControlEffect {
            controller,
            player: p,
            timestamp,
            span,
            extra_turn_after: extra_turn_after && p == player,
        });
    }
}

/// Starts the waiting effects of `span` for the players in `players`: of several
/// effects affecting the same player, the last one created is the one that works
/// (CR 723.1a); the others are overwritten.
fn start(g: &mut Game, players: &[PlayerId], span: ControlSpan) {
    for &p in players {
        let mine: Vec<ControlEffect> = g
            .player_control
            .pending
            .iter()
            .filter(|e| e.player == p && e.span == span)
            .cloned()
            .collect();
        if mine.is_empty() {
            continue;
        }
        g.player_control
            .pending
            .retain(|e| !(e.player == p && e.span == span));
        let e = mine.into_iter().max_by_key(|e| e.timestamp).unwrap();
        if e.extra_turn_after {
            // "After that turn, that player takes an extra turn."
            g.extra_turns.push(p);
        }
        g.log(|_| format!("{} controls {p}", e.controller));
        g.player_control.active.retain(|a| a.player != p);
        g.player_control.active.push(e);
    }
}

/// A turn began (hook in `Game::begin_turn`): control effects for the previous turn end
/// ("the effect doesn't end until the beginning of the next turn", CR 723.1), and the
/// waiting effects for the players taking this turn start (a skipped turn never begins,
/// so they keep waiting, CR 723.1b).
pub fn turn_began(g: &mut Game) {
    g.player_control.active.clear();
    let actives = g.active_players();
    start(g, &actives, ControlSpan::NextTurn);
}

/// A step began (hook in `Game::begin_step`): control during a combat phase starts as the
/// player's combat phase begins and ends when it's over.
pub fn step_began(g: &mut Game, step: Step) {
    if !step.is_combat() {
        g.player_control
            .active
            .retain(|e| e.span != ControlSpan::NextCombatPhase);
    } else if step == Step::BeginningOfCombat {
        let actives = g.active_players();
        start(g, &actives, ControlSpan::NextCombatPhase);
    }
}

/// The permanents whose "You control your opponents while they're searching their
/// libraries" applies to `p`, most recently entered last.
fn search_controller(g: &Game, p: PlayerId) -> Option<PlayerId> {
    if !g.player_control.searching.contains(&p) {
        return None;
    }
    g.statics
        .customs
        .iter()
        .filter(|(_, c, n)| {
            n.as_str() == CONTROL_WHILE_SEARCHING
                && g.are_opponents(*c, p)
                && g.player(*c).in_game()
        })
        .max_by_key(|(s, _, _)| g.obj(*s).timestamp)
        .map(|(_, c, _)| *c)
}

/// The player who makes `p`'s decisions: the player controlling `p` (CR 723.5), or `p`.
pub fn decider(g: &Game, p: PlayerId) -> PlayerId {
    if g.player_control.own_decisions > 0 {
        return p;
    }
    // Opposition Agent: while they're searching their libraries.
    if let Some(c) = search_controller(g, p) {
        return c;
    }
    g.player_control
        .active
        .iter()
        .filter(|e| e.player == p && g.player(e.controller).in_game())
        .max_by_key(|e| e.timestamp)
        .map(|e| e.controller)
        .unwrap_or(p)
}

/// Whether `p` is controlled by another player.
pub fn is_controlled(g: &Game, p: PlayerId) -> bool {
    decider(g, p) != p
}

/// The decision as it's presented to the player controlling the player it's for: that
/// player can't be made to concede (CR 723.6), and cards outside the game aren't visible
/// to the controller (CR 723.4).
pub fn for_controller(g: &Game, decision: Decision) -> Decision {
    match decision {
        Decision::Priority { actions } => Decision::Priority {
            actions: actions
                .into_iter()
                .filter(|a| !matches!(a, Action::Concede))
                .collect(),
        },
        Decision::ChooseEntities {
            source,
            prompt,
            candidates,
            min,
            max,
        } => {
            let visible: Vec<Entity> = candidates
                .into_iter()
                .filter(|e| !is_outside_the_game(g, *e))
                .collect();
            let n = visible.len() as u32;
            Decision::ChooseEntities {
                source,
                prompt,
                candidates: visible,
                min: min.min(n),
                max: max.min(n),
            }
        }
        d => d,
    }
}

/// Checks the controller's answer for the controlled player: making them concede isn't
/// allowed (CR 723.6), so such an answer is ignored.
pub fn check_answer(answer: Answer) -> Answer {
    match answer {
        Answer::Action(Action::Concede) => Answer::Default,
        a => a,
    }
}

fn is_outside_the_game(g: &Game, e: Entity) -> bool {
    matches!(e, Entity::Object(o) if matches!(g.obj(o).zone, Zone::Outside(_)))
}

/// Removes the cards outside the game from what `p` may choose while `p` is controlled
/// by another player: "If an effect instructs that player to choose a card from outside
/// the game, you can't have that player choose any card" (CR 723.4).
pub fn visible_choices(g: &Game, p: PlayerId, candidates: &mut Vec<ObjectId>) {
    if is_controlled(g, p) {
        candidates.retain(|o| !matches!(g.obj(*o).zone, Zone::Outside(_)));
    }
}

/// Whether `viewer` may see the object `id`: what a player may look at is also visible
/// to the player controlling them, except cards outside the game (CR 723.4).
pub fn can_see(g: &Game, viewer: PlayerId, id: ObjectId) -> bool {
    if crate::facedown::can_look_at(g, viewer, id) {
        return true;
    }
    if matches!(g.obj(id).zone, Zone::Outside(_)) {
        return g.obj(id).owner == viewer;
    }
    g.players.iter().any(|q| {
        q.id != viewer && decider(g, q.id) == viewer && crate::facedown::can_look_at(g, q.id, id)
    })
}

/// Runs `f` with each player making their own decisions: for decisions called for by the
/// tournament rules, which the controller of a player can't make (CR 723.5b).
pub fn own_decisions<T>(g: &mut Game, f: impl FnOnce(&mut Game) -> T) -> T {
    g.player_control.own_decisions += 1;
    let r = f(g);
    g.player_control.own_decisions -= 1;
    r
}

/// Runs `f` while `p` searches their own library.
pub fn while_searching<T>(g: &mut Game, p: PlayerId, f: impl FnOnce(&mut Game) -> T) -> T {
    g.player_control.searching.push(p);
    let r = f(g);
    g.player_control.searching.pop();
    r
}

struct PlayerControlRules;

impl KeywordRules for PlayerControlRules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some((span, slot, extra)) = parse_control_effect(name) else {
            return false;
        };
        let players: Vec<PlayerId> = ctx
            .targets
            .get(slot as usize)
            .map(|v| v.iter().filter_map(|e| e.player()).collect())
            .unwrap_or_default();
        for p in players {
            gain_control(g, ctx.controller, p, span, extra);
        }
        true
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != ADDITIONAL_COST_PAID {
            return None;
        }
        let paid = g
            .cast_info(ctx)
            .map(|c| !c.paid.is_empty())
            .unwrap_or(false);
        Some(paid)
    }
}

inventory::submit! { KeywordRegistration(&PlayerControlRules) }
