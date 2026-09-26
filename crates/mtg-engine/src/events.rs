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
    /// A dungeon card brought from outside the game into the command zone by the venture
    /// into the dungeon keyword action (CR 309.2a, 701.49): the only way a dungeon card
    /// can be brought into the game (CR 309.2d).
    Venture,
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
    /// An activated or triggered ability finished resolving (CR 608.2p).
    AbilityResolved {
        ability: ObjectId,
        source: ObjectId,
        controller: PlayerId,
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
    /// A permanent was dealt excess damage (CR 120.10).
    ExcessDamage {
        obj: ObjectId,
        amount: u32,
        combat: bool,
    },
    /// A prevention effect prevented some or all of the damage that would have been dealt
    /// (CR 615.13). `by` is the prevention effect's source; `key` identifies the effect.
    DamagePrevented {
        source: ObjectId,
        target: Entity,
        amount: u32,
        by: Option<ObjectId>,
        key: u64,
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
        /// The player who removed them — the controller of the effect or the player paying
        /// the cost — if a player did ("when you remove the last ... counter").
        by: Option<PlayerId>,
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
    /// A die was rolled (CR 706): its result after modifiers and its natural result
    /// (CR 706.2). The planar die has no numerical result (CR 706.7).
    DieRolled {
        player: PlayerId,
        sides: u32,
        result: u32,
        natural: u32,
        planar: bool,
    },
    /// A coin was flipped (CR 705). A flip only cares about heads or tails has neither a
    /// winner nor a loser (CR 705.2).
    CoinFlipped {
        player: PlayerId,
        won: bool,
        lost: bool,
        heads: bool,
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
    /// `by` exploited `obj` (CR 702.110b): `player`, the controller of `by`'s exploit
    /// ability, sacrificed `obj` (as it was on the battlefield) as that ability resolved.
    Exploited {
        obj: ObjectId,
        by: ObjectId,
        player: PlayerId,
    },
    /// A copy of a spell was put onto the stack (CR 707.10); `player` controls the copy.
    SpellCopied {
        spell: ObjectId,
        player: PlayerId,
    },
    /// `player` tapped `obj` for mana (CR 106.12): a mana ability of it with {T} in its
    /// cost resolved and produced `mana` (CR 106.12a).
    TappedForMana {
        obj: ObjectId,
        player: PlayerId,
        mana: Vec<crate::mana::ManaType>,
    },
    /// Marks the end of a group of simultaneous events (one action of a resolving spell or
    /// ability, CR 608.2c). Events between two markers (or flushes) form one batch for
    /// "whenever one or more …" triggers (CR 603.2c). Not recorded in turn history.
    BatchBoundary,
    /// Any other event identified by name (used by keyword/card implementations).
    Custom {
        name: smol_str::SmolStr,
        player: Option<PlayerId>,
        obj: Option<ObjectId>,
        amount: i32,
    },
}
