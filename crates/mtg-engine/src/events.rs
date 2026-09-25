//! Game events. Every game action emits events; triggered abilities (CR 603) and
//! turn history ("if a creature died this turn") are driven by them.

use crate::ability::{Ability, ZoneKind};
use crate::object::Zone;
use crate::turn::Step;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Why an object changed zones.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoveCause {
    Cast,
    PlayLand,
    Resolve,
    /// Put onto the battlefield / into a zone by an effect.
    Effect,
    Destroy,
    Sacrifice,
    Discard,
    Mill,
    Draw,
    Counter,
    /// A state-based action (e.g. lethal damage is "destroy", toughness 0 is this).
    StateBased,
    /// Ceased to exist (tokens off the battlefield, copies off the stack).
    CeaseToExist,
    Exile,
    Return,
    Search,
    Cleanup,
    Cost,
    /// Returned to the command zone (commander rule).
    Commander,
    Other,
}

/// Triggered abilities of permanents captured just before an event, so that
/// leaves-the-battlefield abilities "look back in time" (CR 603.10).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LookbackSnapshot {
    /// (source object, its controller, triggered ability)
    pub sources: Vec<(ObjectId, PlayerId, Ability)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    ZoneChange {
        old: ObjectId,
        new: ObjectId,
        from: Zone,
        to: Zone,
        cause: MoveCause,
        /// Player responsible (who sacrificed, discarded, cast, ...).
        by: Option<PlayerId>,
        #[serde(skip)]
        lookback: Option<Arc<LookbackSnapshot>>,
    },
    SpellCast {
        spell: ObjectId,
        player: PlayerId,
        from: Option<ZoneKind>,
    },
    AbilityActivated {
        /// The ability on the stack (None for mana abilities).
        ability: Option<ObjectId>,
        source: ObjectId,
        player: PlayerId,
        is_mana: bool,
    },
    AbilityTriggeredOnStack {
        ability: ObjectId,
        source: ObjectId,
    },
    SpellResolved {
        spell: ObjectId,
    },
    Countered {
        what: ObjectId,
    },
    Damage {
        source: ObjectId,
        target: Entity,
        amount: u32,
        combat: bool,
    },
    LifeGained {
        player: PlayerId,
        amount: u32,
    },
    LifeLost {
        player: PlayerId,
        amount: u32,
    },
    Drew {
        player: PlayerId,
        card: ObjectId,
        /// 1-based count of cards drawn this turn by this player.
        nth: u32,
    },
    Discarded {
        player: PlayerId,
        card: ObjectId,
    },
    Milled {
        player: PlayerId,
        cards: Vec<ObjectId>,
    },
    CountersAdded {
        target: Entity,
        kind: CounterKind,
        n: u32,
    },
    CountersRemoved {
        target: Entity,
        kind: CounterKind,
        n: u32,
    },
    Tapped {
        obj: ObjectId,
        for_mana: bool,
    },
    Untapped {
        obj: ObjectId,
    },
    AttackersDeclared {
        player: PlayerId,
        attackers: Vec<(ObjectId, Entity)>,
    },
    BlockersDeclared {
        blocks: Vec<(ObjectId, ObjectId)>,
    },
    BecameBlocked {
        attacker: ObjectId,
        blockers: Vec<ObjectId>,
    },
    AttackerUnblocked {
        attacker: ObjectId,
    },
    /// A creature started blocking an attacker other than by being declared as a blocker:
    /// an effect made it block (`entered == false`), or it was put onto the battlefield
    /// blocking (`entered == true`) (CR 509.3a–e, 509.4).
    BlockAdded {
        blocker: ObjectId,
        attacker: ObjectId,
        entered: bool,
        /// The blocker was already a blocking creature.
        was_blocking: bool,
        /// The attacker was already a blocked creature.
        was_blocked: bool,
    },
    BecameTarget {
        target: Entity,
        by: ObjectId,
        controller: PlayerId,
    },
    StepBegan {
        step: Step,
        active: PlayerId,
    },
    TurnBegan {
        active: PlayerId,
        number: u32,
    },
    TokenCreated {
        obj: ObjectId,
        controller: PlayerId,
    },
    LandPlayed {
        player: PlayerId,
        land: ObjectId,
    },
    Cycled {
        player: PlayerId,
        card: ObjectId,
    },
    TurnedFaceUp {
        obj: ObjectId,
    },
    TurnedFaceDown {
        obj: ObjectId,
    },
    Transformed {
        obj: ObjectId,
    },
    ControlChanged {
        obj: ObjectId,
        from: PlayerId,
        to: PlayerId,
    },
    PlayerLost {
        player: PlayerId,
    },
    PlayerWon {
        player: PlayerId,
    },
    Searched {
        player: PlayerId,
    },
    Shuffled {
        player: PlayerId,
    },
    Sacrificed {
        obj: ObjectId,
        player: PlayerId,
    },
    Destroyed {
        obj: ObjectId,
    },
    Attached {
        obj: ObjectId,
        to: Entity,
    },
    Unattached {
        obj: ObjectId,
        from: Entity,
    },
    PhasedOut {
        obj: ObjectId,
    },
    PhasedIn {
        obj: ObjectId,
    },
    DieRolled {
        player: PlayerId,
        sides: u32,
        result: u32,
    },
    CoinFlipped {
        player: PlayerId,
        won: bool,
    },
    DayNightChanged {
        is_day: bool,
    },
    BecameMonarch {
        player: PlayerId,
    },
    TookInitiative {
        player: PlayerId,
    },
    CrimeCommitted {
        player: PlayerId,
    },
    ManaAdded {
        player: PlayerId,
        source: Option<ObjectId>,
    },
    Exploited {
        obj: ObjectId,
    },
    /// Any other event identified by name (used by keyword/card implementations).
    Custom {
        name: smol_str::SmolStr,
        player: Option<PlayerId>,
        obj: Option<ObjectId>,
        amount: i32,
    },
}
