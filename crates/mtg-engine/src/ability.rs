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
    /// The controller chooses modes whose total number of pawprint symbols is at most this
    /// many ("Choose up to five {P} worth of modes", CR 107.18, 700.2i).
    Pawprints(u32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mode {
    pub text: String,
    pub targets: Vec<TargetSpec>,
    pub effect: Effect,
    /// Spree / tiered additional cost for choosing this mode.
    pub cost: Option<Cost>,
}

impl Mode {
    /// The number of pawprint symbols ({P}) listed for this mode (CR 107.18, 700.2i):
    /// they indicate the mode and aren't a cost.
    pub fn pawprints(&self) -> u32 {
        let head = self.text.split('—').next().unwrap_or("");
        head.matches("{P}").count() as u32
    }
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
    /// Other combat timing windows: "Activate only before attackers are declared",
    /// "only during combat after blockers are declared", ... (CR 506.8, 506.8g).
    CombatWindow(CombatTiming),
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
    /// Functions everywhere except the given zone, even outside the game (an ability that
    /// states which zones it doesn't function in, CR 113.6c).
    AnywhereExcept(ZoneKind),
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
    /// "This ability can't be countered." — an instruction that functions while the
    /// ability is on the stack (CR 603.1a).
    #[serde(default)]
    pub cant_be_countered: bool,
    /// "[...] Do this only once each turn.": the ability triggers only if its source's
    /// controller hasn't taken the optional action this turn (CR 603.2h).
    #[serde(default)]
    pub do_once_per_turn: bool,
}

/// Whether a triggered ability with this trigger and body is a mana ability (CR 605.1b):
/// it triggers from resolving a mana ability ("is tapped for mana"), has no targets, and
/// could add mana.
pub fn is_triggered_mana_ability(trigger: &TriggerCond, body: &Body) -> bool {
    fn from_mana_ability(t: &TriggerCond) -> bool {
        match t {
            TriggerCond::TappedForMana { .. } | TriggerCond::TappedForManaOfType { .. } => true,
            TriggerCond::ThisTurn(t) | TriggerCond::Where { trigger: t, .. } => {
                from_mana_ability(t)
            }
            _ => false,
        }
    }
    // CR 605.1b: unlike an activated mana ability (CR 605.1a), a triggered mana ability
    // may also move cards to or from a library ("add {G} and draw a card").
    fn could_add_mana(e: &Effect) -> bool {
        match e {
            Effect::AddMana { .. } => true,
            Effect::Seq(v) => v.iter().any(could_add_mana),
            Effect::ChooseOne { options, .. } => options.iter().all(|(_, e)| could_add_mana(e)),
            _ => false,
        }
    }
    from_mana_ability(trigger)
        && body.targets.is_empty()
        && body.modal.is_none()
        && could_add_mana(&body.effect)
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
            cant_be_countered: false,
            do_once_per_turn: false,
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
    /// "pay its mana cost": the mana cost of the selected object, with X as 0 unless the
    /// object is a spell on the stack (CR 107.3h).
    PayManaCostOf(Box<Sel>),
    /// "[cost] for each [thing]": the cost repeated a number of times determined as it's
    /// paid, with any choices made separately for each repetition, paid all at once or not
    /// at all (cumulative upkeep, CR 702.24a).
    Repeated {
        cost: Box<Cost>,
        times: Value,
    },
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
    /// Battlefield: "that permanent is [characteristic]" — a continuous effect of the
    /// resolving spell or ability that applies as it enters (CR 611.2e).
    #[serde(default)]
    pub with_mods: Vec<Modification>,
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
            with_mods: vec![],
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
    /// On the bottom, several cards in a random order (a single card: the bottom).
    BottomRandom,
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
    /// The target is required only if this condition holds as targets are chosen, e.g.
    /// only if a kicker cost was paid (CR 601.2c).
    #[serde(default)]
    pub condition: Option<Condition>,
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
            condition: None,
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
/// The link shared by a pregame choice and the characteristic-defining ability that refers
/// to it (CR 607.2p). Choices made for it are kept as the card changes zones.
pub const PREGAME_LINK: u16 = 0x7fff;

pub mod vars {
    use super::Var;
    /// Objects affected by the most recent effect ("it", "those cards", "the exiled card").
    pub const IT: Var = 0;
    /// Objects created by the most recent effect (tokens).
    pub const CREATED: Var = 1;
    /// Cards drawn/revealed/looked at.
    pub const REVEALED: Var = 2;
    /// Objects dealt damage by the most recent damage effect ("a creature dealt damage
    /// this way").
    pub const DAMAGED: Var = 3;
    /// Permanents sacrificed to pay the cost of the resolving spell or ability, or by an
    /// earlier instruction of it ("the sacrificed creature", last known information).
    pub const SACRIFICED: Var = 9;
    /// First user-defined variable.
    pub const USER: Var = 10;
    /// The object a static ability's continuous effect is being applied to, while its
    /// values are evaluated ("each creature you control gets +1/+1 for each +1/+1 counter
    /// on it").
    pub const AFFECTED: Var = 9;
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
    /// Objects linked to the ability of the object that created this token or put this
    /// permanent onto the battlefield (CR 607.1d): "the card exiled with [that object]".
    CreatorLinked,
    /// Cards the controller exiled before the game began with abilities of cards with this
    /// name ("a card you exiled with cards named [name]", CR 607.2n).
    ExiledWithCardsNamed(SmolStr),
    /// The spell this ability resolves for (for "copy that spell").
    TriggerSpell,
    /// Union of selections.
    Union(Vec<Sel>),
    /// The top card of a player's graveyard.
    TopOfGraveyard(PlayerRef),
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
    /// Graveyard size comparisons ("an opponent has eight or more cards in their
    /// graveyard"), counting cards only (CR 108.2b).
    GraveyardSize(Cmp, Box<Value>),
    /// The monarch.
    Monarch,
    /// Controls a number of permanents matching the filter ("controls an Island",
    /// "controls fewer creatures than you").
    Controls(Box<Filter>, Cmp, Box<Value>),
    /// Has a number of counters of a kind ("is poisoned": one or more poison counters,
    /// CR 122.1f).
    Counters(SmolStr, Cmp, Box<Value>),
    /// The defending player.
    Defending,
    /// The active player.
    Active,
    /// A player with one or more poison counters (CR 122.1f).
    Poisoned,
    /// One of the players a reference resolves to ("enchanted player").
    Ref(Box<PlayerRef>),
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
    /// The player chosen as target in slot N, or the controller of the permanent chosen
    /// there ("each creature that player or that planeswalker's controller controls").
    TargetOrController(u8),
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
    /// The player or opponent chosen for the source ("the chosen player", CR 607.2d).
    Chosen,
}

/// The four kinds of stickers (CR 123.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StickerType {
    Name,
    Ability,
    PowerToughness,
    Art,
}

/// What a spell or ability on the stack targets (CR 115.9).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TargetsFilter {
    /// "with [N] target(s)": the number of times objects or players were chosen as its
    /// targets, not how many are still legal (CR 115.9a).
    Count(u32),
    /// "that targets [object or player]": some current target matches (CR 115.9b).
    Targets {
        objects: Option<Filter>,
        players: Option<PlayerFilter>,
    },
    /// "that targets only [object or player]": exactly one different object or player was
    /// chosen as its target(s), and it matches (CR 115.9c).
    Only {
        objects: Option<Filter>,
        players: Option<PlayerFilter>,
    },
}

/// A special action granted by a static ability (CR 116.2d, 116.2e) or by an effect
/// (CR 116.2c).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpecialActionDef {
    /// Who may take it (relative to the source's controller).
    pub who: PlayerFilter,
    /// What taking it costs.
    pub cost: Cost,
    pub action: SpecialActionEffect,
}

/// What a special action does.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SpecialActionEffect {
    /// An effect, carried out immediately without using the stack.
    Effect(Effect),
    /// "For that player to ignore this effect until end of turn" (CR 116.2d): the source's
    /// other static abilities don't apply to that player (or to objects they control)
    /// until end of turn.
    IgnoreSourceEffects,
}

/// How an effect changes the targets of a spell or ability (CR 115.7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetChange {
    /// "Change the target(s)": each target to another legal target, or none (CR 115.7a).
    All,
    /// "Change a target": only one of them (CR 115.7b).
    One,
    /// "Change any targets": any number of them (CR 115.7c).
    Any,
    /// "Choose new targets": any number may be left unchanged (CR 115.7d).
    ChooseNew,
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
    /// "attacking alone" / "blocking alone" (CR 506.5).
    AttackingAlone,
    BlockingAlone,
    /// "attacking a player alone" (CR 506.6).
    AttackingPlayerAlone,
    /// Had to attack in the current combat (CR 506.7).
    HadToAttack,
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
    /// One of these specific objects, fixed when an effect was created (e.g. "a source of
    /// your choice", CR 609.7a). A chosen permanent spell also matches the permanent it
    /// becomes.
    Objects(Vec<crate::types::ObjectId>),
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
    /// Has a Phyrexian mana symbol in its mana cost ("a spell with {H} in its mana cost",
    /// CR 107.4g: {H} means any of the fifteen Phyrexian mana symbols).
    HasPhyrexianMana,
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
    /// "of the chosen color": has the color chosen for the source (CR 607.2d). Matches
    /// nothing while no color is chosen (CR 607.5a).
    ChosenColor,
    /// "of the chosen type": has the creature type, land type, or card type chosen for
    /// the source.
    ChosenType,
    /// "with the chosen name": has the card name chosen for the source.
    ChosenName,
    /// "of the chosen card type": has the card type chosen for the source.
    ChosenCardType,
    /// Has the prepared designation (CR 722.3a).
    Prepared,
    /// A spell or ability on the stack with at least one target that is an object matching
    /// the filter ("a spell that targets ~", "a spell that targets a creature you control").
    Targets(Box<Filter>),
    /// Has a sticker on it ("stickered", CR 123.4), or a sticker of the given kind ("with a
    /// name sticker on it").
    HasSticker(Option<StickerType>),
    /// A spell or ability on the stack described by its targets: "with a single target",
    /// "that targets you", "that targets only [something]" (CR 115.9).
    StackTargets(Box<TargetsFilter>),
    /// A spell that was cast from the given zone ("a spell from exile", "from your graveyard").
    CastFrom(ZoneKind),
    /// A spell for which the named optional additional cost was paid ("a kicked spell":
    /// `"kicker"`).
    CastWithCost(SmolStr),
    /// Was dealt damage this turn by an object in the selection ("a creature dealt damage
    /// by ~ this turn").
    DealtDamageThisTurnBy(Box<Sel>),
    /// Is a basic land type, e.g. "nonbasic land" = Land and Not(Supertype(Basic)).
    /// Of the color chosen by the source's linked ability ("the chosen color",
    /// CR 607.2d). Matches nothing if no such choice was made (CR 607.5a).
    LinkedChosenColor,
    /// Of the creature type chosen by the source's linked ability.
    LinkedChosenCreatureType,
    /// With a mana value of the quality ("odd" or "even") chosen by the source's linked
    /// ability (CR 607.2f).
    ManaValueOfChosenQuality,
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
    /// Number of different color pairs (CR 105.5) among matching objects that are
    /// exactly two colors.
    ColorPairsAmong(Filter),
    /// The source permanent's class level (CR 716.2d: a permanent without a level is
    /// treated as level 1).
    ClassLevel,
    /// The value of X of another object ("put X +1/+1 counters on it" for "a spell with
    /// {X} in its mana cost"): the value used by that object (CR 107.3e).
    XOf(Box<Sel>),
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
    /// Whether all of these trigger conditions have occurred this turn, regardless of
    /// whether any ability triggered on them (CR 603.1b).
    AllTriggerConditionsThisTurn(Vec<TriggerCond>),
    /// The word chosen by the linked ability is this one (anchor words, CR 607.2m).
    ChosenWord(String),
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
    /// "only before/after [a point in the combat phase]" timing windows (CR 506.8).
    CombatTiming(CombatTiming),
    /// This word (e.g. an anchor word, CR 614.12c) was chosen for the source.
    Chose(SmolStr),
    /// This ability's source is tapped/untapped/attacking… via SelMatches(This, …).
    /// Custom conditions implemented in code.
    Custom(SmolStr),
}

/// A point in the combat phase referred to by timing restrictions (CR 506.8).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatPoint {
    /// "combat" / "the combat phase".
    Combat,
    /// "attackers are declared" (the declare attackers step, CR 506.8a).
    AttackersDeclared,
    /// "blockers are declared" (the declare blockers step, CR 506.8b).
    BlockersDeclared,
    /// "the combat damage step".
    CombatDamageStep,
    /// "the end of combat step".
    EndOfCombatStep,
}

/// "Cast/activate only [before/after] [point]", optionally "during combat" (CR 506.8).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatTiming {
    pub point: CombatPoint,
    /// "after" rather than "before".
    pub after: bool,
    /// Also requires "during combat" (CR 506.8c): relative to the current combat phase
    /// rather than the first combat phase of the turn (CR 506.8d).
    pub during_combat: bool,
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
    /// "[doesn't untap] during its controller's next untap step": for each affected
    /// object, until its controller's next untap step has passed (CR 502.3).
    ThroughNextUntapStep,
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
    /// Sets the name (CR 612.8): the object loses its other names.
    SetName(SmolStr),
    /// "Exchange the text boxes of [two objects]" (CR 612.5). As the effect is created,
    /// it becomes a [`Modification::SetText`] for each object with the other's rules text.
    ExchangeText,
    /// Replaces the object's rules text (CR 612.5).
    SetText {
        abilities: Vec<Ability>,
        text: SmolStr,
    },
    /// Has the full text of the selected card (CR 612.6): its name, mana cost, color
    /// indicator, type line, rules text, and power and toughness.
    FullTextOf(Box<Sel>),
    /// Adds rules text following the object's own, without changing its own text (a
    /// splice ability, CR 612.10, 702.47c).
    AddText {
        abilities: Vec<Ability>,
        text: SmolStr,
    },
    /// "Has all names of nonlegendary creature cards in addition to its name" (CR 612.7).
    AllCreatureNames,
    /// A name sticker: adds `word` to the object's name after `position` words (CR 123.6,
    /// 612.9).
    NameSticker {
        word: SmolStr,
        position: u32,
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
    /// "is the chosen type in addition to its other types": adds the creature type or
    /// land type chosen for the effect's source (CR 607.2d). Does nothing while no type
    /// is chosen (CR 607.5a).
    AddChosenType,
    /// "Enchanted land is the chosen type": like [`Modification::SetBasicLandType`] with
    /// the basic land type chosen for the effect's source (CR 305.7).
    SetChosenBasicLandType,
    // Layer 5
    SetColors(ColorSet),
    /// "[This] is the chosen color": the color chosen by the linked ability (CR 607.2p).
    /// Does nothing while no color is chosen (CR 607.5a).
    SetLinkedChosenColor,
    AddColors(ColorSet),
    /// "becomes the color of your choice": the color chosen for the effect's source
    /// (fixed when a resolving effect is created).
    SetChosenColor,
    // Layer 6
    AddAbility(Ability),
    AddKeyword(Keyword),
    /// Adds a keyword whose variable is defined by the effect ("~ has bushido X, where X
    /// is ..."): X is reevaluated each time characteristics are computed (CR 702.1b). It
    /// becomes the keyword's N, and the value of {X} in its cost.
    AddKeywordX(Keyword, Value),
    /// Adds each keyword of these kinds, with all its variants and variables, that an
    /// object matching the filter has ("... has flying. The same is true for first strike,
    /// landwalk, protection, ...", CR 702.1c).
    AddKeywordsOf {
        kinds: Vec<KeywordKind>,
        from: Filter,
    },
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
            ChangeText { .. }
            | SetName(_)
            | ExchangeText
            | SetText { .. }
            | FullTextOf(_)
            | AddText { .. }
            | AllCreatureNames
            | NameSticker { .. } => Layer::L3Text,
            AddTypes(_)
            | RemoveTypes(_)
            | AddSupertypes(_)
            | RemoveSupertypes(_)
            | AddSubtypes(_)
            | RemoveSubtypes(_)
            | SetTypes { .. }
            | AllCreatureTypes
            | RemoveAllCreatureTypes
            | SetBasicLandType(_)
            | AddChosenType
            | SetChosenBasicLandType => Layer::L4Type,
            SetColors(_) | AddColors(_) | SetLinkedChosenColor | SetChosenColor => Layer::L5Color,
            AddAbility(_)
            | AddKeyword(_)
            | AddKeywordX(..)
            | AddKeywordsOf { .. }
            | RemoveKeyword(_)
            | RemoveAllAbilities
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
    /// One mana of any type that a permanent matching the filter could produce
    /// (CR 106.7), e.g. "any type that a land you control could produce".
    CouldProduce(Filter),
    /// One mana of any *color* that a permanent matching the filter could produce
    /// (CR 106.7), e.g. "any color that a land an opponent controls could produce".
    CouldProduceColor(Filter),
    /// N mana of the chosen color (stored on the source, e.g. "the chosen color").
    ChosenColor(Value),
    /// One mana of one of the listed types or of the color chosen for the source
    /// ("Add {R} or one mana of the chosen color").
    OneOfOrChosenColor(Vec<ManaType>),
    /// N mana of a fixed type.
    Amount(ManaType, Value),
    /// Mana of any color among the colors of the selected objects (commander identity etc.).
    AnyColorAmong(Filter),
    /// One mana of any type the permanent tapped for mana produced (from the triggering
    /// event, "one mana of any type that land produced").
    AnyTypeProduced,
    /// Mana represented by the symbols of the selected object's mana cost ("add mana equal
    /// to enchanted permanent's mana cost", CR 106.8–106.11).
    ManaCostOf(Sel),
    /// Doubles the amount of each type of unspent mana the player has (CR 701.10f;
    /// Doubling Cube). The new mana has no restrictions (CR 106.6 example).
    DoubleUnspent,
    /// One mana of any type the triggering mana ability produced ("add one mana of any
    /// type that land produced", CR 106.12a).
    TypeProduced,
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
    /// A player matching would draw `min` or more cards (an effect that refers to the
    /// number of cards drawn, CR 121.2a, 616.1g).
    DrawCards { who: PlayerFilter, min: u32 },
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
    /// A player matching `by` would put counters (of `kind`) on a permanent ("If you would
    /// put one or more counters on a permanent", CR 122.6a).
    PutCountersBy {
        by: PlayerRel,
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
    /// "As [this permanent] is turned face up, ..." (CR 614.1e): performed with
    /// [`ReplacementAction::AsEnters`] as the permanent turns face up.
    TurnedFaceUp,
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
    /// "As this enters, ..." — the effect is performed while the replacement applies,
    /// before the permanent enters (CR 614.1c, 614.12a). Choices ([`Effect::Choose`]) are
    /// stored on the entering object and carried onto the permanent; the entry-modifying
    /// effects [`Effect::EnterTapped`] and [`Effect::EnterWithCounters`] change how it
    /// enters. Conditional ETB replacements ("enters tapped unless ...") are expressed as
    /// `AsEnters(If { .. })`.
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
    /// Prevent N (or all, when `None`) of the damage, then perform an additional effect
    /// right afterward that can refer to the amount prevented as the event amount
    /// (CR 615.5): "prevent that damage. You gain life equal to the damage prevented this
    /// way."
    PreventAndThen(Option<Value>, Box<Effect>),
    /// Enters transformed, with its back face up (CR 616.1d, 712.14).
    EnterTransformed,
}

/// Rule-modifying effects (CR 613.11): restrictions and requirements.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Restriction {
    CantAttack(Filter),
    CantBlock(Filter),
    /// "can't attack or block".
    CantAttackOrBlock(Filter),
    /// "can't attack you" (`planeswalkers`: "or planeswalkers you control"; `battles`:
    /// battles the player protects), and "can't attack unless defending player ..."
    /// (the player it would attack, CR 508.5).
    CantAttackPlayer {
        attackers: Filter,
        defender: PlayerFilter,
        planeswalkers: bool,
        battles: bool,
    },
    /// "is goaded" as a static ability: goaded by the source's controller for as long as
    /// the effect applies (CR 701.15b).
    Goaded(Filter),
    /// "assigns combat damage equal to its toughness rather than its power" (modifies
    /// CR 510.1a).
    DamageByToughness(Filter),
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
    /// "can't be blocked by more than one creature".
    MaxBlockedBy {
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
    /// "can attack as though it didn't have defender" (overrides CR 702.3b).
    AttackDespiteDefender(Filter),
    /// "can't attack alone" / "can't block alone" (CR 506.5, 508.1c).
    CantAttackAlone(Filter),
    CantBlockAlone(Filter),
    /// "No more than N creatures can attack/block each combat" (CR 508.1c, 509.1b).
    MaxAttackers(u32),
    MaxBlockers(u32),
    /// "All creatures able to block [filter] do so" (a blocking requirement for each
    /// creature able to block it, CR 509.1c).
    MustBeBlockedByAll(Filter),
    /// "[attackers] can't attack [defender] (or planeswalkers they control) unless their
    /// controller pays [cost] for each ..." (CR 508.1d, 508.1h).
    AttackCost {
        attackers: Filter,
        defender: PlayerFilter,
        planeswalkers: bool,
        cost: Cost,
    },
    /// "[blockers] can't block unless their controller pays [cost] for each blocking
    /// creature" (CR 509.1c, 509.1d).
    BlockCost {
        blockers: Filter,
        cost: Cost,
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
    /// "[objects] can't enter the battlefield" (CR 608.3e). Handled exactly like
    /// [`Restriction::CantEnter`] (CR 614.17d).
    CantEnterBattlefield(Filter),
    /// "doesn't untap during its controller's untap step".
    DoesntUntap(Filter),
    /// "[Players] can't untap more than [n] [objects] during their untap steps"
    /// (modifies CR 502.3).
    MaxUntaps {
        who: PlayerFilter,
        what: Filter,
        n: u32,
    },
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
    /// "[objects] can't be regenerated [this turn]": regeneration shields and effects
    /// don't apply when they're destroyed (CR 701.19c).
    CantBeRegenerated(Filter),
    /// "[objects] can't enter the battlefield" (CR 614.17d), checked against the object as
    /// it would exist on the battlefield.
    CantEnter(Filter),
    /// "can't be the target of spells or abilities your opponents control" is CantBeTargeted.
    /// "damage can't be prevented".
    DamageCantBePrevented,
    /// "Damage [sources matching the filter] would deal can't be prevented" (CR 615.12).
    SourceDamageCantBePrevented(Filter),
    /// "can't transform".
    CantTransform(Filter),
    /// "can't search libraries".
    CantSearch(PlayerFilter),
    /// Cast spells only at sorcery speed etc.
    SorcerySpeedOnly(PlayerFilter),
    /// "can't play lands".
    CantPlayLands(PlayerFilter),
    /// "While [a player] is choosing targets as part of casting a spell or activating an
    /// ability, that player must choose at least one [object] if able" (CR 601.2c).
    MustTarget {
        chooser: PlayerFilter,
        what: Filter,
    },
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
    /// Loyalty abilities of sources matching (CR 606.4).
    LoyaltyAbilities(Filter),
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
    /// Costs the given mana symbols less (CR 118.7a–g). `colored_only`: "This effect
    /// reduces only the amount of colored mana you pay."
    ReduceMana { mana: ManaCost, colored_only: bool },
    /// Additional non-mana cost ("As an additional cost to cast spells, pay 2 life").
    AdditionalCost(Cost),
    /// "You may pay X rather than pay this spell's mana cost."
    AlternativeCost(Cost),
    /// "You may cast this spell as though it had flash if you pay [cost] more to cast it"
    /// (CR 601.3c).
    FlashForAdditionalCost(Cost),
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
    /// "If [cause] causes a triggered ability of [sources] to trigger, that ability triggers
    /// an additional time" (CR 603.2d). `cause: None` means any trigger event.
    AdditionalTrigger {
        sources: Filter,
        cause: Option<Box<TriggerCond>>,
    },
    /// "You may spend [types] mana as though it were mana of any color to pay [costs]"
    /// (CR 602.1e). An empty list means mana of any type.
    SpendAsAnyColor {
        applies_to: CostTarget,
        types: Vec<ManaType>,
    },
    /// An action a player may take with this card from their opening hand (CR 103.6):
    /// with no delayed ability, "you may begin the game with it on the battlefield";
    /// otherwise "you may reveal this card from your opening hand. If you do, [delayed
    /// triggered ability]", whose source is this card (CR 603.7g).
    OpeningHand {
        delayed: Option<Box<(TriggerCond, Body)>>,
    },
    /// "If this card is your commander, choose a color before the game begins" (CR 607.2p):
    /// the choice is linked to the characteristic-defining ability in the same paragraph
    /// and persists as the card changes zones.
    PregameChoice {
        kind: ChoiceKind,
        only_if_commander: bool,
    },
    /// "Before you shuffle your deck to start the game, you may reveal this card from your
    /// deck and exile [a card matching `what`] you drafted that isn't in your deck"
    /// (CR 607.2n). Functions in the library before the game begins.
    BeforeShuffleExile {
        what: Filter,
    },
    /// Companion (CR 702.139a): the condition the owner's starting deck must fulfill for
    /// this card to be revealed as their companion before the game (CR 103.2b).
    Companion(crate::start::DeckCondition),
    /// "Any time you could mulligan and this card is in your hand, you may [effect]"
    /// (CR 103.5b). Functions in the hand while mulligans are declared.
    AnyTimeCouldMulligan(Box<Effect>),
    /// A static ability that functions on the stack and creates a delayed triggered ability
    /// as the permanent spell resolves and the permanent enters (CR 608.3g), e.g. dash's
    /// "return it to its owner's hand at the beginning of the next end step". The
    /// condition is checked against the spell ("if it was cast for its dash cost").
    DelayedTriggerAsEnters {
        condition: Option<Condition>,
        trigger: TriggerCond,
        body: Body,
    },
    /// A special action players may take any time they have priority (CR 116.2d, 116.2e):
    /// "You may discard this card any time you could cast an instant", "Any player may pay
    /// {2} for that player to ignore this effect until end of turn".
    SpecialAction(SpecialActionDef),
    /// "You may look at the top card of your library any time."
    LookAtTopCard(PlayerRel),
    /// "Play with the top card of your library revealed."
    RevealTopCard(PlayerRel),
    /// Additional land plays per turn.
    AdditionalLandPlays(PlayerRel, u32),
    /// "Cast this spell only [condition]" — e.g. "only during combat before blockers are
    /// declared" (CR 506.8). Checked from the card itself while it's being cast.
    CastOnlyIf(Condition),
    /// An optional cost to attack with the source, paid "as it attacks" (CR 508.1g), e.g.
    /// "You may exert this creature as it attacks. When you do, [then]."
    OptionalAttackCost {
        cost: Cost,
        then: Option<Box<Body>>,
    },
    /// "Enchant [filter]" is a keyword; this covers "can be attached only to ..." etc.
    /// Mana abilities that tap for more: handled via Replacement(ProduceMana).
    /// "Prevent all combat damage that would be dealt ..." is a Replacement.
    /// Custom behavior implemented in code, by name.
    Custom(SmolStr),
    /// "If you cast a spell this way, it gains [ability]": spells matching `what` that a
    /// player casts from `zone` using a permission from this object gain the
    /// modifications; they last until the end of the game, even after the spell becomes a
    /// permanent and even if this object leaves (CR 611.3d).
    CastGrant {
        zone: ZoneKind,
        what: Filter,
        mods: Vec<Modification>,
    },
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
    /// "Whenever [filter] attacks alone" (CR 506.5).
    AttacksAlone(Filter),
    /// "Whenever [filter] attacks a player alone" (CR 506.6).
    AttacksPlayerAlone(Filter),
    /// "Whenever [attacker] attacks [you / a planeswalker you control / ...]" (CR 508.3a).
    AttacksRecipient {
        attacker: Filter,
        recipient: DamageRecipient,
    },
    /// "Whenever [a player, planeswalker, or battle] is attacked" (CR 508.3b): once per
    /// attacked player/permanent.
    IsAttacked(DamageRecipient),
    /// "Whenever [player] attacks with [N or more] [filter]" (CR 508.3c).
    PlayerAttacksWith {
        who: PlayerRel,
        filter: Filter,
        min: u32,
    },
    /// "Whenever [player] attacks [another player]" (CR 508.3e): once per attacked player.
    PlayerAttacksPlayer {
        attacker: PlayerRel,
        defender: PlayerRel,
    },
    /// "Whenever [blocker] blocks a creature" (CR 509.3b): once per attacker blocked.
    /// Event object = the blocked attacker ("that creature"), other = the blocker.
    BlocksCreature {
        blocker: Filter,
        attacker: Filter,
    },
    /// "Whenever [attacker] becomes blocked by a creature" (CR 509.3d): once per blocker.
    /// Event object = the blocker ("that creature"), other = the attacker.
    BlockedByCreature {
        attacker: Filter,
        blocker: Filter,
    },
    /// "Whenever [attacker] becomes blocked by N or more creatures" (CR 509.3e).
    BlockedByN {
        attacker: Filter,
        n: u32,
    },
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
    /// "Whenever [filter] is dealt excess damage" (CR 120.10).
    DealtExcessDamage {
        filter: Filter,
        noncombat_only: bool,
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
    /// "When the Nth [kind] counter is put on [filter]": one or more counters are put on it
    /// such that it had fewer than N before and N or more after (CR 122.7).
    CounterThreshold {
        filter: Filter,
        kind: CounterKind,
        n: u32,
    },
    BecomesTapped(Filter),
    BecomesUntapped(Filter),
    /// "Whenever [filter] is tapped for mana of a specified type" / "Whenever you tap a
    /// permanent for {C}" (CR 106.12a): only if that type of mana was produced.
    TappedForManaOfType {
        filter: Filter,
        mana: ManaType,
    },
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
    /// "Whenever [filter] phases out" (looks back in time, CR 603.10b).
    PhasesOut(Filter),
    /// "Whenever [filter] becomes unattached from a permanent" (looks back, CR 603.10c).
    /// "That permanent" is the trigger object.
    BecomesUnattached(Filter),
    /// "When you lose control of [filter]" (looks back in time, CR 603.10d).
    LoseControl(Filter),
    /// "When/Whenever [filter spell] is countered" (looks back in time, CR 603.10e).
    SpellCountered(Filter),
    /// "Whenever an ability of [source] resolves" / "Whenever the final chapter ability of
    /// a Saga you control resolves" (CR 608.2p): triggers once the ability has finished
    /// resolving. "That Saga" is the trigger object.
    AbilityResolved {
        source: Filter,
        final_chapter: bool,
    },
    /// "Whenever [cause] causes a triggered ability [of a matching source] to trigger":
    /// triggers on another ability triggering (CR 603.3b). `cause` names the kind of
    /// event and what it's about (e.g. `EntersBattlefield(Permanent)`); "that ability" is
    /// the trigger spell.
    AbilityTriggered {
        cause: Box<TriggerCond>,
        source: Filter,
    },
    /// An ability with several trigger conditions ("When ~ enters or dies", "At the
    /// beginning of your upkeep and whenever you cast a green spell"). It triggers once for
    /// each event that matches any of them (CR 603.2c): for a single event, the first
    /// matching condition is used.
    AnyOf(Vec<TriggerCond>),
    /// A trigger event qualified by a condition that is part of the trigger event itself
    /// ("attacks alone", "while you control …", "your second card each turn"). The
    /// condition is evaluated with the event information when the event occurs; unlike an
    /// intervening "if" clause (CR 603.4) it isn't checked again on resolution.
    Where {
        trigger: Box<TriggerCond>,
        cond: Condition,
    },
    /// "… for the first time each turn": triggers only if no earlier event this turn
    /// matched the inner trigger condition.
    FirstTimeEachTurn(Box<TriggerCond>),
    /// Triggers once for each batch of simultaneous events matching the inner condition
    /// (CR 603.2c), grouped by `per`: "whenever one or more creatures die" (once per
    /// batch), "one or more creatures you control deal combat damage to a player" (once
    /// per damaged player), "whenever ~ is dealt damage" (once however many sources dealt
    /// damage at the same time). The event info carries all matching objects (`objects`)
    /// and the total amount; other fields come from the first matching event.
    Batched {
        trigger: Box<TriggerCond>,
        per: BatchPer,
    },
    /// "Whenever [player] copies a [filter] spell" ("whenever you cast or copy an instant or
    /// sorcery spell"): a copy of a spell was put onto the stack (CR 707.10).
    SpellCopied {
        who: PlayerRel,
        filter: Filter,
    },
    /// A player performed a named keyword action reported as [`crate::events::Event::Custom`]
    /// ("scry", "surveil", "proliferate"): "whenever you scry".
    PlayerAction {
        name: SmolStr,
        who: PlayerRel,
    },
    /// "Whenever [attacker] attacks [defender] [with N or more [filter]]", "whenever
    /// [defender] is attacked" (CR 508.3b, 508.3e): once for each player attacked by
    /// creatures the attacking player controls, when attackers are declared. Needs at least
    /// `min` attackers matching `with` attacking that player. Event player = the attacked
    /// player, objects = the creatures attacking them, amount = their number.
    PlayerAttacked {
        attacker: PlayerRel,
        defender: PlayerFilter,
        with: Filter,
        min: u32,
    },
    /// "Whenever [obj] becomes attached to [other]" (`attached`) / "becomes unattached from
    /// [other]" (CR 701.3). Event object = the Aura/Equipment, other = the permanent.
    AttachChanged {
        attached: bool,
        obj: Filter,
        other: Filter,
    },
    /// "Whenever [filter] phases in" (`phased_in`) / "phases out" (CR 702.26).
    Phases {
        phased_in: bool,
        filter: Filter,
    },
    /// A delayed triggered ability that lasts for the rest of the turn ("until end of
    /// turn, whenever …", "whenever … this turn", CR 603.7b); removed in the cleanup step
    /// (CR 514.2).
    ThisTurn(Box<TriggerCond>),
    /// The inner damage trigger ("deals damage", "is dealt damage"), for noncombat damage
    /// only: "whenever a source you control deals noncombat damage to an opponent".
    Noncombat(Box<TriggerCond>),
    /// "Whenever [filter] is tapped for mana", "whenever [player] taps [filter] for mana"
    /// (CR 106.12a): a mana ability with {T} in its cost resolved and produced mana. `who`
    /// is the player who activated it. Event object = the permanent, player = `who`.
    TappedForMana {
        who: PlayerRel,
        filter: Filter,
    },
    /// Keyword-provided and card-specific triggers implemented in code, by name.
    Custom(SmolStr),
}

/// How a [`TriggerCond::Batched`] trigger groups the events of one batch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchPer {
    /// Once per batch.
    Batch,
    /// Once per player in the events (`EventInfo::player`).
    Player,
    /// Once per object the events are about (`EventInfo::object`), e.g. the creature dealt
    /// damage.
    Object,
    /// Once per other object (`EventInfo::other`), e.g. the source dealing damage.
    Other,
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
    /// "Note [a number]": records information for the abilities linked to this one to refer
    /// to ("the noted number", CR 607.2e). Read with `Value::Chosen`.
    Note {
        value: Value,
    },
    /// Sets the value of X for the rest of the resolution, and for reflexive triggers it
    /// creates ("that many" after paying a cost any number of times, CR 603.12a).
    SetX {
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
    /// Damage whose excess (beyond lethal damage, loyalty, or defense) is dealt to
    /// another permanent or player instead (CR 120.4a): "Excess damage is dealt to that
    /// creature's controller instead."
    DealDamageExcess {
        source: Sel,
        amount: Value,
        to: Sel,
        excess_to: Sel,
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
    /// "Move [n / all] [kind] counters from [from] onto [to]" (CR 122.5). `kind: None`:
    /// counters of each kind; `n: None`: all of them.
    MoveCounters {
        from: Sel,
        to: Sel,
        kind: Option<CounterKind>,
        n: Option<Value>,
    },
    /// "Put [its] counters on [to]" for an object that has left the battlefield: the same
    /// number of each kind of counter it had (or only of `kind`) (CR 122.8, 122.9).
    PutCountersOf {
        from: Sel,
        to: Sel,
        kind: Option<CounterKind>,
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
    /// "Until end of turn, you may pay {1} any time you could cast an instant. If you do,
    /// ..." (CR 116.2c): lets the players take a special action later, while `duration`
    /// lasts. `repeatable`: whether it can be taken more than once.
    OfferSpecialAction {
        def: Box<SpecialActionDef>,
        duration: Duration,
        repeatable: bool,
    },
    /// "[Player] puts a [kind of] sticker on [objects]" (CR 123.3): they choose one of the
    /// stickers they have access to that isn't on an object they own, and pay its ticket
    /// cost unless `free` (CR 123.3c). `max_ticket`: "with ticket cost X or less".
    PutSticker {
        who: PlayerRef,
        what: Sel,
        kind: Option<StickerType>,
        max_ticket: Option<Value>,
        free: bool,
    },
    /// "Mana of any type can be spent to cast [that spell]" (CR 118.14): `who` may spend
    /// mana as though it were colorless mana or mana of any color to cast the card.
    SpendAnyTypeMana {
        who: PlayerRef,
        what: Sel,
        duration: Duration,
    },
    /// "[Player] may change the target(s) of / choose new targets for [spell or ability]"
    /// (CR 115.7). With `to`, the new target must be that object or player ("change the
    /// target of target spell to this creature").
    ChangeTargets {
        what: Sel,
        who: PlayerRef,
        how: TargetChange,
        to: Option<Sel>,
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
    /// Attach as though the permanent it's attached to were a creature ("equip
    /// planeswalker", CR 702.6e).
    AttachAsCreature {
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
    /// "[It] enters tapped": only meaningful inside [`ReplacementAction::AsEnters`], where
    /// it modifies how the permanent enters (CR 614.1c); elsewhere it does nothing.
    EnterTapped,
    /// "[It] enters with N [kind] counters on it": only meaningful inside
    /// [`ReplacementAction::AsEnters`] (CR 614.1c, 122.6); elsewhere it does nothing.
    EnterWithCounters {
        kind: CounterKind,
        n: Value,
    },
    /// "[It] enters prepared": only meaningful inside [`ReplacementAction::AsEnters`]
    /// (CR 722.3a); elsewhere it does nothing.
    EnterPrepared,
    /// "... enter as a copy of X, except [exceptions]": if the permanent enters as a copy,
    /// these modifications are part of its copiable values (CR 707.9b). Only meaningful
    /// inside [`ReplacementAction::AsEnters`].
    EnterCopyExceptions(Vec<Modification>),
    /// "[It] enters with haste", "as ~ enters, it becomes a 3/3 creature": an effect on
    /// the permanent performed as it's put onto the battlefield, with the permanent as
    /// its source (CR 614.1c). Only meaningful inside [`ReplacementAction::AsEnters`],
    /// where it's deferred until right after the permanent enters (before its
    /// zone-change event); elsewhere it does nothing.
    OnEntry(Box<Effect>),
    /// "As ~ enters, it becomes your choice of a 3/3 creature or a 2/2 creature with
    /// flying": an "as enters" ability that sets power and toughness (and maybe other
    /// characteristics) modifies the permanent's copiable values (CR 707.2). Only
    /// meaningful inside [`ReplacementAction::AsEnters`].
    EnterAs(Vec<Modification>),
    /// "It becomes day" / "it becomes night" (CR 731.1).
    SetDayNight {
        day: bool,
    },
    /// "[permanents] become prepared" / "become unprepared" (CR 722.3a–c).
    SetPrepared {
        what: Sel,
        prepared: bool,
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
    /// Two players exchange life totals (CR 119.7, 119.8).
    ExchangeLifeTotals {
        a: PlayerRef,
        b: PlayerRef,
    },
    AddMana {
        who: PlayerRef,
        mana: ManaProduction,
        restriction: Option<ManaRestriction>,
    },
    /// "Add [mana]. When that mana is spent to cast [a spell], [effect]." (CR 106.6): the
    /// inner `AddMana` adds mana carrying a delayed triggered ability that triggers when
    /// that mana is spent (one per mana produced, CR 106.6a).
    AddManaWithSpentTrigger {
        add: Box<Effect>,
        spell_filter: Filter,
        body: Box<Body>,
    },
    /// "This Class's level becomes N" (a class level bar's activated ability, CR 107.16a,
    /// 716.2a).
    SetClassLevel {
        level: u32,
    },
    /// "[Player] activates a mana ability of each [filter] they control" (Drain Power).
    ActivateManaAbilities {
        who: PlayerRef,
        filter: Filter,
    },
    /// "[Player] loses all unspent mana [and you add the mana lost this way]" (CR 106.13):
    /// empties the player's mana pool; the lost mana (with its sources, restrictions, and
    /// riders) is added to `to`'s mana pool, if any.
    LoseUnspentMana {
        who: PlayerRef,
        to: Option<PlayerRef>,
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
    /// A reflexive triggered ability ("When you do, ..."): created during resolution, it
    /// triggers immediately and is put on the stack the next time a player would receive
    /// priority, with its own targets (CR 603.12). Wrap it in a condition to make it
    /// trigger only if the action was taken.
    Reflexive {
        body: Box<Body>,
    },
    /// "At the beginning of the next end step" / "at end of combat" (common delayed triggers).
    AtNext {
        step: TriggerStep,
        effect: Box<Effect>,
    },
    /// "[Player] gets an emblem with [ability]" (CR 114.2): each such player puts an
    /// emblem with the abilities into the command zone; they own and control it.
    CreateEmblem {
        who: PlayerRef,
        abilities: Vec<Ability>,
    },
    /// "Each player chooses from among the permanents they control an artifact, a creature,
    /// an enchantment, and a land, then sacrifices the rest" (CR 101.4): each player, in
    /// APNAP order, chooses one permanent they control among `among` for each of `keep`
    /// in the order given (CR 101.4c; a permanent with several of those types may be
    /// chosen for each of them), then all their other permanents among `among` are
    /// sacrificed at the same time.
    KeepAndSacrificeRest {
        who: PlayerRef,
        among: Filter,
        keep: Vec<Filter>,
    },
    /// "Restart the game[, leaving in exile all ... exiled with ~]" (CR 104.6, 727): the
    /// game ends and a new one begins with the resolving ability's controller as the
    /// starting player. Cards selected by `keep` stay in exile, out of their owners' decks
    /// (CR 727.5); they're "it" for the rest of the effect, which happens just before the
    /// new game's first untap step (CR 727.4).
    RestartGame {
        keep: Option<Sel>,
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
    /// "Change the text of [objects] by replacing all instances of one [kind of word] with
    /// another" (CR 612): the words are chosen as it resolves; a layer 3 effect results.
    ChangeText {
        what: Sel,
        words: crate::text_change::TextWords,
        /// Words the new word can't be ("The new creature type can't be Wall").
        exclude: Vec<SmolStr>,
        duration: Duration,
    },
    /// "Exile [objects] until [event]" (CR 610.3): when the event happens, the objects
    /// return to the zones they were in (to the battlefield under their owners' control).
    ExileUntil {
        what: Sel,
        until: UntilEvent,
    },
    /// "[Permanents] phase out until [event]" (CR 610.4).
    PhaseOutUntil {
        what: Sel,
        until: UntilEvent,
    },
    /// Performs `effect`, with `replacement` applying to the events it causes as a
    /// self-replacement effect (CR 614.15): "Counter target spell. If that spell is
    /// countered this way, put it on top of its owner's library instead of into that
    /// player's graveyard."
    SelfReplace {
        replacement: ReplacementDef,
        effect: Box<Effect>,
    },
    /// "A source of your choice" (CR 609.7a): the player chooses a source of damage —
    /// a permanent, a spell, or a face-up object in the command zone — matching the
    /// filter. The choice is stored in `var`.
    ChooseSource {
        who: PlayerRef,
        filter: Filter,
        var: Var,
    },
    /// "The next [filter] spell you cast this turn [has ...]" (CR 611.2f): a continuous
    /// effect that begins to apply to the next matching spell its controller puts on the
    /// stack. `expires` is how long the effect waits for that spell.
    NextSpell {
        filter: Filter,
        mods: Vec<Modification>,
        expires: Duration,
    },
    /// Card-specific behavior implemented in code, by name.
    Custom(SmolStr),
}

/// The event that ends an "until" effect (CR 610.3, 610.4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UntilEvent {
    /// "until [this object] leaves the battlefield".
    SourceLeavesBattlefield,
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
    /// "choose a color other than [color]".
    ColorOtherThan(Color),
    /// "choose A, B, or C": one of the listed words — creature types, land types, card
    /// types, colors, or anchor words (CR 614.12c, 607.2f). Stored as the chosen text
    /// (and as the chosen type/color when the word is one).
    OneOf(Vec<String>),
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
    /// One of several words with no rules meaning, e.g. anchor words ("choose Khans or
    /// Dragons", CR 607.2f, 607.2m, 614.12b).
    Word(Vec<String>),
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
