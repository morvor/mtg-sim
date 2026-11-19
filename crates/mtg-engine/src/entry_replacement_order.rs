//! The order in which the replacement effects that modify how several permanents enter
//! the battlefield at the same time are applied (CR 616.1, 616.1e–g, 614.12).
//!
//! Each permanent's own effects ("As this creature enters, ...", "This creature enters
//! with ...") are applied one at a time, in the order their controller chooses: the CR's
//! example of two Voice of All tokens created at once ("the effects of the two Voice of
//! All tokens may be applied in either order", 616.1g) and Thief of Blood's ruling ("If
//! more than one Thief of Blood enters the battlefield at the same time, you can apply
//! the replacement effect of any of them.") give that choice to the player, whichever
//! permanent each effect modifies. Players choose in APNAP order (616.1), each for the
//! permanents that would enter under their control (`entering-would-be-controller-chooses`):
//! `Game::move_objects` already handles the moves player by player.
//!
//! The order matters only when applying one permanent's effects can change what another's
//! do: one of them acts on the game as it's applied (Thief of Blood removes every
//! counter, Sutured Ghoul exiles cards from a graveyard, Ixidron turns creatures face
//! down) and another one reads the game or acts on it too (Golgari Grave-Troll counts
//! the creature cards in its owner's graveyard, Clone chooses a creature to copy, a
//! second Thief of Blood). Choices, conditions and life or energy payments (paid once all
//! the choices are made, CR 614.12b, `entry_costs.rs`) change nothing as they're made, so
//! permanents with only such effects, or only effects that read the game, are applied in
//! the order of the moves without a question. When the order can matter, the player is
//! asked, before each of those permanents' effects are applied, which of the remaining
//! ones comes next (616.1f: "this process is repeated"), with the next one in the order
//! of the moves as the default (interpretation `simultaneous-entries-ordered-by-controller`).

use crate::ability::{CostPart, Effect, ReplacementAction, Value};
use crate::decision::{Answer, Decision};
use crate::game::Game;
use crate::object::Zone;
use crate::replacement::{MoveEv, ReplEvent};
use crate::types::PlayerId;

/// What applying a replacement effect that modifies how a permanent enters does besides
/// modifying that permanent's entering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Reach {
    /// Nothing that depends on the game or changes it: "enters tapped", "enters with two
    /// +1/+1 counters on it".
    Plain,
    /// Reads the game or asks for a choice, but changes nothing as it's applied: "enters
    /// with a +1/+1 counter for each creature card in your graveyard", "you may have it
    /// enter as a copy of any creature on the battlefield", "choose a color", "enters
    /// tapped unless a player has 13 or less life", "you may pay 2 life" (paid later).
    Reads,
    /// Changes the game as it's applied: removes counters, exiles or sacrifices objects,
    /// returns a permanent to its owner's hand, turns permanents face down.
    Acts,
}

/// How far an effect performed while a permanent enters reaches (see [`Reach`]). Unknown
/// effects act: asking needlessly costs a question, not asking when it matters costs the
/// player a choice.
fn effect_reach(e: &Effect) -> Reach {
    match e {
        Effect::Noop
        | Effect::EnterTapped
        | Effect::EnterPrepared
        | Effect::EnterCopyExceptions(_)
        | Effect::EnterAs(_) => Reach::Plain,
        Effect::EnterWithCounters { n, .. } => value_reach(n),
        Effect::Seq(v) => v.iter().map(effect_reach).max().unwrap_or(Reach::Plain),
        Effect::If {
            then, otherwise, ..
        } => effect_reach(then)
            .max(effect_reach(otherwise))
            .max(Reach::Reads),
        Effect::May { effect, .. } => effect_reach(effect).max(Reach::Reads),
        Effect::ChooseOne { options, .. } => options
            .iter()
            .map(|(_, e)| effect_reach(e))
            .max()
            .unwrap_or(Reach::Plain)
            .max(Reach::Reads),
        // Life and energy are paid once every entering permanent's choices are made
        // (CR 614.12b); revealing a card changes nothing. Other costs are paid as they're
        // chosen (`entry_costs.rs`).
        Effect::PayOptional {
            cost,
            then,
            otherwise,
            ..
        } => {
            let deferred = cost.mana.is_none()
                && cost.parts.iter().all(|p| {
                    matches!(
                        p,
                        CostPart::PayLife(_)
                            | CostPart::PayEnergy(_)
                            | CostPart::RevealFromHand { .. }
                    )
                });
            let own = if deferred { Reach::Reads } else { Reach::Acts };
            own.max(effect_reach(then)).max(effect_reach(otherwise))
        }
        // Remembering a choice or a number, and effects on the entering permanent itself
        // performed as it enters.
        Effect::Choose { .. }
        | Effect::ChooseCounterKind { .. }
        | Effect::ChooseSource { .. }
        | Effect::Store { .. }
        | Effect::StoreValue { .. }
        | Effect::Note { .. }
        | Effect::NoteLinked { .. }
        | Effect::NoteManaSpent { .. }
        | Effect::SetX { .. }
        | Effect::OnEntry(_) => Reach::Reads,
        Effect::EnterCopyExtra { effect, .. } => effect_reach(effect).max(Reach::Reads),
        _ => Reach::Acts,
    }
}

fn value_reach(v: &Value) -> Reach {
    if matches!(v, Value::Const(_)) {
        Reach::Plain
    } else {
        Reach::Reads
    }
}

/// How far a replacement effect that modifies how a permanent enters reaches.
fn action_reach(a: &ReplacementAction) -> Reach {
    match a {
        ReplacementAction::EnterTapped | ReplacementAction::EnterUntapped => Reach::Plain,
        ReplacementAction::EnterWithCounters(_, n) => value_reach(n),
        ReplacementAction::AsEnters(e) => effect_reach(e),
        // A copy, control or face-down choice reads the game; an event replaced by
        // another (exiled instead) doesn't change it as it's chosen.
        _ => Reach::Reads,
    }
}

/// The players who choose the order of their entering permanents' replacement effects
/// for one `Game::move_objects`, and the moves whose order each one chooses.
#[derive(Debug, Default)]
pub(crate) struct EntryOrder {
    /// Per player: the moves whose effects they order (in the order of the moves), and
    /// those not yet applied.
    choosers: Vec<(PlayerId, Vec<usize>, Vec<usize>)>,
}

impl EntryOrder {
    /// The moves (indices into `moves`, processed in `order`) whose replacement effects
    /// their controller orders: for each player with two or more permanents entering
    /// whose effects reach beyond themselves, one of which changes the game as it's
    /// applied (see the module docs).
    pub(crate) fn new(g: &mut Game, moves: &[MoveEv], order: &[usize]) -> EntryOrder {
        let entering: Vec<usize> = order
            .iter()
            .copied()
            .filter(|i| moves[*i].to == Zone::Battlefield)
            .collect();
        if entering.len() < 2 {
            return EntryOrder::default();
        }
        let mut by_player: Vec<(PlayerId, Vec<usize>)> = Vec::new();
        for i in entering {
            let p = g.affected_player(&ReplEvent::Move(moves[i].clone()));
            match by_player.iter_mut().find(|(q, _)| *q == p) {
                Some((_, v)) => v.push(i),
                None => by_player.push((p, vec![i])),
            }
        }
        let mut choosers = Vec::new();
        for (p, mine) in by_player {
            if mine.len() < 2 {
                continue;
            }
            let mut reaching = Vec::new();
            let mut acts = false;
            for i in mine {
                let reach = g
                    .entry_replacements(&moves[i])
                    .iter()
                    .map(|(_, a)| action_reach(a))
                    .max()
                    .unwrap_or(Reach::Plain);
                if reach > Reach::Plain {
                    reaching.push(i);
                    acts |= reach == Reach::Acts;
                }
            }
            if reaching.len() >= 2 && acts {
                choosers.push((p, reaching.clone(), reaching));
            }
        }
        EntryOrder { choosers }
    }

    /// The move whose replacement effects are applied in the place of move `slot` (the next
    /// one in the order of the moves): `slot` itself, unless it's one of the permanents
    /// whose order their controller chooses; then the one they choose among those of theirs
    /// not yet applied (CR 616.1, 616.1f), by default the first of them in the order of the
    /// moves.
    pub(crate) fn pick(&mut self, g: &mut Game, moves: &[MoveEv], slot: usize) -> usize {
        let Some(k) = self
            .choosers
            .iter()
            .position(|(_, all, _)| all.contains(&slot))
        else {
            return slot;
        };
        let (p, _, left) = &self.choosers[k];
        let p = *p;
        let left = left.clone();
        if left.is_empty() {
            return slot;
        }
        let pick = if left.len() < 2 {
            0
        } else {
            // "Thief of Blood #61: As ~ enters, remove all counters from all permanents. ..."
            let options: Vec<String> = left
                .iter()
                .map(|i| {
                    let obj = moves[*i].obj;
                    let name = format!("{}: ", g.obj(obj).chars.name);
                    let texts: Vec<String> = g
                        .entry_replacements(&moves[*i])
                        .into_iter()
                        .map(|(t, _)| {
                            t.strip_prefix(name.as_str())
                                .map(str::to_string)
                                .unwrap_or(t)
                        })
                        .collect();
                    format!("{}: {}", g.describe(obj), texts.join(" / "))
                })
                .collect();
            match g.ask(p, Decision::ChooseReplacement { options }) {
                Answer::Index(i) if i < left.len() => i,
                _ => 0,
            }
        };
        let i = left[pick];
        self.choosers[k].2.retain(|x| *x != i);
        i
    }
}
