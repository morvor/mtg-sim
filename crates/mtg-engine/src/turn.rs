//! Turn structure (CR 500–514), priority (CR 117), and the main game loop.

use crate::ability::*;
use crate::decision::{Action, Answer, Decision};
use crate::events::Event;
use crate::game::*;
use crate::object::Zone;
use crate::types::*;
use serde::{Deserialize, Serialize};

/// Steps of a turn (main phases are treated as steps without substeps).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Step {
    Untap,
    Upkeep,
    Draw,
    PrecombatMain,
    BeginningOfCombat,
    DeclareAttackers,
    DeclareBlockers,
    /// The first-strike combat damage step (CR 510.4).
    FirstStrikeDamage,
    CombatDamage,
    EndOfCombat,
    PostcombatMain,
    End,
    Cleanup,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Phase {
    Beginning,
    PrecombatMain,
    Combat,
    PostcombatMain,
    Ending,
}

impl Step {
    pub fn phase(self) -> Phase {
        match self {
            Step::Untap | Step::Upkeep | Step::Draw => Phase::Beginning,
            Step::PrecombatMain => Phase::PrecombatMain,
            Step::BeginningOfCombat
            | Step::DeclareAttackers
            | Step::DeclareBlockers
            | Step::FirstStrikeDamage
            | Step::CombatDamage
            | Step::EndOfCombat => Phase::Combat,
            Step::PostcombatMain => Phase::PostcombatMain,
            Step::End | Step::Cleanup => Phase::Ending,
        }
    }
    pub fn is_main(self) -> bool {
        matches!(self, Step::PrecombatMain | Step::PostcombatMain)
    }
    pub fn is_combat(self) -> bool {
        self.phase() == Phase::Combat
    }
    pub fn trigger_step(self) -> TriggerStep {
        match self {
            Step::Untap => TriggerStep::Untap,
            Step::Upkeep => TriggerStep::Upkeep,
            Step::Draw => TriggerStep::Draw,
            Step::PrecombatMain => TriggerStep::PrecombatMain,
            Step::BeginningOfCombat => TriggerStep::BeginningOfCombat,
            Step::DeclareAttackers => TriggerStep::DeclareAttackers,
            Step::DeclareBlockers => TriggerStep::DeclareBlockers,
            Step::FirstStrikeDamage | Step::CombatDamage => TriggerStep::CombatDamage,
            Step::EndOfCombat => TriggerStep::EndOfCombat,
            Step::PostcombatMain => TriggerStep::PostcombatMain,
            Step::End => TriggerStep::End,
            Step::Cleanup => TriggerStep::Cleanup,
        }
    }
}

/// Where we are within the current step.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stage {
    /// The step is about to begin (turn-based actions pending).
    Begin,
    /// Players are receiving priority.
    Priority,
    /// The step is ending.
    End,
    /// The game hasn't started.
    PreGame,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TurnState {
    pub number: u32,
    pub active: PlayerId,
    pub step: Step,
    pub stage: Stage,
    /// Remaining steps of this turn after the current one.
    pub schedule: Vec<Step>,
    pub priority: Option<PlayerId>,
    /// Consecutive passes with no action in between.
    pub passes: u32,
    pub main_phases: u32,
    pub combat_phases: u32,
    /// The starting player of the game.
    pub starting_player: PlayerId,
    /// Cleanup step granted priority (so another cleanup step follows, CR 514.3a).
    pub cleanup_priority: bool,
    /// Whether this is an extra turn.
    pub extra: bool,
    /// Upkeep steps seen this turn (CR 503.2).
    pub upkeeps: u32,
    /// The player whose turn it was last turn.
    pub previous_active: Option<PlayerId>,
    /// Steps that have begun this turn, in order (for combat timing windows, CR 506.8).
    #[serde(default)]
    pub step_log: Vec<Step>,
    /// (attacking player, attacked player) pairs for creatures declared as attackers this
    /// turn (CR 508.6: "has attacked [a player]").
    #[serde(default)]
    pub attacked_players: Vec<(PlayerId, PlayerId)>,
}

impl TurnState {
    pub fn new(active: PlayerId) -> TurnState {
        TurnState {
            number: 0,
            active,
            step: Step::Untap,
            stage: Stage::PreGame,
            schedule: vec![],
            priority: None,
            passes: 0,
            main_phases: 0,
            combat_phases: 0,
            starting_player: active,
            cleanup_priority: false,
            extra: false,
            upkeeps: 0,
            previous_active: None,
            step_log: vec![],
            attacked_players: vec![],
        }
    }

    pub fn default_schedule() -> Vec<Step> {
        vec![
            Step::Upkeep,
            Step::Draw,
            Step::PrecombatMain,
            Step::BeginningOfCombat,
            Step::DeclareAttackers,
            Step::DeclareBlockers,
            Step::CombatDamage,
            Step::EndOfCombat,
            Step::PostcombatMain,
            Step::End,
            Step::Cleanup,
        ]
    }
}

impl Game {
    // ------------------------------------------------------------------
    // Starting the game (CR 103)
    // ------------------------------------------------------------------

    /// Shuffles libraries, determines the starting player, draws opening hands, runs
    /// mulligans, and begins the first turn.
    pub fn start(&mut self) {
        // CR 103.1: determine the starting player. While starting the game, the starting
        // player is considered the active player (CR 101.4e).
        let starting = crate::start::choose_starting_player(self);
        self.turn.starting_player = starting;
        self.turn.active = starting;
        // CR 103.2: sideboards, companions, commanders, sticker sheets and conspiracies;
        // then effects that make a player the starting player (CR 103.1c).
        crate::start::additional_steps(self);
        let starting = self.turn.starting_player;
        // CR 607.2n: actions taken before shuffling decks to start the game.
        crate::opening_hand::before_shuffle_actions(self);
        // CR 103.3: each player shuffles their deck, which becomes their library.
        for p in self.player_ids() {
            self.shuffle_library(p);
        }
        // CR 103.3a: supplementary planar decks are shuffled too.
        crate::planechase::shuffle_planar_decks(self);
        // CR 613.7i, 613.7j: vanguard and conspiracy card timestamps.
        crate::variants::begin_game(self);
        // CR 103.4: starting life totals.
        crate::start::set_starting_life(self);
        // CR 103.5: draw opening hands, then mulligans.
        for p in self.apnap() {
            for _ in 0..self.starting_hand_size(p) {
                self.draw_card_raw(p);
            }
        }
        if !self.config.skip_mulligans {
            crate::mulligan::run_mulligans(self);
        }
        // CR 103.6: actions with cards from opening hands.
        crate::opening_hand::opening_hand_actions(self);
        // CR 103.7, 901.5: the starting plane.
        crate::planechase::set_starting_plane(self);
        self.events.clear();
        // CR 103.8: the starting player takes their first turn.
        self.begin_turn(starting, false);
    }

    /// Runs the game until it ends. Returns the result.
    pub fn run(&mut self) -> GameResult {
        if self.turn.stage == Stage::PreGame {
            self.start();
        }
        while self.result.is_none() {
            self.advance();
            if self.turn.number > self.config.max_turns
                || self.actions_taken > self.config.max_actions
            {
                self.draw_game();
            }
        }
        self.result.clone().unwrap()
    }

    /// Begins a new turn for `active`.
    pub fn begin_turn(&mut self, active: PlayerId, extra: bool) {
        self.turn.previous_active = if self.turn.number > 0 {
            Some(self.turn.active)
        } else {
            None
        };
        let last_spells = self
            .history
            .spells_cast
            .iter()
            .filter(|(p, _)| *p == self.turn.active)
            .count() as u32;
        self.spells_cast_last_turn_by_active = last_spells;
        self.turn.number += 1;
        self.turn.active = active;
        self.turn.extra = extra;
        self.turn.step = Step::Untap;
        self.turn.stage = Stage::Begin;
        self.turn.schedule = TurnState::default_schedule();
        self.turn.priority = None;
        self.turn.passes = 0;
        self.turn.main_phases = 0;
        self.turn.combat_phases = 0;
        self.turn.upkeeps = 0;
        self.turn.cleanup_priority = false;
        self.turn.step_log.clear();
        self.turn.attacked_players.clear();
        self.last_turn_history = std::mem::take(&mut self.history);
        self.turn_events.clear();
        for p in self.players.iter_mut() {
            p.lands_played_this_turn = 0;
            p.speed_increased_this_turn = false;
            p.mana_spent_this_turn = 0;
        }
        for o in self.objects.iter_mut() {
            o.activations_this_turn.clear();
            o.triggers_this_turn.clear();
        }
        // CR 302.6: permanents the active player (with shared team turns, each player on
        // the active team, CR 805.4a) has controlled continuously since the turn began are
        // no longer "summoning sick".
        let actives = self.active_players();
        let bf = self.battlefield.clone();
        for id in bf {
            if actives.contains(&self.obj(id).controller) {
                self.objects[id.0 as usize].summoning_sick = false;
            }
        }
        // Effects that last "until your next turn" end (CR 611.2b).
        for p in actives {
            self.expire_until_next_turn(p);
        }
        // Goad ends at the goading player's next turn (CR 701.15b).
        for o in self.objects.iter_mut() {
            o.goaded_by.retain(|p| *p != active);
        }
        self.log(|g| format!("--- Turn {} ({}) ---", g.turn.number, active));
        self.emit(Event::TurnBegan {
            active,
            number: self.turn.number,
        });
        self.dirty = true;
    }

    /// Advances the game by one unit: performs a step's turn-based actions, gives a
    /// player priority and performs their action, resolves the top of the stack, or
    /// moves to the next step.
    pub fn advance(&mut self) {
        if self.result.is_some() {
            return;
        }
        match self.turn.stage {
            Stage::PreGame => self.start(),
            Stage::Begin => self.begin_step(),
            Stage::Priority => self.priority_round(),
            Stage::End => self.end_step(),
        }
    }

    fn step_has_priority(&self, step: Step) -> bool {
        !matches!(step, Step::Untap | Step::Cleanup)
    }

    fn begin_step(&mut self) {
        let step = self.turn.step;
        let active = self.turn.active;
        // CR 614.10: skip effects.
        if self.should_skip_step(step) {
            self.turn.stage = Stage::End;
            return;
        }
        self.expire_effects_at_step_begin(step);
        self.turn.step_log.push(step);
        self.emit(Event::StepBegan { step, active });
        // CR 614.10b: an action a skip effect scheduled is the first thing that happens.
        crate::skip::run_step_start_actions(self);
        match step {
            Step::Untap => self.untap_step_actions(),
            Step::Upkeep => {
                self.turn.upkeeps += 1;
            }
            Step::Draw => {
                // With shared team turns, each player on the active team draws
                // (CR 805.4b).
                for p in self.active_players() {
                    self.draw_cards(p, 1);
                }
            }
            Step::PrecombatMain | Step::PostcombatMain => {
                self.turn.main_phases += 1;
                if step == Step::PrecombatMain && self.turn.main_phases == 1 {
                    self.precombat_main_actions();
                }
            }
            Step::BeginningOfCombat => {
                self.turn.combat_phases += 1;
                crate::combat::begin_combat(self);
            }
            Step::DeclareAttackers => crate::combat::declare_attackers_step(self),
            Step::DeclareBlockers => crate::combat::declare_blockers_step(self),
            Step::FirstStrikeDamage => crate::combat::combat_damage_step(self, true),
            Step::CombatDamage => {
                let first_strike_happened =
                    self.combat.as_ref().is_some_and(|c| c.first_strike_step);
                crate::combat::combat_damage_step(self, false);
                let _ = first_strike_happened;
            }
            Step::EndOfCombat => {}
            Step::End => {}
            Step::Cleanup => self.cleanup_actions(),
        }
        self.flush_events();
        if self.step_has_priority(step) {
            self.turn.stage = Stage::Priority;
            self.turn.priority = Some(active);
            self.turn.passes = 0;
        } else if step == Step::Cleanup {
            // CR 514.3a: if SBAs or triggers happen, players get priority.
            let did_something = self.settle();
            if did_something || !self.stack.is_empty() {
                self.turn.cleanup_priority = true;
                self.turn.stage = Stage::Priority;
                self.turn.priority = Some(active);
                self.turn.passes = 0;
            } else {
                self.turn.stage = Stage::End;
            }
        } else {
            self.turn.stage = Stage::End;
        }
    }

    fn priority_round(&mut self) {
        // CR 117.5: SBAs and triggers before a player receives priority.
        self.settle();
        if self.result.is_some() {
            return;
        }
        let Some(p) = self.turn.priority else {
            self.turn.stage = Stage::End;
            return;
        };
        if !self.player(p).in_game() {
            self.turn.priority = Some(self.next_player(p));
            return;
        }
        let actions = self.legal_actions(p);
        // CR 104.4b: a loop of mandatory actions is a draw.
        let forced = actions
            .iter()
            .all(|a| matches!(a, Action::Pass | Action::Concede));
        if self.check_mandatory_loop(forced) {
            return;
        }
        let answer = self.ask(
            p,
            Decision::Priority {
                actions: actions.clone(),
            },
        );
        let action = match answer {
            Answer::Action(a) => a,
            _ => Action::Pass,
        };
        self.take_action(p, action);
    }

    /// Performs a priority action for player `p` (used by the loop and by tests).
    pub fn take_action(&mut self, p: PlayerId, action: Action) {
        match action {
            Action::Pass => self.pass_priority(p),
            Action::Concede => {
                // CR 104.3a: a player can concede at any time.
                self.player_loses(p);
                self.turn.passes = 0;
                if self.turn.priority == Some(p) {
                    self.turn.priority = Some(self.next_player(p));
                }
            }
            other => {
                if self.perform_action(p, other).is_ok() {
                    // CR 117.3c: the player who acted receives priority again.
                    self.turn.passes = 0;
                    self.turn.priority = Some(p);
                } else {
                    // Illegal action: treat as pass to avoid infinite loops with bad agents.
                    self.pass_priority(p);
                }
            }
        }
    }

    /// Player `p` passes priority (CR 117.3d).
    pub fn pass_priority(&mut self, p: PlayerId) {
        self.turn.passes += 1;
        let n = self.players_in_game().len() as u32;
        if self.turn.passes >= n {
            // CR 117.4: all players passed in succession.
            self.turn.passes = 0;
            if self.stack.is_empty() {
                self.turn.stage = Stage::End;
                self.turn.priority = None;
            } else {
                self.resolve_top();
                // CR 117.3b: the active player receives priority after resolution.
                self.turn.priority = Some(self.turn.active);
            }
        } else {
            self.turn.priority = Some(self.next_player(p));
        }
    }

    fn end_step(&mut self) {
        let step = self.turn.step;
        // CR 500.5: effects lasting until end of step expire; mana empties.
        self.empty_mana_pools();
        if step == Step::DeclareAttackers && self.combat.as_ref().is_none_or(|c| !c.any_attackers) {
            // CR 508.8: if no creatures were declared as attackers or put onto the
            // battlefield attacking, skip this combat's declare blockers and combat damage
            // steps.
            while matches!(
                self.turn.schedule.first(),
                Some(Step::DeclareBlockers | Step::CombatDamage | Step::FirstStrikeDamage)
            ) {
                self.turn.schedule.remove(0);
            }
        }
        if step == Step::EndOfCombat {
            // CR 511.3 / 500.5a
            crate::combat::end_combat(self);
            self.expire_effects(|d| matches!(d, Duration::EndOfCombat));
        }
        if step == Step::Cleanup && self.turn.cleanup_priority {
            // CR 514.3a: another cleanup step.
            self.turn.cleanup_priority = false;
            self.turn.stage = Stage::Begin;
            return;
        }
        self.flush_events();
        self.next_step();
    }

    fn next_step(&mut self) {
        if self.turn.schedule.is_empty() {
            // Next turn (CR 500.7: extra turns first). Skipped turns never begin
            // (CR 614.10).
            let mut after = self.turn.active;
            for _ in 0..1000 {
                if let Some(p) = self.extra_turns.pop() {
                    if self.player(p).in_game() {
                        if crate::skip::consume_turn_skip(self, p) {
                            continue;
                        }
                        self.begin_turn(p, true);
                        return;
                    }
                }
                let next = self.next_turn_player(after);
                if crate::skip::consume_turn_skip(self, next) {
                    after = next;
                    continue;
                }
                self.begin_turn(next, false);
                return;
            }
            let next = self.next_turn_player(after);
            self.begin_turn(next, false);
            return;
        }
        let mut next = self.turn.schedule.remove(0);
        if next == Step::CombatDamage && self.dirty {
            self.recompute();
        }
        if next == Step::CombatDamage
            && self.combat.as_ref().is_some_and(|c| !c.first_strike_step)
            && crate::combat::any_first_strike(self)
        {
            // CR 510.4: first-strike damage step, then a regular one.
            self.turn.schedule.insert(0, Step::CombatDamage);
            next = Step::FirstStrikeDamage;
        }
        self.turn.step = next;
        self.turn.stage = Stage::Begin;
        self.turn.priority = None;
        self.turn.passes = 0;
        // Static abilities whose conditions depend on the step are re-evaluated.
        self.dirty = true;
    }

    /// Adds an additional combat phase (and main phase) after the current phase (CR 500.8).
    pub fn add_extra_combat(&mut self, with_main: bool) {
        let mut extra = vec![
            Step::BeginningOfCombat,
            Step::DeclareAttackers,
            Step::DeclareBlockers,
            Step::CombatDamage,
            Step::EndOfCombat,
        ];
        if with_main {
            extra.push(Step::PostcombatMain);
        }
        // Insert after the current phase.
        let cur_phase = self.turn.step.phase();
        let pos = self
            .turn
            .schedule
            .iter()
            .position(|s| s.phase() != cur_phase)
            .unwrap_or(self.turn.schedule.len());
        for (i, s) in extra.into_iter().enumerate() {
            self.turn.schedule.insert(pos + i, s);
        }
    }

    fn should_skip_step(&mut self, step: Step) -> bool {
        let active = self.turn.active;
        // CR 103.8: in a two-player game the player who plays first skips the draw step of
        // their first turn (CR 103.8a); in other multiplayer games no one does
        // (CR 103.8c).
        if step == Step::Draw && self.first_turn_draw_skipped(active) {
            return true;
        }
        let kind = match step {
            Step::Untap => StepKind::Untap,
            Step::Upkeep => StepKind::Upkeep,
            Step::Draw => StepKind::Draw,
            Step::BeginningOfCombat => StepKind::Combat,
            Step::End => StepKind::End,
            _ => return false,
        };
        // CR 614.1b: static "skip" effects replace the step with nothing.
        if crate::skip::static_skip(self, kind, active) {
            self.players[active.idx()].skips.push(kind);
        }
        if let Some(i) = self.players[active.idx()]
            .skips
            .iter()
            .position(|k| *k == kind)
        {
            self.players[active.idx()].skips.remove(i);
            if kind == StepKind::Combat {
                // Skip the whole combat phase (only this one, not additional combat phases
                // later in the turn).
                while let Some(s) = self.turn.schedule.first().copied() {
                    if !s.is_combat() {
                        break;
                    }
                    self.turn.schedule.remove(0);
                    if s == Step::EndOfCombat {
                        break;
                    }
                }
            }
            return true;
        }
        false
    }

    /// Whether `active` skips the draw step of this turn because it's the starting
    /// player's first turn (CR 103.8a–c).
    pub fn first_turn_draw_skipped(&self, active: PlayerId) -> bool {
        // CR 103.8b: in Two-Headed Giant, the team who plays first skips it.
        let skips = self
            .config
            .starting_player_skips_draw
            .unwrap_or(self.is_two_player() || self.config.variant == Variant::TwoHeadedGiant);
        skips && active == self.turn.starting_player && self.turn.number == 1 && !self.turn.extra
    }

    fn untap_step_actions(&mut self) {
        let active = self.turn.active;
        // CR 502.1: phasing.
        for p in self.active_players() {
            crate::keyword_impls::phasing_untap_step(self, p);
        }
        // CR 502.2: day/night.
        if let Some(is_day) = self.day {
            if self.turn.number > 1 {
                let n = self.spells_cast_last_turn_by_active;
                if is_day && n == 0 {
                    self.set_day(false);
                } else if !is_day && n >= 2 {
                    self.set_day(true);
                }
            }
        }
        // CR 502.3: untap (each player on the active team, CR 805.4a).
        let _ = active;
        for active in self.active_players() {
            self.recompute();
            let mut to_untap: Vec<ObjectId> = self
                .permanents()
                .filter(|o| o.controller == active && o.tapped)
                .map(|o| o.id)
                .filter(|id| !self.doesnt_untap(*id))
                .collect();
            self.limit_untaps(active, &mut to_untap);
            for id in to_untap {
                self.untap(id);
            }
            // CR 701.43a: exertion lasts until its controller's next untap step.
            for id in self.battlefield.clone() {
                if self.obj(id).controller == active {
                    self.objects[id.0 as usize].exerted = false;
                }
            }
            self.expire_through_next_untap_step(active);
        }
    }

    /// "Doesn't untap during its controller's next untap step": that untap step has now
    /// passed for the permanents the active player controls (and the effect no longer
    /// applies to objects that left the battlefield, CR 400.7).
    fn expire_through_next_untap_step(&mut self, active: PlayerId) {
        let objects = &self.objects;
        let battlefield = &self.battlefield;
        for e in self.rule_effects.iter_mut() {
            if !matches!(e.duration, Duration::ThroughNextUntapStep) {
                continue;
            }
            if let Some(v) = e.objects.as_mut() {
                v.retain(|o| battlefield.contains(o) && objects[o.0 as usize].controller != active);
            }
        }
        self.rule_effects.retain(|e| {
            !matches!(e.duration, Duration::ThroughNextUntapStep)
                || e.objects.as_ref().is_some_and(|v| !v.is_empty())
        });
        self.dirty = true;
    }

    pub fn set_day(&mut self, is_day: bool) {
        if self.day != Some(is_day) {
            let had_designation = self.day.is_some();
            self.day = Some(is_day);
            // CR 731.1a: "day becomes night"/"night becomes day" means losing one
            // designation and gaining the other; the game first becoming day or night
            // from neither isn't such a change.
            if had_designation {
                self.emit(Event::DayNightChanged { is_day });
            }
            crate::keyword_impls::day_night_changed(self);
        }
    }

    fn precombat_main_actions(&mut self) {
        let active = self.turn.active;
        // CR 505.3: archenemy sets a scheme in motion.
        if self.config.variant == Variant::Archenemy {
            crate::variants::archenemy_main_phase(self, active);
        }
        // CR 505.4 / 714.3b: lore counters on Sagas.
        self.recompute();
        let sagas: Vec<ObjectId> = self
            .permanents()
            .filter(|o| {
                o.controller == active
                    && o.chars.has_subtype("Saga")
                    && crate::saga::has_chapters(o)
            })
            .map(|o| o.id)
            .collect();
        for s in sagas {
            self.add_counters(Entity::Object(s), counters::LORE, 1, None);
        }
        // CR 505.5: attractions.
        crate::variants::roll_to_visit_attractions(self, active);
    }

    fn cleanup_actions(&mut self) {
        // CR 514.1: discard to maximum hand size (each player on the active team).
        self.recompute();
        for active in self.active_players() {
            let Some(max) = self.player(active).max_hand_size else {
                continue;
            };
            let hand = self.player(active).hand.clone();
            let excess = hand.len() as i32 - max.max(0);
            if excess > 0 {
                let chosen = self.ask_objects(
                    active,
                    None,
                    "Discard down to maximum hand size",
                    hand,
                    excess as u32,
                    excess as u32,
                );
                for c in chosen {
                    self.discard(active, c, None);
                }
            }
        }
        // CR 514.2: remove damage; end "until end of turn" effects.
        for id in self.battlefield.clone() {
            let o = &mut self.objects[id.0 as usize];
            o.damage = 0;
            o.deathtouch_damage = false;
        }
        self.expire_effects(|d| matches!(d, Duration::EndOfTurn | Duration::ThisTurn));
        // "Until end of turn, whenever …" delayed triggered abilities (CR 603.7b).
        self.delayed_triggers
            .retain(|d| !matches!(d.trigger, crate::ability::TriggerCond::ThisTurn(_)));
        self.dirty = true;
    }

    fn empty_mana_pools(&mut self) {
        for p in self.players.iter_mut() {
            p.mana_pool.empty();
        }
    }

    /// Removes continuous effects whose duration matches.
    pub fn expire_effects(&mut self, pred: impl Fn(&Duration) -> bool) {
        self.effects.retain(|e| !pred(&e.duration));
        self.rule_effects.retain(|e| !pred(&e.duration));
        self.player_effects.retain(|e| !pred(&e.duration));
        self.replacements.retain(|e| !pred(&e.duration));
        self.dirty = true;
    }

    fn expire_until_next_turn(&mut self, active: PlayerId) {
        let until =
            |d: &Duration, c: PlayerId| matches!(d, Duration::UntilYourNextTurn) && c == active;
        self.effects.retain(|e| !until(&e.duration, e.controller));
        self.rule_effects
            .retain(|e| !until(&e.duration, e.controller));
        self.player_effects
            .retain(|e| !until(&e.duration, e.controller));
        self.replacements
            .retain(|e| !until(&e.duration, e.controller));
        // "until the end of your next turn" becomes "until end of turn" once that turn starts.
        let turn = self.turn.number;
        for e in self.effects.iter_mut() {
            if matches!(e.duration, Duration::UntilEndOfYourNextTurn)
                && e.controller == active
                && e.created_turn < turn
            {
                e.duration = Duration::EndOfTurn;
            }
        }
        for e in self.rule_effects.iter_mut() {
            if matches!(e.duration, Duration::UntilEndOfYourNextTurn) && e.controller == active {
                e.duration = Duration::EndOfTurn;
            }
        }
        self.dirty = true;
    }

    fn expire_effects_at_step_begin(&mut self, _step: Step) {}

    // ------------------------------------------------------------------
    // Test/simulation conveniences
    // ------------------------------------------------------------------

    /// Advances until the predicate holds or the game ends. Returns false if the game
    /// ended or `max` iterations passed first.
    pub fn run_until(&mut self, max: usize, mut pred: impl FnMut(&Game) -> bool) -> bool {
        for _ in 0..max {
            if pred(self) {
                return true;
            }
            if self.result.is_some() {
                return false;
            }
            self.advance();
        }
        pred(self)
    }

    /// Whether the current step is a main phase of the active player with an empty stack
    /// (sorcery timing, CR 307.1).
    pub fn is_sorcery_timing(&self, p: PlayerId) -> bool {
        self.is_active_player(p) && self.turn.step.is_main() && self.stack.is_empty()
    }

    /// Checks whether the zone is a hand/library for hidden info purposes.
    pub fn is_hidden_zone(zone: Zone) -> bool {
        !zone.is_public()
    }
}
