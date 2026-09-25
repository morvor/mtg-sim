//! Decisions players make, and the [`Agent`] trait that makes them.
//!
//! The engine asks the relevant player's agent whenever the rules call for a choice.
//! Agents receive the full game state (hidden information included — agents that should
//! not cheat must restrict themselves).

use crate::game::Game;
use crate::object::CastMethod;
use crate::types::*;
use serde::{Deserialize, Serialize};

/// Things a player can do when they have priority (CR 117.1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Pass,
    PlayLand {
        card: ObjectId,
    },
    Cast {
        card: ObjectId,
        method: CastMethod,
    },
    /// Activate an activated ability (by ability uid) of a source.
    Activate {
        source: ObjectId,
        ability: u64,
    },
    Special(SpecialAction),
    Concede,
}

/// Special actions (CR 116).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpecialAction {
    /// Turn a face-down permanent face up (CR 116.2b, 702.37e).
    TurnFaceUp { obj: ObjectId },
    /// Exile a card with suspend from hand (CR 702.62a).
    Suspend { card: ObjectId },
    /// Foretell (CR 702.143a).
    Foretell { card: ObjectId },
    /// Plot (CR 702.170a).
    Plot { card: ObjectId },
    /// Put companion into hand (CR 702.139c).
    CompanionToHand { card: ObjectId },
    /// A special action granted by a static ability of `source` (CR 116.2d, 116.2e).
    Static { source: ObjectId, ability: u64 },
    /// A special action an effect allows (CR 116.2c), by offer id.
    Offer { id: u32 },
    /// Roll the planar die (CR 116.2i, 901.9).
    RollPlanarDie,
    /// End the turn with a split second / etc. Other special actions by name.
    Other { name: String, obj: Option<ObjectId> },
}

/// A choice the engine needs a player to make.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Decision {
    /// The player has priority. Answer with `Answer::Action`.
    Priority { actions: Vec<Action> },
    /// Keep or mulligan (CR 103.5). Answer `Bool(true)` to mulligan.
    Mulligan { mulligans_taken: u32 },
    /// Choose `n` cards to put on the bottom (London mulligan). Answer `Entities`.
    PutOnBottom { cards: Vec<ObjectId>, n: u32 },
    /// Choose modes (CR 700.2). Answer `Indices`.
    ChooseModes {
        source: ObjectId,
        modes: Vec<String>,
        min: u32,
        max: u32,
        allow_repeat: bool,
    },
    /// Announce X (CR 107.3). Answer `Number`.
    ChooseX { source: ObjectId, max: i64 },
    /// Choose a casting method / alternative cost. Answer `Index`.
    ChooseCastingMethod {
        card: ObjectId,
        options: Vec<String>,
    },
    /// Pay an optional additional cost (kicker, buyback, ...). Answer `Bool` (or `Number`
    /// for multikicker: how many times).
    OptionalCost {
        source: ObjectId,
        name: String,
        repeatable: bool,
    },
    /// Choose targets for one target slot. Answer `Entities`.
    ChooseTargets {
        source: ObjectId,
        text: String,
        candidates: Vec<Entity>,
        min: u32,
        max: u32,
    },
    /// Divide an amount among recipients (CR 601.2d). Answer `Numbers` (one per recipient,
    /// each at least `min_each`).
    Divide {
        source: ObjectId,
        total: u32,
        recipients: Vec<Entity>,
        min_each: u32,
    },
    /// Yes/no ("you may ..."). Answer `Bool`.
    YesNo {
        source: Option<ObjectId>,
        prompt: String,
    },
    /// Choose among entities (non-targeted choices, sacrifices, discards, ...). Answer `Entities`.
    ChooseEntities {
        source: Option<ObjectId>,
        prompt: String,
        candidates: Vec<Entity>,
        min: u32,
        max: u32,
    },
    /// Order items (triggers, cards going to the bottom, ...). Answer `Indices` (a permutation).
    Order { prompt: String, items: Vec<String> },
    /// Choose one option by index. Answer `Index`.
    ChooseOption {
        source: Option<ObjectId>,
        prompt: String,
        options: Vec<String>,
    },
    /// Choose a number in range. Answer `Number`.
    ChooseNumber {
        source: Option<ObjectId>,
        prompt: String,
        min: i64,
        max: i64,
    },
    /// Name a card (CR 201.3). Answer `Text`.
    NameCard {
        source: Option<ObjectId>,
        prompt: String,
    },
    /// Declare attackers (CR 508.1). Each option lists what that creature may attack.
    /// Answer `Attackers`.
    DeclareAttackers {
        options: Vec<(ObjectId, Vec<Entity>)>,
    },
    /// Declare blockers (CR 509.1). Each option lists attackers that creature may block.
    /// Answer `Blockers`.
    DeclareBlockers {
        options: Vec<(ObjectId, Vec<ObjectId>)>,
    },
    /// Assign combat damage for one creature (CR 510.1). `recipients` are the creatures it
    /// can assign to plus (for trample) the player/permanent it's attacking, plus (for
    /// trample over planeswalkers attacking a planeswalker, CR 702.19c) that
    /// planeswalker's controller. `lethal` gives the lethal damage for each creature
    /// recipient, followed in that last case by the damage the planeswalker must be
    /// assigned before its controller can be. Answer `Numbers`.
    AssignCombatDamage {
        creature: ObjectId,
        amount: u32,
        recipients: Vec<Entity>,
        lethal: Vec<u32>,
        trample: bool,
    },
    /// Scry (CR 701.22): split into top (in order) and bottom. Answer `Split(top, bottom)`.
    Scry { cards: Vec<ObjectId> },
    /// Surveil (CR 701.25): split into top (in order) and graveyard. Answer `Split(top, graveyard)`.
    Surveil { cards: Vec<ObjectId> },
    /// Choose which replacement/prevention effect to apply first (CR 616.1). Answer `Index`.
    ChooseReplacement { options: Vec<String> },
}

/// A player's answer to a [`Decision`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Answer {
    /// Let the engine choose a reasonable default.
    Default,
    Action(Action),
    Bool(bool),
    Index(usize),
    Indices(Vec<usize>),
    Number(i64),
    Numbers(Vec<i64>),
    Entities(Vec<Entity>),
    Text(String),
    Attackers(Vec<(ObjectId, Entity)>),
    Blockers(Vec<(ObjectId, ObjectId)>),
    Split(Vec<ObjectId>, Vec<ObjectId>),
}

/// Something that makes decisions for a player.
pub trait Agent: Send {
    fn decide(&mut self, game: &Game, player: PlayerId, decision: &Decision) -> Answer;

    fn name(&self) -> &str {
        "agent"
    }
}

/// An agent that always lets the engine choose defaults (passes priority, keeps hands,
/// declares no attackers or blockers, chooses first legal options).
#[derive(Clone, Default)]
pub struct PassiveAgent;

impl Agent for PassiveAgent {
    fn decide(&mut self, _game: &Game, _player: PlayerId, decision: &Decision) -> Answer {
        match decision {
            Decision::Priority { .. } => Answer::Action(Action::Pass),
            Decision::Mulligan { .. } => Answer::Bool(false),
            Decision::DeclareAttackers { .. } => Answer::Attackers(vec![]),
            Decision::DeclareBlockers { .. } => Answer::Blockers(vec![]),
            _ => Answer::Default,
        }
    }
    fn name(&self) -> &str {
        "passive"
    }
}
