//! "That combat": the additional combat phase an instruction just added (CR 500.8, 505.1a).
//!
//! * "After this phase, there is an additional combat phase. Only land creatures can attack
//!   during that combat phase." (Bumi, Unleashed), "... At the beginning of that combat,
//!   untap all creatures you control." (Moraug, Fury of Akoum): `Effect::AddTurnParts`
//!   records which combat phase of the turn the added one will be (ctx number
//!   [`ADDED_COMBAT`]); [`NextTurnPart::AddedCombatPhase`] effects wait for exactly that
//!   combat phase (see `next_turn.rs`).
//! * "After the second main phase this turn, there's an additional combat phase followed
//!   by an additional main phase." (World at War), "After the first combat phase this
//!   turn, there's an additional combat phase." (Swinging Ship): `Effect::Custom`
//!   ([`after_nth`]) adds the phases after that phase of the turn, if it hasn't ended yet
//!   (CR 500.8), and records the added combat phase likewise.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::turn::Step;

/// Ctx number: which combat phase of the turn (1 = the first) the additional combat phase
/// the last instruction added is.
pub const ADDED_COMBAT: Var = vars::USER + 13372;

/// `Effect::Custom` prefix: "combat4:after:<main|combat>:<n>:<parts>", parts being
/// "combat" or "combat+main".
const AFTER_PREFIX: &str = "combat4:after:";

/// The `Effect::Custom` adding `parts` after the `nth` main (`main`) or combat phase of
/// the turn; `None` for parts it doesn't add.
pub fn after_nth(main: bool, nth: u32, parts: &[TurnPart]) -> Option<String> {
    let p = match parts {
        [TurnPart::CombatPhase] => "combat",
        [TurnPart::CombatPhase, TurnPart::MainPhase] => "combat+main",
        _ => return None,
    };
    Some(format!(
        "{AFTER_PREFIX}{}:{nth}:{p}",
        if main { "main" } else { "combat" }
    ))
}

pub fn parse_after(name: &str) -> Option<(bool, u32, Vec<TurnPart>)> {
    let r = name.strip_prefix(AFTER_PREFIX)?;
    let mut it = r.split(':');
    let main = match it.next()? {
        "main" => true,
        "combat" => false,
        _ => return None,
    };
    let nth: u32 = it.next()?.parse().ok()?;
    let parts = match it.next()? {
        "combat" => vec![TurnPart::CombatPhase],
        "combat+main" => vec![TurnPart::CombatPhase, TurnPart::MainPhase],
        _ => return None,
    };
    Some((main, nth, parts))
}

/// Whether `s` (following `prev`) begins a new phase in a turn's schedule.
fn starts_phase(s: Step, prev: Step) -> bool {
    s.phase() != prev.phase() || s.is_main() || matches!(s, Step::Untap | Step::BeginningOfCombat) || s < prev
}

/// Index in the schedule of the first step after the phase the step at `i` (or, for
/// `None`, the current step) belongs to.
fn phase_end(g: &Game, i: Option<usize>) -> usize {
    let (mut prev, from) = match i {
        Some(i) => (g.turn.schedule[i], i + 1),
        None => (g.turn.step, 0),
    };
    for (j, s) in g.turn.schedule.iter().enumerate().skip(from) {
        if starts_phase(*s, prev) {
            return j;
        }
        prev = *s;
    }
    g.turn.schedule.len()
}

/// Which combat phase of the turn a combat phase inserted into the schedule at `pos` is.
pub fn combat_number_at(g: &Game, pos: usize) -> u32 {
    let before = g.turn.schedule[..pos.min(g.turn.schedule.len())]
        .iter()
        .filter(|s| **s == Step::BeginningOfCombat)
        .count() as u32;
    g.turn.combat_phases + before + 1
}

/// The combat phase of the turn an additional combat phase added after the current phase
/// (`after_phase`) or step will be.
pub fn added_after_current(g: &Game, after_phase: bool) -> u32 {
    let pos = if after_phase { phase_end(g, None) } else { 0 };
    combat_number_at(g, pos)
}

/// Adds `parts` after the `nth` main or combat phase of the turn, unless it has ended.
/// Returns the number of the added combat phase.
fn add_after_nth(g: &mut Game, main: bool, nth: u32, parts: &[TurnPart]) -> Option<u32> {
    let begun = if main {
        g.turn.main_phases
    } else {
        g.turn.combat_phases
    };
    let is_kind = |s: Step| {
        if main {
            s.is_main()
        } else {
            s == Step::BeginningOfCombat
        }
    };
    let in_kind = if main {
        g.turn.step.is_main()
    } else {
        g.turn.step.phase() == crate::turn::Phase::Combat
    };
    let pos = if begun == nth && in_kind {
        // During that phase: after this phase.
        phase_end(g, None)
    } else if begun < nth {
        let k = (nth - begun) as usize;
        let i = g
            .turn
            .schedule
            .iter()
            .enumerate()
            .filter(|(_, s)| is_kind(**s))
            .nth(k - 1)
            .map(|(i, _)| i)?;
        phase_end(g, Some(i))
    } else {
        // That phase is over (or never came): nothing is added.
        return None;
    };
    let n = combat_number_at(g, pos);
    let mut steps = Vec::new();
    for p in parts {
        match p {
            TurnPart::CombatPhase => steps.extend([
                Step::BeginningOfCombat,
                Step::DeclareAttackers,
                Step::DeclareBlockers,
                Step::CombatDamage,
                Step::EndOfCombat,
            ]),
            TurnPart::MainPhase => steps.push(Step::PostcombatMain),
            _ => return None,
        }
    }
    for (j, s) in steps.into_iter().enumerate() {
        g.turn.schedule.insert(pos + j, s);
    }
    Some(n)
}

pub struct AddedCombatPhases;

impl KeywordRules for AddedCombatPhases {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some((main, nth, parts)) = parse_after(name) else {
            return false;
        };
        // CR 500.10a: only the active player's turn gets phases (the effect's controller
        // is the one who'd get them, "this turn" being their turn).
        if !g.is_active_player(ctx.controller) {
            ctx.nums.remove(&ADDED_COMBAT);
            return true;
        }
        match add_after_nth(g, main, nth, &parts) {
            Some(n) => {
                ctx.nums.insert(ADDED_COMBAT, n as i64);
            }
            None => {
                ctx.nums.remove(&ADDED_COMBAT);
            }
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&AddedCombatPhases) }
