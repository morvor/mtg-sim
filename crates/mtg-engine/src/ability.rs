//! The ability language: a data representation of everything card text can say.
//!
//! Oracle text is compiled into these structures by [`crate::oracle`], and the engine
//! interprets them. The main pieces:
//!
//! * [`AbilityDef`] — one ability of an object: spell, activated, triggered, static,
//!   or keyword (CR 113).
//! * [`Effect`] — what a spell or ability does when it resolves (CR 609, 610).
//! * [`Sel`] — selects players/objects an effect acts on (targets, "each creature", …).
//! * [`Filter`] — predicates over objects ("nontoken creature you control").
//! * [`Value`] — numbers, possibly computed from the game ("the number of Elves you control").
//! * [`Condition`] — booleans over the game ("if you control an artifact").
//! * [`Cost`] — costs to cast/activate (CR 118).
//! * [`TriggerCond`] — trigger events (CR 603).
//! * [`StaticEffect`] — continuous effects, restrictions, replacement effects (CR 604, 611, 614).

use crate::keywords::{Keyword, KeywordKind};
use crate::mana::{ManaCost, ManaRestriction, ManaType};
use crate::types::{CardType, Color, ColorSet, CounterKind, PlayerId, Subtype, Supertype};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

static NEXT_ABILITY_UID: AtomicU64 = AtomicU64::new(1);

/// Allocates a process-unique id for an ability definition. Ability identity matters
/// for "activate only once each turn", linked abilities (CR 607), and for removing
/// specific abilities.
pub fn next_ability_uid() -> u64 {
    NEXT_ABILITY_UID.fetch_add(1, Ordering::Relaxed)
}

/// One ability (CR 113).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AbilityDef {
    pub uid: u64,
    pub kind: AbilityKind,
    /// Oracle text this ability was compiled from (for display/debugging).
    pub text: String,
    /// For abilities linked by CR 607: abilities on the same object with the same
    /// non-zero `link` share data (e.g. "exiled with this").
    pub link: u16,
}

impl AbilityDef {
    pub fn new(kind: AbilityKind, text: impl Into<String>) -> Arc<AbilityDef> {
        Arc::new(AbilityDef {
            uid: next_ability_uid(),
            kind,
            text: text.into(),
            link: 0,
        })
    }
    pub fn with_link(kind: AbilityKind, text: impl Into<String>, link: u16) -> Arc<AbilityDef> {
        Arc::new(AbilityDef {
            uid: next_ability_uid(),
            kind,
            text: text.into(),
            link,
        })
    }
    pub fn keyword(&self) -> Option<&Keyword> {
        match &self.kind {
            AbilityKind::Keyword(k) => Some(k),
            _ => None,
        }
    }
    pub fn is_mana_ability(&self) -> bool {
        match &self.kind {
            AbilityKind::Activated(a) => a.is_mana_ability,
            AbilityKind::Triggered(t) => t.is_mana_ability,
            _ => false,
        }
    }
}

pub type Ability = Arc<AbilityDef>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AbilityKind {
    /// The effect text of an instant or sorcery (CR 113.3a).
    Spell(SpellAbility),
    Activated(ActivatedAbility),
    Triggered(TriggeredAbility),
    Static(StaticAbility),
    Keyword(Keyword),
    /// Text the compiler could not understand. Its presence marks a card as only
    /// partially supported; it has no game effect.
    Unsupported(String),
}

/// What gets put on the stack: targets + effect, or a set of modes (CR 700.2).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Body {
    pub targets: Vec<TargetSpec>,
    pub effect: Effect,
    pub modal: Option<Modal>,
}

impl Body {
    pub fn simple(targets: Vec<TargetSpec>, effect: Effect) -> Body {
        Body {
            targets,
            effect,
            modal: None,
        }
    }
    pub fn effect(effect: Effect) -> Body {
        Body {
            targets: vec![],
            effect,
            modal: None,
        }
    }
}

/// Modal spells and abilities (CR 700.2).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Modal {
    pub min: Value,
    pub max: Value,
    /// "You may choose the same mode more than once" (CR 700.2d).
    pub allow_repeat: bool,
    pub modes: Vec<Mode>,
    /// Escalate / spree / "choose one that hasn't been chosen" bookkeeping is handled by
    /// the engine based on these flags.
    pub per_mode_cost: bool,
    /// Modes chosen by an opponent (CR 700.2e) or at random.
    pub chooser: ModeChooser,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModeChooser {
    #[default]
    Controller,
    Opponent,
    Random,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mode {
    pub text: String,
    pub targets: Vec<TargetSpec>,
    pub effect: Effect,
    /// Spree / tiered additional cost for choosing this mode.
    pub cost: Option<Cost>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SpellAbility {
    pub body: Body,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActivatedAbility {
    pub cost: Cost,
    pub body: Body,
    pub timing: ActivationTiming,
    /// Mana abilities (CR 605) don't use the stack.
    pub is_mana_ability: bool,
    /// Loyalty abilities (CR 606).
    pub is_loyalty: bool,
    /// "Activate only once each turn" and similar.
    pub max_per_turn: Option<u32>,
    /// Extra condition ("Activate only if you control an artifact").
    pub condition: Option<Condition>,
    /// Zone the ability functions from (battlefield by default; hand for cycling etc.).
    pub zone: FunctionZone,
    /// "Any player may activate this ability."
    pub any_player: bool,
}

impl ActivatedAbility {
    pub fn new(cost: Cost, body: Body) -> Self {
        ActivatedAbility {
            cost,
            body,
            timing: ActivationTiming::Instant,
            is_mana_ability: false,
            is_loyalty: false,
            max_per_turn: None,
            condition: None,
            zone: FunctionZone::Battlefield,
            any_player: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivationTiming {
    #[default]
    Instant,
    /// "Activate only as a sorcery" (CR 602.5d).
    Sorcery,
    /// "Activate only during your upkeep" etc.
    YourUpkeep,
    /// "Activate only during combat".
    Combat,
    /// "Activate only before blockers are declared" (CR 506.8).
    BeforeBlockers,
    YourTurn,
    OpponentsTurn,
}

/// Where an ability functions (CR 113.6).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FunctionZone {
    #[default]
    Battlefield,
    Hand,
    Graveyard,
    Exile,
    Library,
    Stack,
    Command,
    /// Functions in every zone (e.g. characteristic-defining abilities, CR 604.3).
    Anywhere,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TriggeredAbility {
    pub trigger: TriggerCond,
    /// Intervening "if" clause (CR 603.4): checked on trigger and on resolution.
    pub intervening_if: Option<Condition>,
    pub body: Body,
    /// "This ability triggers only once each turn."
    pub once_per_turn: bool,
    /// Triggered mana abilities (CR 605.1b) resolve immediately.
    pub is_mana_ability: bool,
    pub zone: FunctionZone,
}

impl TriggeredAbility {
    pub fn new(trigger: TriggerCond, body: Body) -> Self {
        TriggeredAbility {
            trigger,
            intervening_if: None,
            body,
            once_per_turn: false,
            is_mana_ability: false,
            zone: FunctionZone::Battlefield,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticAbility {
    /// "As long as ..." condition.
    pub condition: Option<Condition>,
    pub effect: StaticEffect,
    pub zone: FunctionZone,
    /// Characteristic-defining abilities apply first within their layer and in all zones
    /// (CR 604.3, 613.3).
    pub is_cda: bool,
}

impl StaticAbility {
    pub fn new(effect: StaticEffect) -> Self {
        StaticAbility {
            condition: None,
            effect,
            zone: FunctionZone::Battlefield,
            is_cda: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Costs (CR 118)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Cost {
    pub mana: Option<ManaCost>,
    pub parts: Vec<CostPart>,
}

impl Cost {
    pub fn free() -> Cost {
        Cost::default()
    }
    pub fn mana(m: ManaCost) -> Cost {
        Cost {
            mana: Some(m),
            parts: vec![],
        }
    }
    pub fn tap() -> Cost {
        Cost {
            mana: None,
            parts: vec![CostPart::Tap],
        }
    }
    pub fn with(mut self, p: CostPart) -> Cost {
        self.parts.push(p);
        self
    }
    pub fn has_tap(&self) -> bool {
        self.parts.iter().any(|p| matches!(p, CostPart::Tap))
    }
    pub fn is_free(&self) -> bool {
        self.mana.as_ref().is_none_or(|m| m.is_zero()) && self.parts.is_empty()
    }
    pub fn loyalty(&self) -> Option<i32> {
        self.parts.iter().find_map(|p| {
            if let CostPart::Loyalty(n) = p {
                Some(*n)
            } else {
                None
            }
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CostPart {
    /// {T}
    Tap,
    /// {Q}
    Untap,
    PayLife(Value),
    /// Loyalty cost: +N or −N loyalty counters (CR 606).
    Loyalty(i32),
    SacrificeSelf,
    Sacrifice {
        filter: Filter,
        count: Value,
    },
    DiscardSelf,
    Discard {
        filter: Filter,
        count: Value,
        random: bool,
    },
    DiscardHand,
    ExileSelf,
    /// Exile cards from a zone (e.g. "Exile a card from your graveyard").
    Exile {
        filter: Filter,
        zone: ZoneKind,
        count: Value,
    },
    ReturnToHand {
        filter: Filter,
        count: Value,
    },
    ReturnSelfToHand,
    RemoveCounters {
        kind: CounterKind,
        count: Value,
    },
    /// "Remove X +1/+1 counters from among creatures you control" etc.
    RemoveCountersFromAmong {
        kind: Option<CounterKind>,
        filter: Filter,
        count: Value,
    },
    AddCounters {
        kind: CounterKind,
        count: Value,
    },
    /// "Tap an untapped creature you control".
    TapUntapped {
        filter: Filter,
        count: Value,
    },
    UntapTapped {
        filter: Filter,
        count: Value,
    },
    PayEnergy(Value),
    PayPlayerCounters {
        kind: CounterKind,
        count: Value,
    },
    Mill(Value),
    RevealFromHand {
        filter: Filter,
        count: Value,
    },
    ExertSelf,
    /// "Collect evidence N" (CR 701.59).
    CollectEvidence(u32),
    /// Forage (CR 701.61): exile three cards from your graveyard or sacrifice a Food.
    Forage,
    /// Put a card from hand on top/bottom of library etc.
    PutFromHandOnLibrary {
        filter: Filter,
        count: Value,
        top: bool,
    },
    /// Arbitrary effect performed as a cost (e.g. "Blight 1").
    Effect(Box<Effect>),
}

// ---------------------------------------------------------------------------
// Zones
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ZoneKind {
    Library,
    Hand,
    Battlefield,
    Graveyard,
    Stack,
    Exile,
    Command,
    Ante,
    /// Outside the game (sideboard, CR 400.11).
    Outside,
}

/// Where an effect puts an object.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Destination {
    pub zone: ZoneKind,
    /// For libraries: position from top (0 = top). `None` = bottom when `bottom` is true.
    pub position: LibraryPosition,
    /// Battlefield: enters tapped.
    pub tapped: bool,
    /// Battlefield: under whose control (default: owner, or the effect's controller for
    /// "put onto the battlefield under your control").
    pub controller: Option<PlayerRef>,
    /// Exile face down.
    pub face_down: bool,
    /// Battlefield: enters attacking.
    pub attacking: bool,
    /// Battlefield: enters transformed (back face up).
    pub transformed: bool,
    /// Counters it enters with.
    pub with_counters: Vec<(CounterKind, Value)>,
}

impl Destination {
    pub fn zone(zone: ZoneKind) -> Destination {
        Destination {
            zone,
            position: LibraryPosition::Top,
            tapped: false,
            controller: None,
            face_down: false,
            attacking: false,
            transformed: false,
            with_counters: vec![],
        }
    }
    pub fn battlefield() -> Destination {
        Destination::zone(ZoneKind::Battlefield)
    }
    pub fn under_your_control(mut self) -> Destination {
        self.controller = Some(PlayerRef::You);
        self
    }
    pub fn tapped(mut self) -> Destination {
        self.tapped = true;
        self
    }
    pub fn library_top() -> Destination {
        Destination::zone(ZoneKind::Library)
    }
    pub fn library_bottom() -> Destination {
        let mut d = Destination::zone(ZoneKind::Library);
        d.position = LibraryPosition::Bottom;
        d
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LibraryPosition {
    Top,
    Bottom,
    /// Nth from the top (0-based).
    FromTop(u32),
    /// Shuffle into.
    Shuffled,
}

// ---------------------------------------------------------------------------
// Targets (CR 115)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TargetSpec {
    pub what: TargetKind,
    pub min: u32,
    pub max: Value,
    /// Each target in this slot must be different from targets in these other slots
    /// ("another target creature").
    pub distinct_from: Vec<u8>,
    /// "Divide N damage among any number of targets" (CR 601.2d).
    pub divide: Option<Value>,
    /// Targets chosen by an opponent (CR 601.7).
    pub chosen_by_opponent: bool,
    /// Human-readable description ("target creature you control").
    pub text: String,
}

impl TargetSpec {
    pub fn one(what: TargetKind, text: impl Into<String>) -> TargetSpec {
        TargetSpec {
            what,
            min: 1,
            max: Value::Const(1),
            distinct_from: vec![],
            divide: None,
            chosen_by_opponent: false,
            text: text.into(),
        }
    }
    pub fn up_to(n: i32, what: TargetKind, text: impl Into<String>) -> TargetSpec {
        TargetSpec {
            min: 0,
            max: Value::Const(n),
            ..TargetSpec::one(what, text)
        }
    }
    pub fn object(filter: Filter, text: impl Into<String>) -> TargetSpec {
        TargetSpec::one(TargetKind::Object(filter), text)
    }
    pub fn player(filter: PlayerFilter, text: impl Into<String>) -> TargetSpec {
        TargetSpec::one(TargetKind::Player(filter), text)
    }
    pub fn any_target() -> TargetSpec {
        TargetSpec::one(TargetKind::AnyTarget, "any target")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TargetKind {
    /// A permanent, card, or spell matching the filter (the filter should include its
    /// zone; the default zone for objects is the battlefield).
    Object(Filter),
    Player(PlayerFilter),
    /// "any target": creature, player, planeswalker, or battle (CR 115.4).
    AnyTarget,
    /// Object or player: `(object filter, player filter)`.
    ObjectOrPlayer(Filter, PlayerFilter),
    /// A spell on the stack.
    Spell(Filter),
    /// An activated or triggered ability on the stack.
    Ability(Filter),
    /// A spell or ability on the stack.
    SpellOrAbility(Filter),
}

// ---------------------------------------------------------------------------
// Selections, players, filters
// ---------------------------------------------------------------------------

/// A variable slot used to remember objects/players/numbers during resolution
/// ("exile target creature. Its controller creates a token" — "its" refers to the
/// exiled creature's last known information).
pub type Var = u16;

/// Well-known variables.
pub mod vars {
    use super::Var;
    /// Objects affected by the most recent effect ("it", "those cards", "the exiled card").
    pub const IT: Var = 0;
    /// Objects created by the most recent effect (tokens).
    pub const CREATED: Var = 1;
    /// Cards drawn/revealed/looked at.
    pub const REVEALED: Var = 2;
    /// First user-defined variable.
    pub const USER: Var = 10;
}

/// Selects players and/or objects.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Sel {
    /// Nothing.
    None,
    /// The source object of the ability (the permanent itself, "this creature").
    This,
    /// All legal targets chosen in target slot N.
    Target(u8),
    /// All targets across all slots.
    AllTargets,
    /// Objects/players stored in a variable.
    Var(Var),
    /// The object the triggering event was about ("that creature", "it"). For zone
    /// changes, the object in its new zone.
    TriggerObject,
    /// Last known information of the triggering object (e.g. "its power" in a dies trigger).
    TriggerLki,
    /// The other object involved in the trigger (e.g. the blocker, or the damage source).
    TriggerOtherObject,
    /// The player from the triggering event ("that player").
    TriggerPlayer,
    /// The permanent or player this object is attached to ("enchanted creature").
    AttachedTo,
    /// Objects attached to the source ("equipment attached to it").
    AttachedToThis,
    /// All objects matching the filter.
    All(Filter),
    /// Players.
    Players(PlayerRef),
    /// Objects chosen during resolution (not targeted): a player chooses.
    Choose {
        chooser: PlayerRef,
        filter: Filter,
        count: Value,
        up_to: bool,
        store: Option<Var>,
    },
    /// Cards linked to this object by CR 607 ("the exiled cards", "cards exiled with this").
    Linked,
    /// The spell this ability resolves for (for "copy that spell").
    TriggerSpell,
    /// Union of selections.
    Union(Vec<Sel>),
}

/// Refers to one or more players.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PlayerRef {
    /// A specific player (locked in at resolution).
    Player(PlayerId),
    /// The controller of the resolving spell/ability, or of the source for statics.
    You,
    /// Each opponent.
    EachOpponent,
    /// Each player.
    EachPlayer,
    /// Each other player (multiplayer-aware "each other player").
    EachOtherPlayer,
    /// Players chosen as targets in slot N.
    Target(u8),
    /// Controller of the object(s) selected.
    ControllerOf(Box<Sel>),
    /// Owner of the object(s) selected.
    OwnerOf(Box<Sel>),
    /// The player from the triggering event.
    TriggerPlayer,
    /// The active player.
    ActivePlayer,
    /// The defending player (CR 508.5).
    DefendingPlayer,
    /// A player chosen during resolution by the controller.
    ChosenPlayer(Var),
    /// Players stored in a variable.
    Var(Var),
    /// Each player matching a filter.
    Each(PlayerFilter),
    /// The owner of the source object.
    Owner,
    /// "that player" in the context of a "for each player" loop.
    Iterated,
    /// The opponent chosen by "choose an opponent".
    ChosenOpponent,
    /// The monarch / initiative holder etc.
    Monarch,
}

/// Player predicates, used in targets and filters.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PlayerFilter {
    Any,
    /// Exactly this player.
    Is(PlayerId),
    You,
    Opponent,
    NotYou,
    /// The controller of the source (same as You for most purposes).
    Controller,
    /// A player who was dealt damage this turn, etc.
    DealtDamageThisTurn,
    /// Life comparisons.
    Life(Cmp, Box<Value>),
    /// Hand size comparisons.
    HandSize(Cmp, Box<Value>),
    /// The monarch.
    Monarch,
    /// The defending player.
    Defending,
    /// The active player.
    Active,
    And(Vec<PlayerFilter>),
    Or(Vec<PlayerFilter>),
    Not(Box<PlayerFilter>),
}

/// Comparison operators for filters and conditions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cmp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl Cmp {
    pub fn eval(self, a: i64, b: i64) -> bool {
        match self {
            Cmp::Eq => a == b,
            Cmp::Ne => a != b,
            Cmp::Lt => a < b,
            Cmp::Le => a <= b,
            Cmp::Gt => a > b,
            Cmp::Ge => a >= b,
        }
    }
}

/// Relationship of a player to the source's controller.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerRel {
    You,
    Opponent,
    Any,
    NotYou,
    /// The player chosen as target in slot N (for "creature target player controls").
    Target(u8),
    /// The triggering player.
    TriggerPlayer,
    /// The defending player.
    Defending,
    /// The active player.
    Active,
    /// A teammate (multiplayer team variants).
    Teammate,
    /// The iterated player in "for each player".
    Iterated,
}

/// Object predicates (CR 608.2j: filters check only the stated characteristics).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Filter {
    Any,
    And(Vec<Filter>),
    Or(Vec<Filter>),
    Not(Box<Filter>),
    Type(CardType),
    Supertype(Supertype),
    Subtype(Subtype),
    /// Has at least one of the listed colors.
    Color(Color),
    /// Has exactly these colors.
    ExactColors(ColorSet),
    Colorless,
    Multicolored,
    Monocolored,
    /// Is a permanent (on the battlefield).
    Permanent,
    /// "permanent card" — a card with a permanent type (CR 110.4b).
    PermanentCard,
    /// A spell on the stack.
    Spell,
    /// Nonland permanents / cards etc. are expressed with Not(Type(Land)).
    Token,
    /// A card (not a token, not a copy of a card on the stack).
    Card,
    /// A copy (of a spell or card).
    Copy,
    ControlledBy(PlayerRel),
    OwnedBy(PlayerRel),
    /// In the given zone. Filters without a zone apply to the battlefield (for permanents)
    /// or the stack (for spells).
    InZone(ZoneKind),
    Tapped,
    Untapped,
    Attacking,
    Blocking,
    Blocked,
    Unblocked,
    /// "attacking you" / "attacking a planeswalker you control".
    AttackingPlayer(PlayerRel),
    /// "creature blocking it", "creature blocked by it" relative to the source.
    BlockingSource,
    BlockedBySource,
    Power(Cmp, Box<Value>),
    Toughness(Cmp, Box<Value>),
    ManaValue(Cmp, Box<Value>),
    /// Loyalty/defense comparisons.
    Loyalty(Cmp, Box<Value>),
    Named(SmolStr),
    /// Has the same name as an object in the selection.
    SameNameAs(Box<Sel>),
    /// Shares a creature type / card type / color with a selection.
    SharesCreatureType(Box<Sel>),
    SharesCardType(Box<Sel>),
    SharesColor(Box<Sel>),
    HasKeyword(KeywordKind),
    HasCounter(Option<CounterKind>),
    /// Has at least one ability (for "creature with no abilities" use Not).
    HasAbilities,
    /// The source object itself.
    Source,
    /// "another", "other": not the source.
    Other,
    /// A member of the selection.
    In(Box<Sel>),
    /// The object the source is attached to ("enchanted creature").
    AttachedToSource,
    /// Attached to something ("equipped", "enchanted").
    Attached,
    /// Has an Aura/Equipment attached ("enchanted creature" in "each enchanted creature").
    Enchanted,
    Equipped,
    /// Entered the battlefield this turn.
    EnteredThisTurn,
    /// Was dealt damage this turn.
    DealtDamageThisTurn,
    /// "historic": artifact, legendary, or Saga (CR 700.6).
    Historic,
    /// Face-down.
    FaceDown,
    /// Has a mana cost with {X}.
    HasX,
    /// Is a commander (CR 903.3).
    Commander,
    /// Modified: has counters, or is equipped/enchanted by a permanent its controller controls (CR 700.9).
    Modified,
    /// Power greater than its base power etc. can be added as needed.
    /// "creature card with mana value less than or equal to X" etc. use ManaValue.
    /// Cards in a graveyard that were put there from the battlefield this turn.
    DiedThisTurn,
    /// Attacked this turn.
    AttackedThisTurn,
    /// Is a basic land type, e.g. "nonbasic land" = Land and Not(Supertype(Basic)).
    /// Custom predicates implemented in code, by name.
    Custom(SmolStr),
}

impl Filter {
    pub fn creature() -> Filter {
        Filter::Type(CardType::Creature)
    }
    pub fn and(v: Vec<Filter>) -> Filter {
        let mut out = Vec::new();
        for f in v {
            match f {
                Filter::Any => {}
                Filter::And(inner) => out.extend(inner),
                other => out.push(other),
            }
        }
        match out.len() {
            0 => Filter::Any,
            1 => out.pop().unwrap(),
            _ => Filter::And(out),
        }
    }
    pub fn not(f: Filter) -> Filter {
        Filter::Not(Box::new(f))
    }
    pub fn you_control(self) -> Filter {
        Filter::and(vec![self, Filter::ControlledBy(PlayerRel::You)])
    }
    pub fn opp_controls(self) -> Filter {
        Filter::and(vec![self, Filter::ControlledBy(PlayerRel::Opponent)])
    }
    pub fn in_zone(self, z: ZoneKind) -> Filter {
        Filter::and(vec![self, Filter::InZone(z)])
    }
    pub fn other(self) -> Filter {
        Filter::and(vec![self, Filter::Other])
    }
    /// The zone this filter restricts to, if any.
    pub fn zone(&self) -> Option<ZoneKind> {
        match self {
            Filter::InZone(z) => Some(*z),
            Filter::Spell => Some(ZoneKind::Stack),
            Filter::Permanent => Some(ZoneKind::Battlefield),
            Filter::And(v) => v.iter().find_map(|f| f.zone()),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Values and conditions
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Value {
    Const(i32),
    /// The value of X chosen for the spell/ability (CR 107.3).
    X,
    /// Number of objects matching the filter.
    Count(Filter),
    /// Number of entities in a selection.
    CountSel(Box<Sel>),
    /// Number of players matching.
    CountPlayers(PlayerFilter),
    PowerOf(Box<Sel>),
    ToughnessOf(Box<Sel>),
    ManaValueOf(Box<Sel>),
    LoyaltyOf(Box<Sel>),
    CountersOn(Box<Sel>, Option<CounterKind>),
    PlayerCounters(PlayerRef, CounterKind),
    LifeTotal(PlayerRef),
    StartingLife,
    HandSize(PlayerRef),
    LibrarySize(PlayerRef),
    GraveyardSize(PlayerRef),
    /// Number of cards in a player's graveyard matching filter.
    CardsInGraveyard(PlayerRef, Filter),
    /// "that much" — the amount from the triggering event (damage, life, counters).
    EventAmount,
    /// Result of the previous effect in a sequence (damage dealt, cards milled, ...).
    Prev,
    /// Numeric variable.
    Var(Var),
    /// Devotion to colors (CR 700.5).
    Devotion(ColorSet),
    /// Number of basic land types among lands you control (domain).
    Domain,
    /// Number of spells cast this turn before this one (storm, CR 702.40).
    StormCount,
    /// Number of cards drawn this turn by a player.
    CardsDrawnThisTurn(PlayerRef),
    /// Life gained this turn by a player.
    LifeGainedThisTurn(PlayerRef),
    /// Life lost this turn by a player.
    LifeLostThisTurn(PlayerRef),
    /// Number of creatures that died this turn.
    CreaturesDiedThisTurn,
    /// Number of times this ability has resolved this turn.
    TimesResolvedThisTurn,
    /// Number of distinct card types among cards in graveyards etc.
    CardTypesAmong(Filter),
    /// Greatest power among objects matching.
    GreatestPower(Filter),
    GreatestManaValue(Filter),
    /// Number of different mana types spent to cast this spell (converge etc.).
    ColorsSpent,
    /// Amount of mana spent to cast this spell.
    ManaSpent,
    /// The number the player chose ("choose a number").
    Chosen,
    /// Kicker count.
    TimesKicked,
    /// Speed (CR 702.179).
    Speed(PlayerRef),
    Sum(Vec<Value>),
    Diff(Box<Value>, Box<Value>),
    Mul(Box<Value>, Box<Value>),
    /// Integer division, rounded up or down.
    Div(Box<Value>, i32, bool),
    Min(Box<Value>, Box<Value>),
    Max(Box<Value>, Box<Value>),
    /// Custom computed values implemented in code.
    Custom(SmolStr),
}

impl Value {
    pub fn c(n: i32) -> Value {
        Value::Const(n)
    }
    pub fn as_const(&self) -> Option<i32> {
        if let Value::Const(n) = self {
            Some(*n)
        } else {
            None
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Const(0)
    }
}

impl From<i32> for Value {
    fn from(n: i32) -> Self {
        Value::Const(n)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Condition {
    Always,
    Never,
    Not(Box<Condition>),
    And(Vec<Condition>),
    Or(Vec<Condition>),
    /// Compare two values.
    Compare(Value, Cmp, Value),
    /// At least one object matches ("if you control an artifact").
    Exists(Filter),
    /// The selection is non-empty.
    SelNonEmpty(Sel),
    /// All objects in the selection match the filter ("if it's a creature").
    SelMatches(Sel, Filter),
    /// A player matches.
    PlayerMatches(PlayerRef, PlayerFilter),
    /// It's the controller's turn.
    YourTurn,
    /// It's not the controller's turn.
    NotYourTurn,
    /// The spell was kicked / a particular additional or alternative cost was paid.
    CostPaid(SmolStr),
    /// Spell was cast (for ETB "if you cast it").
    WasCast,
    /// The previous "you may" / optional cost in this resolution was taken ("if you do").
    PrevHappened,
    /// The previous effect affected at least one object/did something ("if a creature died this way").
    PrevAffectedAny,
    /// The spell/ability was cast from the given zone.
    CastFrom(ZoneKind),
    /// Game-state predicates about the current turn.
    Phase(PhaseCond),
    /// You have the city's blessing (CR 702.131).
    CitysBlessing,
    /// You are the monarch.
    IsMonarch,
    /// You have the initiative.
    HasInitiative,
    /// It's day / night (CR 731).
    IsDay,
    IsNight,
    /// Source has max speed etc.
    MaxSpeed,
    /// This ability's source is tapped/untapped/attacking… via SelMatches(This, …).
    /// Custom conditions implemented in code.
    Custom(SmolStr),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhaseCond {
    Combat,
    MainPhase,
    Upkeep,
    DeclareAttackers,
    EndStep,
}

// ---------------------------------------------------------------------------
// Effects (CR 609–610)
// ---------------------------------------------------------------------------

/// How long a continuous effect from a resolving spell/ability lasts (CR 611.2).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Duration {
    /// "until end of turn" — ends in the cleanup step (CR 514.2).
    EndOfTurn,
    /// "until end of combat".
    EndOfCombat,
    /// "until your next turn".
    UntilYourNextTurn,
    /// "until the end of your next turn".
    UntilEndOfYourNextTurn,
    /// "for as long as [source] remains on the battlefield" / "for as long as you control".
    WhileSourceOnBattlefield,
    WhileYouControlSource,
    /// "for as long as [object] remains tapped" etc.
    WhileCondition(Condition),
    /// Indefinitely (e.g. "gain control of target creature").
    Permanent,
    /// Until the affected object leaves (used by Auras granting effects via resolution).
    UntilHostLeaves,
    /// "this turn" for rule-modifying effects — same as EndOfTurn.
    ThisTurn,
}

/// Layer-specific modifications of characteristics (CR 613).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Modification {
    // Layer 2
    SetController(PlayerRef),
    // Layer 3
    /// Replace one color word or basic land type with another in the text (CR 612).
    ChangeText {
        from: SmolStr,
        to: SmolStr,
    },
    // Layer 4
    AddTypes(Vec<CardType>),
    RemoveTypes(Vec<CardType>),
    AddSupertypes(Vec<Supertype>),
    RemoveSupertypes(Vec<Supertype>),
    AddSubtypes(Vec<Subtype>),
    RemoveSubtypes(Vec<Subtype>),
    /// Set card types (removes others) but keep supertypes unless listed.
    SetTypes {
        types: Vec<CardType>,
        subtypes: Vec<Subtype>,
    },
    /// "is every creature type" (changeling, CR 702.73).
    AllCreatureTypes,
    RemoveAllCreatureTypes,
    /// Lands become a basic land type, losing other land types (CR 305.7).
    SetBasicLandType(Vec<Subtype>),
    // Layer 5
    SetColors(ColorSet),
    AddColors(ColorSet),
    // Layer 6
    AddAbility(Ability),
    AddKeyword(Keyword),
    RemoveKeyword(KeywordKind),
    RemoveAllAbilities,
    /// "can't have or gain [ability]".
    CantHaveKeyword(KeywordKind),
    // Layer 7
    /// 7a: characteristic-defining P/T.
    CdaPT(Option<Value>, Option<Value>),
    /// 7b: set base power/toughness.
    SetPT(Option<Value>, Option<Value>),
    /// 7c: +N/+N.
    ModifyPT(Value, Value),
    /// 7d: switch.
    SwitchPT,
}

impl Modification {
    /// The layer (1–7) and sublayer ordinal this modification applies in.
    pub fn layer(&self) -> Layer {
        use Modification::*;
        match self {
            SetController(_) => Layer::L2Control,
            ChangeText { .. } => Layer::L3Text,
            AddTypes(_)
            | RemoveTypes(_)
            | AddSupertypes(_)
            | RemoveSupertypes(_)
            | AddSubtypes(_)
            | RemoveSubtypes(_)
            | SetTypes { .. }
            | AllCreatureTypes
            | RemoveAllCreatureTypes
            | SetBasicLandType(_) => Layer::L4Type,
            SetColors(_) | AddColors(_) => Layer::L5Color,
            AddAbility(_) | AddKeyword(_) | RemoveKeyword(_) | RemoveAllAbilities
            | CantHaveKeyword(_) => Layer::L6Ability,
            CdaPT(..) => Layer::L7aCda,
            SetPT(..) => Layer::L7bSet,
            ModifyPT(..) => Layer::L7cModify,
            SwitchPT => Layer::L7dSwitch,
        }
    }
}

/// Layers and sublayers in application order (CR 613.1–613.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Layer {
    L1aCopy,
    L1bFaceDown,
    L2Control,
    L3Text,
    L4Type,
    L5Color,
    L6Ability,
    L7aCda,
    L7bSet,
    L7cModify,
    L7dSwitch,
}

impl Layer {
    pub const ALL: [Layer; 11] = [
        Layer::L1aCopy,
        Layer::L1bFaceDown,
        Layer::L2Control,
        Layer::L3Text,
        Layer::L4Type,
        Layer::L5Color,
        Layer::L6Ability,
        Layer::L7aCda,
        Layer::L7bSet,
        Layer::L7cModify,
        Layer::L7dSwitch,
    ];
}

/// Description of a token to create (CR 111).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenSpec {
    pub name: SmolStr,
    pub colors: ColorSet,
    pub supertypes: Vec<Supertype>,
    pub card_types: Vec<CardType>,
    pub subtypes: Vec<Subtype>,
    pub power: Option<i32>,
    pub toughness: Option<i32>,
    pub abilities: Vec<Ability>,
    /// Name of a Scryfall token card to copy characteristics from, when available.
    pub scryfall_name: Option<SmolStr>,
}

/// What mana an effect adds (CR 106).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ManaProduction {
    /// Fixed mana, e.g. {G} or {C}{C}.
    Fixed(Vec<ManaType>),
    /// N mana of any one color (chosen).
    AnyOneColor(Value),
    /// N mana in any combination of colors.
    AnyCombination(Value),
    /// One mana of one of the listed types (chosen).
    OneOf(Vec<ManaType>),
    /// Mana of any type that a land an opponent controls could produce, etc.
    CouldProduce(Filter),
    /// N mana of the chosen color (stored on the source, e.g. "the chosen color").
    ChosenColor(Value),
    /// N mana of a fixed type.
    Amount(ManaType, Value),
    /// Mana of any color among the colors of the selected objects (commander identity etc.).
    AnyColorAmong(Filter),
}

/// Replacement effect definitions (CR 614–616).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplacementDef {
    pub event: ReplacementEvent,
    pub action: ReplacementAction,
    /// Self-replacement effects apply first (CR 614.15).
    pub self_replacement: bool,
    /// Optional replacement ("you may").
    pub optional: bool,
}

/// Which events a replacement effect watches (relative to its source).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ReplacementEvent {
    /// An object matching the filter would enter the battlefield (CR 614.1c–d).
    EntersBattlefield(Filter),
    /// An object would be put into a zone from another zone.
    ZoneChange {
        filter: Filter,
        from: Option<ZoneKind>,
        to: Option<ZoneKind>,
    },
    /// A permanent matching would die (battlefield → graveyard).
    Dies(Filter),
    /// A player matching would draw a card.
    Draw(PlayerFilter),
    /// Damage would be dealt. `source`/`target` filter the damage event.
    Damage {
        source: Filter,
        to_players: Option<PlayerFilter>,
        to_objects: Option<Filter>,
        combat_only: bool,
    },
    /// A player would gain life.
    GainLife(PlayerFilter),
    /// A player would lose life.
    LoseLife(PlayerFilter),
    /// Counters would be put on an object/player.
    PutCounters {
        on_objects: Option<Filter>,
        on_players: Option<PlayerFilter>,
        kind: Option<CounterKind>,
    },
    /// One or more tokens would be created under a player's control.
    CreateTokens(PlayerFilter),
    /// A permanent would be destroyed.
    Destroy(Filter),
    /// Would lose the game.
    LoseGame(PlayerFilter),
    /// A step/phase would begin for a player ("skip your draw step").
    SkipStep { step: StepKind, whose: PlayerRel },
    /// Adding mana ("if a land you control would produce mana, it produces twice as much").
    ProduceMana(Filter),
    /// A permanent would untap during its controller's untap step.
    UntapDuringUntapStep(Filter),
    /// A spell or ability would be countered.
    Countered(Filter),
    /// Discard.
    Discard(PlayerFilter, Filter),
    /// Mill.
    Mill(PlayerFilter),
    /// "If you would search your library".
    Search(PlayerFilter),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StepKind {
    Untap,
    Upkeep,
    Draw,
    Main,
    Combat,
    End,
    Turn,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ReplacementAction {
    /// Enters tapped (CR 614.1c).
    EnterTapped,
    /// Enters with counters.
    EnterWithCounters(CounterKind, Value),
    /// "As this enters, choose ..." — perform the effect as it enters (the choice is stored on the object).
    AsEnters(Box<Effect>),
    /// Enters as a copy of (chosen) object (CR 707.9).
    EnterAsCopy { filter: Filter, optional: bool },
    /// Enters under another player's control.
    EnterUnderControl(PlayerRef),
    /// Put into a different zone instead ("exile it instead").
    MoveInstead(Destination),
    /// Nothing happens (skip, prevention of the entire event).
    Prevent,
    /// Prevent N damage (a shield that is used up).
    PreventAmount(Value),
    /// Multiply the amount (double damage, double tokens, double counters) ×N.
    Multiply(i32),
    /// Add N to the amount.
    Add(Value),
    /// Subtract N from the amount.
    Subtract(Value),
    /// Redirect damage to the selection.
    Redirect(Sel),
    /// Do something else entirely instead.
    Instead(Box<Effect>),
    /// Perform the original event and then an additional effect.
    Also(Box<Effect>),
    /// Regeneration shield (CR 701.19).
    Regenerate,
}

/// Rule-modifying effects (CR 613.11): restrictions and requirements.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Restriction {
    CantAttack(Filter),
    CantBlock(Filter),
    /// "can't attack or block".
    CantAttackOrBlock(Filter),
    /// "can't attack you or planeswalkers you control".
    CantAttackPlayer {
        attackers: Filter,
        defender: PlayerFilter,
    },
    /// "attacks each combat if able".
    MustAttack(Filter),
    /// "blocks each combat if able".
    MustBlock(Filter),
    /// "must be blocked if able" / lure.
    MustBeBlocked(Filter),
    /// Evasion: "can't be blocked" (or except by filter).
    CantBeBlocked(Filter),
    CantBeBlockedBy {
        attacker: Filter,
        blocker: Filter,
    },
    /// "can't be blocked except by two or more creatures" (menace-like).
    MinBlockers {
        attacker: Filter,
        n: u32,
    },
    /// "can block an additional creature each combat" / "any number".
    ExtraBlocks {
        blocker: Filter,
        n: Option<u32>,
    },
    /// "can block only creatures with flying".
    CanBlockOnly {
        blocker: Filter,
        attackers: Filter,
    },
    CantBeTargeted {
        what: Filter,
        by: TargetRestriction,
    },
    PlayerCantBeTargeted {
        who: PlayerFilter,
        by: TargetRestriction,
    },
    /// "can't cast spells" (matching filter).
    CantCast {
        who: PlayerFilter,
        what: Filter,
    },
    /// "can't activate abilities" of matching sources (non-mana unless specified).
    CantActivate {
        who: PlayerFilter,
        sources: Filter,
        include_mana: bool,
    },
    /// "can't be countered".
    CantBeCountered(Filter),
    /// "doesn't untap during its controller's untap step".
    DoesntUntap(Filter),
    /// "can't gain life".
    CantGainLife(PlayerFilter),
    /// "can't lose life".
    CantLoseLife(PlayerFilter),
    /// "can't lose the game" / "can't win the game".
    CantLoseGame(PlayerFilter),
    CantWinGame(PlayerFilter),
    /// "can't draw more than one card each turn" etc.
    MaxDrawsPerTurn(PlayerFilter, u32),
    /// "can't cast more than one spell each turn".
    MaxSpellsPerTurn(PlayerFilter, u32),
    /// "can't be sacrificed".
    CantBeSacrificed(Filter),
    /// "can't be the target of spells or abilities your opponents control" is CantBeTargeted.
    /// "damage can't be prevented".
    DamageCantBePrevented,
    /// "can't transform".
    CantTransform(Filter),
    /// "can't search libraries".
    CantSearch(PlayerFilter),
    /// Cast spells only at sorcery speed etc.
    SorcerySpeedOnly(PlayerFilter),
    /// "can't play lands".
    CantPlayLands(PlayerFilter),
    /// "can't block creatures with power greater than this"...
    Custom(SmolStr),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TargetRestriction {
    /// Hexproof-like: by spells/abilities opponents control.
    Opponents,
    /// Shroud-like: by any spell or ability.
    Any,
    /// By sources matching the filter (protection-like).
    Sources(Filter),
}

/// Cost modification static effects (CR 601.2f).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CostModifier {
    /// Spells (or abilities) affected.
    pub applies_to: CostTarget,
    /// Whose spells ("spells you cast", "spells your opponents cast").
    pub who: PlayerRel,
    pub change: CostChange,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CostTarget {
    Spells(Filter),
    /// Activated abilities of sources matching.
    Abilities(Filter),
    /// The source itself (a spell that costs less, e.g. affinity-like).
    ThisSpell,
    /// Specific: "Equip abilities", "ninjutsu abilities".
    Keyword(KeywordKind),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CostChange {
    /// Costs {N} more.
    IncreaseGeneric(Value),
    /// Costs {N} less (generic only).
    ReduceGeneric(Value),
    /// Costs specific mana more ("costs {W} more").
    IncreaseMana(ManaCost),
    /// Costs specific colored mana less.
    ReduceColored(Color, Value),
    /// Additional non-mana cost ("As an additional cost to cast spells, pay 2 life").
    AdditionalCost(Cost),
    /// "You may pay X rather than pay this spell's mana cost."
    AlternativeCost(Cost),
}

/// Static abilities (CR 604) and what they do.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StaticEffect {
    /// A characteristic-changing continuous effect applied to objects matching `affected`
    /// (relative to the source).
    Continuous {
        affected: Filter,
        mods: Vec<Modification>,
    },
    /// Effects on players ("You have hexproof", "Your maximum hand size is ...").
    PlayerEffect {
        affected: PlayerFilter,
        effect: PlayerModification,
    },
    Restriction(Restriction),
    CostModifier(CostModifier),
    Replacement(ReplacementDef),
    /// Permission to play/cast cards from a zone ("You may play lands from the top of your
    /// library", "You may cast spells from your graveyard").
    PlayPermission(PlayPermission),
    /// Can be cast as though it had flash (a filter of spells).
    FlashPermission {
        who: PlayerRel,
        what: Filter,
    },
    /// "You may look at the top card of your library any time."
    LookAtTopCard(PlayerRel),
    /// "Play with the top card of your library revealed."
    RevealTopCard(PlayerRel),
    /// Additional land plays per turn.
    AdditionalLandPlays(PlayerRel, u32),
    /// "Enchant [filter]" is a keyword; this covers "can be attached only to ..." etc.
    /// Mana abilities that tap for more: handled via Replacement(ProduceMana).
    /// "Prevent all combat damage that would be dealt ..." is a Replacement.
    /// Custom behavior implemented in code, by name.
    Custom(SmolStr),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PlayerModification {
    Hexproof,
    Shroud,
    ProtectionFrom(Filter),
    MaxHandSize(Option<Value>),
    /// Hand size modifier (+N).
    HandSizeDelta(i32),
    /// "You may play an additional land on each of your turns."
    AdditionalLandPlays(u32),
    CantLoseGame,
    /// "Spells you cast have ..." etc. are handled elsewhere.
    /// Skip draw step etc. handled via replacements.
    /// "You can't be attacked", etc.
    Custom(SmolStr),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayPermission {
    pub who: PlayerRel,
    pub zone: ZoneKind,
    /// Only the top card of the library, if zone is Library.
    pub top_only: bool,
    pub what: Filter,
    /// Lands can be played.
    pub lands: bool,
    /// Spells can be cast.
    pub spells: bool,
    /// Alternative cost, e.g. "without paying its mana cost" or "by paying life".
    pub cost: Option<Cost>,
}

/// Trigger events (CR 603). Filters are relative to the ability's source.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TriggerCond {
    /// "When/Whenever [filter] enters" (CR 603.6a).
    EntersBattlefield(Filter),
    /// "When/Whenever [filter] leaves the battlefield" (look back in time, CR 603.10a).
    LeavesBattlefield(Filter),
    /// "When/Whenever [filter] dies".
    Dies(Filter),
    /// General zone change.
    ZoneChange {
        filter: Filter,
        from: Option<ZoneKind>,
        to: Option<ZoneKind>,
    },
    /// "Whenever [player] casts a [filter] spell".
    CastSpell {
        who: PlayerRel,
        filter: Filter,
    },
    /// "Whenever you cast your Nth spell each turn".
    NthSpellCast {
        who: PlayerRel,
        n: u32,
    },
    /// "Whenever [player] activates an ability" (of matching source).
    AbilityActivated {
        who: PlayerRel,
        source: Filter,
        include_mana: bool,
    },
    /// "Whenever [filter] attacks".
    Attacks(Filter),
    /// "Whenever you attack" / "Whenever [player] attacks" (one or more creatures).
    PlayerAttacks(PlayerRel),
    /// "Whenever [filter] attacks and isn't blocked".
    AttacksUnblocked(Filter),
    /// "Whenever [filter] blocks" / "blocks a creature".
    Blocks(Filter),
    /// "Whenever [filter] becomes blocked".
    BecomesBlocked(Filter),
    /// "Whenever [filter] blocks or becomes blocked".
    BlocksOrBecomesBlocked(Filter),
    /// "Whenever [source filter] deals (combat) damage to [recipient]".
    DealsDamage {
        source: Filter,
        to: DamageRecipient,
        combat_only: bool,
    },
    /// "Whenever [filter] is dealt damage".
    IsDealtDamage {
        filter: Filter,
        combat_only: bool,
    },
    /// "Whenever [player] is dealt damage".
    PlayerDealtDamage {
        who: PlayerRel,
        combat_only: bool,
    },
    /// "At the beginning of [whose] [step]".
    BeginningOf {
        step: TriggerStep,
        whose: PlayerRel,
    },
    Draws {
        who: PlayerRel,
    },
    Discards {
        who: PlayerRel,
        filter: Filter,
    },
    GainsLife {
        who: PlayerRel,
    },
    LosesLife {
        who: PlayerRel,
    },
    CountersPut {
        filter: Filter,
        kind: Option<CounterKind>,
    },
    CountersRemoved {
        filter: Filter,
        kind: Option<CounterKind>,
    },
    BecomesTapped(Filter),
    BecomesUntapped(Filter),
    /// "Whenever [filter] becomes the target of a spell or ability [opponent controls]".
    BecomesTarget {
        filter: Filter,
        by: PlayerRel,
    },
    Sacrificed(Filter),
    TokenCreated(Filter),
    LandPlayed {
        who: PlayerRel,
        filter: Filter,
    },
    /// "Whenever you cycle or discard" / "When you cycle this card".
    Cycled {
        who: PlayerRel,
        filter: Filter,
    },
    /// "Whenever a player searches their library".
    Searched(PlayerRel),
    /// "Whenever [filter] is turned face up".
    TurnedFaceUp(Filter),
    Transforms(Filter),
    /// "Whenever you sacrifice [filter]".
    YouSacrifice(Filter),
    /// "Whenever a spell or ability an opponent controls causes you to discard" etc. use Custom.
    /// State triggers (CR 603.8): "When you control no Islands, sacrifice this".
    State(Condition),
    /// "Whenever you gain control of" etc.
    ControlChanged(Filter),
    /// "Whenever a player loses the game".
    PlayerLoses,
    /// "Whenever you roll a die".
    RollDie(PlayerRel),
    /// "Whenever you flip a coin" / "win a coin flip".
    FlipCoin(PlayerRel),
    /// "Whenever [player] mills"/"cards are put into graveyard from library".
    Mills(PlayerRel),
    /// "Whenever you commit a crime" (CR 700.13).
    CommitCrime(PlayerRel),
    /// "Whenever day becomes night or night becomes day".
    DayNightChanges,
    /// "Whenever you expend N" (CR 700.14).
    Expend {
        who: PlayerRel,
        n: u32,
    },
    /// Keyword-provided and card-specific triggers implemented in code, by name.
    Custom(SmolStr),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DamageRecipient {
    Any,
    Player(PlayerRel),
    /// A creature/permanent matching.
    Object(Filter),
    PlayerOrPlaneswalker(PlayerRel),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TriggerStep {
    Untap,
    Upkeep,
    Draw,
    PrecombatMain,
    BeginningOfCombat,
    DeclareAttackers,
    DeclareBlockers,
    CombatDamage,
    EndOfCombat,
    PostcombatMain,
    End,
    Cleanup,
    /// "At the beginning of each turn".
    Turn,
}

/// Instructions performed as a spell or ability resolves.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum Effect {
    #[default]
    Noop,
    Seq(Vec<Effect>),
    If {
        cond: Condition,
        then: Box<Effect>,
        otherwise: Box<Effect>,
    },
    /// "[Player] may [effect]" — the result is recorded for `Condition::PrevHappened`.
    May {
        who: PlayerRef,
        effect: Box<Effect>,
    },
    /// "[Player] may pay [cost]. If they do, [then]. If they don't, [otherwise]."
    /// Also covers "unless [player] pays" (swap then/otherwise).
    PayOptional {
        who: PlayerRef,
        cost: Cost,
        then: Box<Effect>,
        otherwise: Box<Effect>,
    },
    /// Perform for each entity in the selection, binding it to `var`.
    ForEach {
        sel: Sel,
        var: Var,
        effect: Box<Effect>,
    },
    /// For each player (APNAP order), binding `PlayerRef::Iterated`.
    ForEachPlayer {
        who: PlayerRef,
        effect: Box<Effect>,
    },
    Repeat {
        times: Value,
        effect: Box<Effect>,
    },
    /// Choose one of several effects at resolution ("choose one —" when not modal on cast,
    /// or "choose one at random").
    ChooseOne {
        who: PlayerRef,
        options: Vec<(String, Effect)>,
    },
    /// Store a selection into a variable (evaluated now).
    Store {
        var: Var,
        sel: Sel,
    },
    /// Store a number into a variable.
    StoreValue {
        var: Var,
        value: Value,
    },

    // --- Objects -----------------------------------------------------------
    Destroy {
        what: Sel,
        no_regen: bool,
    },
    Exile {
        what: Sel,
        face_down: bool,
        /// Link exiled cards to the source (CR 607, "exiled with this").
        link: bool,
    },
    /// "[Player] sacrifices [count] [filter]" — the player chooses.
    Sacrifice {
        who: PlayerRef,
        filter: Filter,
        count: Value,
    },
    /// Sacrifice specific objects (e.g. "sacrifice this creature").
    SacrificeObjects {
        what: Sel,
    },
    /// Move objects to a destination ("return to owner's hand", "put on top of library",
    /// "put onto the battlefield").
    Move {
        what: Sel,
        to: Destination,
    },
    Tap {
        what: Sel,
    },
    Untap {
        what: Sel,
    },
    DealDamage {
        source: Sel,
        amount: Value,
        to: Sel,
    },
    /// Divided damage using the division chosen on casting (CR 601.2d).
    DealDividedDamage {
        source: Sel,
        slot: u8,
    },
    Fight {
        a: Sel,
        b: Sel,
    },
    AddCounters {
        what: Sel,
        kind: CounterKind,
        n: Value,
    },
    RemoveCounters {
        what: Sel,
        kind: Option<CounterKind>,
        n: Value,
    },
    /// Apply layer modifications to the selected objects for a duration (CR 611.2).
    Modify {
        what: Sel,
        mods: Vec<Modification>,
        duration: Duration,
    },
    /// Create a rule-modifying effect (restriction/requirement) for a duration.
    AddRestriction {
        restriction: Restriction,
        duration: Duration,
    },
    /// Create a player effect for a duration.
    AddPlayerEffect {
        who: PlayerRef,
        effect: PlayerModification,
        duration: Duration,
    },
    /// Create a replacement/prevention effect for a duration (or until used).
    AddReplacement {
        def: ReplacementDef,
        duration: Duration,
        /// Number of times it can apply before it's used up.
        uses: Option<u32>,
    },
    GainControl {
        what: Sel,
        who: PlayerRef,
        duration: Duration,
    },
    ExchangeControl {
        a: Sel,
        b: Sel,
    },
    CreateToken {
        spec: TokenSpec,
        count: Value,
        controller: PlayerRef,
        tapped: bool,
        attacking: bool,
    },
    /// Create token copies of objects (CR 707.2, 111.10).
    CreateTokenCopy {
        of: Sel,
        count: Value,
        controller: PlayerRef,
        tapped: bool,
        attacking: bool,
        /// Exceptions ("except it's a 1/1", "except it has haste").
        mods: Vec<Modification>,
    },
    CounterSpell {
        what: Sel,
    },
    CopySpell {
        what: Sel,
        count: Value,
        new_targets: bool,
    },
    /// "[this] becomes a copy of [object]" (CR 707).
    BecomeCopy {
        what: Sel,
        of: Sel,
        duration: Duration,
    },
    Transform {
        what: Sel,
    },
    Regenerate {
        what: Sel,
    },
    /// Attach the source (or selection) to a target (CR 701.3).
    Attach {
        what: Sel,
        to: Sel,
    },
    Unattach {
        what: Sel,
    },
    PhaseOut {
        what: Sel,
    },
    /// Turn face down / face up.
    TurnFaceUp {
        what: Sel,
    },
    /// Put a creature/permanent into its owner's library at a position.
    /// (Use Move.)
    /// Remove from combat.
    RemoveFromCombat {
        what: Sel,
    },
    /// "Choose a color/creature type/card name/number" — stored on the source object.
    Choose {
        who: PlayerRef,
        kind: ChoiceKind,
    },

    // --- Players ------------------------------------------------------------
    Draw {
        who: PlayerRef,
        n: Value,
    },
    Discard {
        who: PlayerRef,
        n: Value,
        random: bool,
        filter: Filter,
    },
    DiscardHand {
        who: PlayerRef,
    },
    Mill {
        who: PlayerRef,
        n: Value,
    },
    GainLife {
        who: PlayerRef,
        n: Value,
    },
    LoseLife {
        who: PlayerRef,
        n: Value,
    },
    SetLife {
        who: PlayerRef,
        n: Value,
    },
    AddMana {
        who: PlayerRef,
        mana: ManaProduction,
        restriction: Option<ManaRestriction>,
    },
    AddPlayerCounters {
        who: PlayerRef,
        kind: CounterKind,
        n: Value,
    },
    Scry {
        who: PlayerRef,
        n: Value,
    },
    Surveil {
        who: PlayerRef,
        n: Value,
    },
    /// Search a library for cards matching filter and put them into a destination.
    Search {
        who: PlayerRef,
        whose: PlayerRef,
        filter: Filter,
        count: Value,
        to: Destination,
        reveal: bool,
        shuffle: bool,
    },
    Shuffle {
        who: PlayerRef,
    },
    /// Shuffle objects into their owners' libraries.
    ShuffleInto {
        what: Sel,
    },
    /// Reveal/look at the top N cards and choose some to put somewhere
    /// ("Look at the top N cards, put M of them into your hand and the rest on the bottom").
    Dig {
        who: PlayerRef,
        n: Value,
        reveal: bool,
        filter: Filter,
        take: Value,
        take_up_to: bool,
        take_to: Destination,
        rest_to: Destination,
    },
    RevealHand {
        who: PlayerRef,
    },
    /// Reveal cards from the top until a card matching filter is revealed.
    RevealUntil {
        who: PlayerRef,
        filter: Filter,
        found_to: Destination,
        rest_to: Destination,
    },
    ExtraTurn {
        who: PlayerRef,
    },
    ExtraCombat {
        after_this: bool,
    },
    /// Skip a player's next step/phase/turn.
    Skip {
        who: PlayerRef,
        step: StepKind,
    },
    /// Create a delayed triggered ability (CR 603.7).
    DelayedTrigger {
        trigger: TriggerCond,
        body: Box<Body>,
        /// Fires once and is then removed.
        once: bool,
    },
    /// "At the beginning of the next end step" / "at end of combat" (common delayed triggers).
    AtNext {
        step: TriggerStep,
        effect: Box<Effect>,
    },
    CreateEmblem {
        abilities: Vec<Ability>,
    },
    /// Win or lose the game.
    WinGame {
        who: PlayerRef,
    },
    LoseGame {
        who: PlayerRef,
    },
    /// Cast a card during resolution (CR 608.2g), optionally without paying its mana cost.
    CastCard {
        who: PlayerRef,
        what: Sel,
        free: bool,
        optional: bool,
    },
    /// Play a land / cast a spell from exile etc. later: grant a play permission to the
    /// selected cards for a duration.
    GrantPlayPermission {
        who: PlayerRef,
        what: Sel,
        duration: Duration,
        free: bool,
    },
    /// Prevent the next N damage / all damage (CR 615).
    PreventDamage {
        to: Sel,
        amount: Option<Value>,
        duration: Duration,
        combat_only: bool,
    },
    /// Become the monarch (CR 725).
    BecomeMonarch {
        who: PlayerRef,
    },
    TakeInitiative {
        who: PlayerRef,
    },
    /// Keyword actions implemented in code (proliferate, explore, amass, connive, ...).
    KeywordAction {
        action: KeywordAction,
        who: PlayerRef,
        what: Sel,
        n: Value,
    },
    /// Card-specific behavior implemented in code, by name.
    Custom(SmolStr),
}

impl Effect {
    pub fn seq(v: Vec<Effect>) -> Effect {
        let mut out = Vec::new();
        for e in v {
            match e {
                Effect::Noop => {}
                Effect::Seq(inner) => out.extend(inner),
                other => out.push(other),
            }
        }
        match out.len() {
            0 => Effect::Noop,
            1 => out.pop().unwrap(),
            _ => Effect::Seq(out),
        }
    }
}

/// Choices stored on the source ("the chosen color", "the chosen creature type").
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChoiceKind {
    Color,
    CreatureType,
    CardName,
    /// A card name of a nonland card, etc.
    CardNameFiltered(String),
    Number {
        min: i32,
        max: i32,
    },
    Opponent,
    Player,
    BasicLandType,
    CardType,
    OddOrEven,
}

/// Keyword actions (CR 701) that aren't expressible as simple effect sequences.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeywordAction {
    Proliferate,
    Investigate,
    Explore,
    Amass,
    Connive,
    Goad,
    Populate,
    Bolster,
    Adapt,
    Monstrosity,
    Manifest,
    ManifestDread,
    Cloak,
    Support,
    Detain,
    Fateseal,
    Clash,
    Venture,
    Incubate,
    Discover,
    CollectEvidence,
    Suspect,
    Forage,
    Endure,
    TimeTravel,
    Learn,
    TheRingTemptsYou,
    OpenAttraction,
    RollAttractions,
    Planeswalk,
    SetInMotion,
    Abandon,
    Vote,
    Exert,
    Meld,
    Convert,
    Double,
    Triple,
    Assemble,
    VillainousChoice,
    Harness,
    Airbend,
    Earthbend,
    Waterbend,
    Blight,
    Heal,
    Recruit,
    EmpowerJace,
    Behold,
    Exchange,
    Seek,
    Conjure,
    Specialize,
    Perpetually,
    Mutate,
    Other,
}
