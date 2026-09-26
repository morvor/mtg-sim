//! Game objects (CR 109) and their characteristics (CR 109.3).

use crate::ability::{Ability, AbilityKind, ZoneKind};
use crate::card::CardDef;
use crate::keywords::{Keyword, KeywordKind};
use crate::mana::{ManaCost, ManaType};
use crate::types::*;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use smol_str::SmolStr;
use std::collections::BTreeMap;
use std::sync::Arc;

/// An object's characteristics (CR 109.3): name, mana cost, color, color indicator,
/// card type, subtype, supertype, rules text (abilities), power, toughness, loyalty,
/// defense, hand modifier, and life modifier.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Characteristics {
    pub name: SmolStr,
    pub mana_cost: Option<ManaCost>,
    pub color_indicator: Option<ColorSet>,
    pub colors: ColorSet,
    pub supertypes: SupertypeSet,
    pub card_types: CardTypeSet,
    pub subtypes: SmallVec<[Subtype; 3]>,
    pub abilities: Vec<Ability>,
    pub power: Option<i32>,
    pub toughness: Option<i32>,
    pub loyalty: Option<i32>,
    pub defense: Option<i32>,
    pub hand_modifier: Option<i32>,
    pub life_modifier: Option<i32>,
    /// Oracle text, for display and text-changing effects.
    pub rules_text: Arc<str>,
    /// Also has the name of each nonlegendary creature card (Spy Kit, CR 612.7).
    #[serde(default)]
    pub all_creature_names: bool,
    /// Names interchangeable with its name (CR 201.3): for all rules and effects that
    /// refer to names, the object has these names too (CR 201.3a).
    #[serde(default)]
    pub interchangeable_names: SmallVec<[SmolStr; 1]>,
    /// Is every creature type (changeling, CR 702.73a; "is every creature type",
    /// CR 205.3m), in addition to its listed subtypes. Only a creature or kindred object
    /// can have it (CR 205.3d).
    #[serde(default)]
    pub all_creature_types: bool,
}

impl Characteristics {
    pub fn is(&self, t: CardType) -> bool {
        self.card_types.contains(t)
    }
    pub fn is_creature(&self) -> bool {
        self.is(CardType::Creature)
    }
    pub fn is_land(&self) -> bool {
        self.is(CardType::Land)
    }
    pub fn is_permanent_type(&self) -> bool {
        self.card_types.has_permanent_type()
    }
    pub fn has_supertype(&self, s: Supertype) -> bool {
        self.supertypes.contains(s)
    }
    pub fn is_legendary(&self) -> bool {
        self.has_supertype(Supertype::Legendary)
    }
    pub fn has_subtype(&self, s: &str) -> bool {
        self.subtypes.iter().any(|x| x.as_str() == s)
            || (self.all_creature_types
                && (self.is(CardType::Creature) || self.is(CardType::Kindred))
                && is_creature_type(s))
    }
    pub fn mana_value(&self) -> u32 {
        self.mana_cost.as_ref().map_or(0, |m| m.mana_value())
    }
    pub fn keywords(&self) -> impl Iterator<Item = &Keyword> {
        self.abilities.iter().filter_map(|a| a.keyword())
    }
    pub fn has_keyword(&self, k: KeywordKind) -> bool {
        self.keywords().any(|kw| kw.kind == k)
    }
    pub fn keyword(&self, k: KeywordKind) -> Option<&Keyword> {
        self.keywords().find(|kw| kw.kind == k)
    }
    pub fn keyword_count(&self, k: KeywordKind) -> usize {
        self.keywords().filter(|kw| kw.kind == k).count()
    }
    /// Each of the object's own names (CR 201.2): a split card's combined name "A // B"
    /// is two names (CR 709.4a), and names interchangeable with its name are its names
    /// too (CR 201.3a). (The names of "all nonlegendary creature cards", CR 612.7, aren't
    /// listed; see [`Characteristics::has_name`].)
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.name
            .split(" // ")
            .map(str::trim)
            .filter(|n| !n.is_empty())
            .chain(self.interchangeable_names.iter().map(SmolStr::as_str))
    }
    /// True if the object has at least one name (CR 201.2a: a face-down permanent, for
    /// example, has none).
    pub fn has_a_name(&self) -> bool {
        self.names().next().is_some() || self.all_creature_names
    }
    /// True if the object has the name `n` (CR 201.2): one of its own names, or, with "all
    /// names of nonlegendary creature cards" (CR 612.7), any such card's name.
    pub fn has_name(&self, n: &str) -> bool {
        let n = n.trim();
        !n.is_empty()
            && (self.names().any(|x| x.eq_ignore_ascii_case(n))
                || (self.all_creature_names
                    && crate::text_change::is_nonlegendary_creature_name(n)))
    }
    /// True if the two objects have the same name: at least one name in common, even if
    /// either has additional names. An object with no name doesn't have the same name as
    /// any other object, including another object with no name (CR 201.2a).
    pub fn shares_name_with(&self, other: &Characteristics) -> bool {
        if !self.has_a_name() || !other.has_a_name() {
            return false;
        }
        (self.all_creature_names && other.all_creature_names)
            || self.names().any(|n| other.has_name(n))
            || other.names().any(|n| self.has_name(n))
    }
    /// True if the object has no abilities other than those the compiler couldn't parse.
    pub fn has_no_abilities(&self) -> bool {
        self.abilities
            .iter()
            .all(|a| matches!(a.kind, AbilityKind::Unsupported(_)))
    }
}

/// What sort of object this is (CR 109.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjKind {
    /// A physical card.
    Card,
    /// A token (CR 111).
    Token,
    /// A copy of a card (e.g. cast from a copy effect, CR 707.12).
    CardCopy,
    /// A copy of a spell on the stack (CR 707.10).
    SpellCopy,
    /// An activated or triggered ability on the stack (CR 113.1b).
    StackAbility,
    /// An emblem in the command zone (CR 114).
    Emblem,
}

/// A zone (CR 400).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Zone {
    Library(PlayerId),
    Hand(PlayerId),
    Battlefield,
    Graveyard(PlayerId),
    Stack,
    Exile,
    Command,
    Ante,
    /// Outside the game (sideboard etc., CR 400.11).
    Outside(PlayerId),
    /// No longer exists (tokens/copies that ceased to exist, abilities that resolved).
    Nowhere,
}

impl Zone {
    pub fn kind(self) -> Option<ZoneKind> {
        Some(match self {
            Zone::Library(_) => ZoneKind::Library,
            Zone::Hand(_) => ZoneKind::Hand,
            Zone::Battlefield => ZoneKind::Battlefield,
            Zone::Graveyard(_) => ZoneKind::Graveyard,
            Zone::Stack => ZoneKind::Stack,
            Zone::Exile => ZoneKind::Exile,
            Zone::Command => ZoneKind::Command,
            Zone::Ante => ZoneKind::Ante,
            Zone::Outside(_) => ZoneKind::Outside,
            Zone::Nowhere => return None,
        })
    }
    /// Public zones (CR 400.2).
    pub fn is_public(self) -> bool {
        !matches!(self, Zone::Library(_) | Zone::Hand(_) | Zone::Outside(_))
    }
    pub fn of_kind(kind: ZoneKind, owner: PlayerId) -> Zone {
        match kind {
            ZoneKind::Library => Zone::Library(owner),
            ZoneKind::Hand => Zone::Hand(owner),
            ZoneKind::Battlefield => Zone::Battlefield,
            ZoneKind::Graveyard => Zone::Graveyard(owner),
            ZoneKind::Stack => Zone::Stack,
            ZoneKind::Exile => Zone::Exile,
            ZoneKind::Command => Zone::Command,
            ZoneKind::Ante => Zone::Ante,
            ZoneKind::Outside => Zone::Outside(owner),
        }
    }
}

/// Which face or half of a multi-part card is being used.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FaceState {
    /// Normal: front face, or combined characteristics for split cards off the stack.
    #[default]
    Front,
    /// Back face of a double-faced card (transformed/converted, or cast as its back face).
    Back,
    /// Casting/using one half of a split card, or the Adventure/Omen part (index into faces).
    Half(u8),
    /// Fused split card on the stack (CR 702.102).
    Fused,
    /// Flip card flipped (CR 710).
    Flipped,
    /// A melded permanent (CR 712.4).
    Melded,
}

/// How a spell is being cast (alternative costs and casting permissions).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CastMethod {
    #[default]
    Normal,
    /// Without paying its mana cost (CR 118.9).
    Free,
    /// A named keyword alternative cost / permission (Flashback, Evoke, Dash, ...).
    Keyword(KeywordKind),
    /// Face down (morph/disguise, CR 708).
    FaceDown(KeywordKind),
    /// An alternative cost from a static ability or effect.
    Alternative(u64),
    /// Cast as an Adventure / Omen / other half (index into faces).
    Half(u8),
}

/// Record of how a spell was cast and what was paid, kept on the permanent it becomes
/// (for "if it was kicked", "if X was 5 or more", etc.).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CastInfo {
    pub method: CastMethod,
    pub from: Option<ZoneKind>,
    pub x: Option<i32>,
    pub times_kicked: u32,
    /// Names of optional additional/alternative costs that were paid ("kicker", "bargain", ...).
    pub paid: Vec<SmolStr>,
    pub mana_spent: Vec<ManaType>,
    pub mana_spent_snow: u32,
    /// Objects sacrificed/exiled/discarded to pay costs (last known info ids).
    pub cost_objects: Vec<ObjectId>,
    pub was_cast: bool,
    /// Turn number on which it was cast.
    pub turn: u32,
    /// Creatures tapped to pay for it with convoke: they "convoked" it (CR 702.51c).
    #[serde(default)]
    pub convoked: Vec<ObjectId>,
    /// It was cast "any time a sorcery couldn't have been cast": without its controller
    /// having priority, outside their main phase, or while another object was on the stack
    /// (CR 307.5a).
    #[serde(default)]
    pub instant_timing: bool,
    /// Cards exiled from a graveyard to pay for it with delve (CR 702.66a), as they are in
    /// exile: the cards "exiled with it".
    #[serde(default)]
    pub delved: Vec<ObjectId>,
}

/// Data from the event that caused a triggered ability to trigger, used by "that
/// creature", "that player", "that much damage", etc.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EventInfo {
    /// The object the event is about. For zone changes this is the object in its new
    /// zone ("return it to its owner's hand").
    pub object: Option<ObjectId>,
    /// Last known information of the object before the event (for zone changes).
    pub lki: Option<ObjectId>,
    pub other: Option<ObjectId>,
    pub player: Option<PlayerId>,
    pub amount: i32,
    pub spell: Option<ObjectId>,
    /// Objects involved (e.g. all attackers).
    pub objects: Vec<ObjectId>,
    /// The mana produced, for "whenever [a permanent] is tapped for mana" ("add one mana of
    /// any type that land produced").
    pub mana: Vec<ManaType>,
}

/// The kind of object on the stack.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StackKind {
    Spell,
    Activated { source: ObjectId, ability: Ability },
    Triggered { source: ObjectId, ability: Ability },
}

/// Choices made for a spell or ability as it was put on the stack.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ChosenMode {
    /// `None` for non-modal bodies.
    pub mode: Option<usize>,
    /// Targets per target slot.
    pub targets: Vec<Vec<Entity>>,
    /// Division of damage/counters among targets (CR 601.2d), per slot.
    pub divided: Vec<Vec<u32>>,
}

/// Stack-specific data for spells and abilities on the stack.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StackInfo {
    pub kind: StackKind,
    pub chosen: Vec<ChosenMode>,
    pub x: Option<i32>,
    pub cast: CastInfo,
    pub event: Option<EventInfo>,
    /// Characteristics of the source as last known (for abilities, CR 113.7a).
    pub source_lki: Option<Box<Characteristics>>,
    /// Numeric/object variables fixed when put on the stack (e.g. "choose a number").
    pub chosen_values: BTreeMap<u16, i64>,
}

/// Values chosen for a permanent ("as this enters, choose a color").
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Choices {
    pub color: Option<Color>,
    pub creature_type: Option<Subtype>,
    pub card_name: Option<SmolStr>,
    pub number: Option<i32>,
    pub player: Option<PlayerId>,
    pub basic_land_type: Option<Subtype>,
    pub card_type: Option<CardType>,
    /// Modal permanent choices (anchor words, CR 614.12c), etc.
    pub mode: Option<usize>,
    pub text: Option<SmolStr>,
}

/// A game object. Objects in zones are referenced by id; when an object changes zones
/// a new object is created and this one is kept as last known information.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameObject {
    pub id: ObjectId,
    pub kind: ObjKind,
    pub owner: PlayerId,
    /// Controller before layer-2 control-changing effects (the player who put it onto
    /// the battlefield, or the caster).
    pub base_controller: PlayerId,
    /// Current controller (computed).
    pub controller: PlayerId,
    pub zone: Zone,
    /// The card this object represents (None for tokens without a card and abilities).
    #[serde(skip)]
    pub card: Option<Arc<CardDef>>,
    pub face: FaceState,
    /// Printed/defined characteristics for the current face, before any effects.
    pub base: Characteristics,
    /// Copiable values after layer 1 (CR 707.2).
    pub copiable: Characteristics,
    /// Current characteristics after all layers.
    pub chars: Characteristics,
    // Status (CR 110.5)
    pub tapped: bool,
    pub flipped: bool,
    pub face_down: bool,
    pub phased_out: bool,
    /// Phased out indirectly (attached to something that phased out, CR 702.26g).
    pub phased_out_indirectly: bool,
    /// The player under whose control it phased out: it phases in during that player's
    /// untap step (CR 702.26a, 702.26n).
    #[serde(default)]
    pub phased_out_under: Option<PlayerId>,
    pub counters: BTreeMap<CounterKind, u32>,
    /// Timestamp of each kind of counter (CR 613.7c): all counters of a kind share the
    /// timestamp of the most recently placed one.
    #[serde(default)]
    pub counter_timestamps: BTreeMap<CounterKind, Timestamp>,
    /// Damage marked (CR 120.6).
    pub damage: u32,
    /// Dealt damage by a deathtouch source since the last SBA check (CR 704.5h).
    pub deathtouch_damage: bool,
    pub attached_to: Option<Entity>,
    pub timestamp: Timestamp,
    /// Hasn't been continuously controlled since its controller's most recent turn began
    /// (CR 302.6 "summoning sickness").
    pub summoning_sick: bool,
    /// Timestamp of the moment it came under its current controller's control (it was
    /// created, or control of it changed), e.g. for echo (CR 702.30a).
    #[serde(default)]
    pub control_since: Timestamp,
    pub stack: Option<Box<StackInfo>>,
    /// How this permanent was cast, if it was.
    pub cast: Option<Box<CastInfo>>,
    pub choices: Choices,
    /// Choices made by this object's abilities, by link id (CR 607.2d, 607.5a): an ability
    /// refers only to choices made by the abilities it's linked to.
    pub linked_choices: BTreeMap<u16, Choices>,
    /// Objects linked to this one by CR 607 (e.g. cards exiled with it), by link id.
    pub linked: BTreeMap<u16, Vec<ObjectId>>,
    /// The object (and link) whose ability created this token or put this permanent onto
    /// the battlefield (CR 607.1d).
    pub created_by: Option<(ObjectId, u16)>,
    /// Previous incarnation of this card (before its last zone change).
    pub prev: Option<ObjectId>,
    /// Next incarnation (after it changed zones).
    pub next: Option<ObjectId>,
    /// Turn number this object entered its current zone.
    pub entered_turn: u32,
    /// Times each activated ability (by uid) has been activated this turn.
    pub activations_this_turn: BTreeMap<u64, u32>,
    /// Times each triggered ability (by uid) has triggered/resolved this turn.
    pub triggers_this_turn: BTreeMap<u64, u32>,
    /// Timestamp when this permanent gained the world supertype (CR 704.5k).
    pub world_since: Option<Timestamp>,
    /// Sticker/merge/other extra state for rarely used rules.
    pub merged_with: Vec<ObjectId>,
    /// For objects representing an emblem, plane, etc., this is where extra flags live.
    pub monstrous: bool,
    pub renowned: bool,
    pub suspected: bool,
    pub saddled: bool,
    pub solved: bool,
    pub exerted: bool,
    /// Class level (CR 716).
    pub class_level: u32,
    /// Goaded by these players until their next turns (CR 701.15).
    pub goaded_by: Vec<PlayerId>,
    /// Was a commander (card identity carried across zones, CR 903.3).
    pub is_commander: bool,
    /// Sector designation (space sculptor).
    pub sector: Option<SmolStr>,
    /// Paired with (soulbond).
    pub paired_with: Option<ObjectId>,
    /// The "prepared" designation (CR 722.3a): the copy of its prepare spell in exile
    /// that its controller may cast (CR 722.3c).
    pub prepared: Option<ObjectId>,
    /// Base power and toughness (CR 208.4b): its power and toughness after
    /// characteristic-defining abilities and effects that set power and toughness
    /// (layers 7a-7b), ignoring effects and counters that modify them without setting them.
    #[serde(default)]
    pub base_pt: (Option<i32>, Option<i32>),
}

/// The value of X an object uses (CR 107.3e): the value announced for a spell or ability
/// on the stack. Off the stack, X is 0 (CR 107.3g, 107.3m).
pub fn x_value_of(o: &GameObject) -> i32 {
    if o.zone == Zone::Stack {
        o.stack.as_ref().and_then(|s| s.x).unwrap_or(0).max(0)
    } else {
        0
    }
}

/// How the spell that became a permanent was cast, for that permanent's own
/// enters-the-battlefield triggered ability: such an ability that refers to X uses the
/// value of X of the spell (CR 107.3m), and conditions like "if it was kicked" see the
/// spell's costs.
pub fn etb_trigger_cast_info(
    g: &crate::game::Game,
    t: &crate::game::PendingTrigger,
) -> Option<CastInfo> {
    use crate::ability::TriggerCond;
    let AbilityKind::Triggered(tr) = &t.ability.kind else {
        return None;
    };
    // CR 702.37f: "When this is turned face up" abilities use the X of its morph cost.
    if !matches!(
        tr.trigger,
        TriggerCond::EntersBattlefield(_) | TriggerCond::TurnedFaceUp(_)
    ) || t.event.object != Some(t.source)
    {
        return None;
    }
    g.try_obj(t.source)?.cast.as_deref().cloned()
}

impl GameObject {
    pub fn new(
        id: ObjectId,
        kind: ObjKind,
        owner: PlayerId,
        zone: Zone,
        base: Characteristics,
    ) -> GameObject {
        GameObject {
            id,
            kind,
            owner,
            base_controller: owner,
            controller: owner,
            zone,
            card: None,
            face: FaceState::Front,
            copiable: base.clone(),
            chars: base.clone(),
            base,
            tapped: false,
            flipped: false,
            face_down: false,
            phased_out: false,
            phased_out_indirectly: false,
            phased_out_under: None,
            counters: BTreeMap::new(),
            counter_timestamps: BTreeMap::new(),
            damage: 0,
            deathtouch_damage: false,
            attached_to: None,
            timestamp: 0,
            summoning_sick: true,
            control_since: 0,
            stack: None,
            cast: None,
            choices: Choices::default(),
            linked_choices: BTreeMap::new(),
            linked: BTreeMap::new(),
            created_by: None,
            prev: None,
            next: None,
            entered_turn: 0,
            activations_this_turn: BTreeMap::new(),
            triggers_this_turn: BTreeMap::new(),
            world_since: None,
            merged_with: vec![],
            monstrous: false,
            renowned: false,
            suspected: false,
            saddled: false,
            solved: false,
            exerted: false,
            class_level: 0,
            goaded_by: vec![],
            is_commander: false,
            sector: None,
            paired_with: None,
            prepared: None,
            base_pt: (None, None),
        }
    }

    pub fn name(&self) -> &str {
        &self.chars.name
    }
    pub fn is_token(&self) -> bool {
        self.kind == ObjKind::Token
    }
    pub fn is_card(&self) -> bool {
        self.kind == ObjKind::Card
    }
    pub fn counter(&self, k: &str) -> u32 {
        self.counters.get(k).copied().unwrap_or(0)
    }
    pub fn is(&self, t: CardType) -> bool {
        self.chars.is(t)
    }
    pub fn is_creature(&self) -> bool {
        self.chars.is_creature()
    }
    pub fn has_keyword(&self, k: KeywordKind) -> bool {
        self.chars.has_keyword(k)
    }
    pub fn power(&self) -> i32 {
        self.chars.power.unwrap_or(0)
    }
    pub fn toughness(&self) -> i32 {
        self.chars.toughness.unwrap_or(0)
    }
    /// Loyalty: a planeswalker permanent's loyalty is the number of loyalty counters on
    /// it (CR 306.5b); anywhere else, it's the loyalty number printed on the card
    /// (CR 209.1).
    pub fn loyalty(&self) -> i32 {
        if self.zone == Zone::Battlefield {
            self.counter(crate::types::counters::LOYALTY) as i32
        } else {
            self.chars.loyalty.unwrap_or(0)
        }
    }
    /// Defense: a battle permanent's defense is the number of defense counters on it
    /// (CR 310.4c); anywhere else, it's the defense number printed on the card (CR 210.1).
    pub fn defense(&self) -> i32 {
        if self.zone == Zone::Battlefield {
            self.counter(crate::types::counters::DEFENSE) as i32
        } else {
            self.chars.defense.unwrap_or(0)
        }
    }
    pub fn on_battlefield(&self) -> bool {
        self.zone == Zone::Battlefield
    }
    pub fn is_spell(&self) -> bool {
        self.zone == Zone::Stack
            && matches!(
                self.stack.as_deref().map(|s| &s.kind),
                Some(StackKind::Spell)
            )
    }
    pub fn is_stack_ability(&self) -> bool {
        self.kind == ObjKind::StackAbility
    }
}
