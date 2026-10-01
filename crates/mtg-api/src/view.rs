//! Observations: what a player can see of the game, as plain serializable structs built
//! from a [`Game`] (the engine's own types aren't serialized directly).
//!
//! Hidden information follows the rules: the order and contents of libraries
//! (CR 401.2), other players' hands (CR 402.3), and the identity of face-down objects
//! (CR 708.5, 406.3) appear only in the view of a player allowed to see them. The
//! omniscient view (`viewer: None`) shows everything.

use crate::visibility::{can_see, Viewer};
use mtg_engine::ability::AbilityKind;
use mtg_engine::combat::CombatState;
use mtg_engine::object::{Characteristics, GameObject, StackKind, Zone};
use mtg_engine::turn::Step;
use mtg_engine::{Entity, Game, GameResult, ObjectId, PlayerId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

fn is_false(b: &bool) -> bool {
    !*b
}
fn is_zero(n: &u32) -> bool {
    *n == 0
}

/// A player or an object: `{"player": 1}` or `{"object": 42}`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityRef {
    Player(u8),
    Object(u32),
}

impl From<Entity> for EntityRef {
    fn from(e: Entity) -> Self {
        match e {
            Entity::Player(p) => EntityRef::Player(p.0),
            Entity::Object(o) => EntityRef::Object(o.0),
        }
    }
}

impl From<EntityRef> for Entity {
    fn from(e: EntityRef) -> Self {
        match e {
            EntityRef::Player(p) => Entity::Player(PlayerId(p)),
            EntityRef::Object(o) => Entity::Object(ObjectId(o)),
        }
    }
}

/// The name of a zone, as used in the protocol.
pub fn zone_name(z: Zone) -> &'static str {
    match z {
        Zone::Library(_) => "library",
        Zone::Hand(_) => "hand",
        Zone::Battlefield => "battlefield",
        Zone::Graveyard(_) => "graveyard",
        Zone::Stack => "stack",
        Zone::Exile => "exile",
        Zone::Command => "command",
        Zone::Ante => "ante",
        Zone::Outside(_) => "outside",
        Zone::Nowhere => "nowhere",
    }
}

/// `snake_case` name of a step.
pub fn step_name(s: Step) -> &'static str {
    match s {
        Step::Untap => "untap",
        Step::Upkeep => "upkeep",
        Step::Draw => "draw",
        Step::PrecombatMain => "precombat_main",
        Step::BeginningOfCombat => "beginning_of_combat",
        Step::DeclareAttackers => "declare_attackers",
        Step::DeclareBlockers => "declare_blockers",
        Step::FirstStrikeDamage => "first_strike_damage",
        Step::CombatDamage => "combat_damage",
        Step::EndOfCombat => "end_of_combat",
        Step::PostcombatMain => "postcombat_main",
        Step::End => "end",
        Step::Cleanup => "cleanup",
    }
}

/// The phase a step belongs to (CR 500.1).
pub fn phase_name(s: Step) -> &'static str {
    match s {
        Step::Untap | Step::Upkeep | Step::Draw => "beginning",
        Step::PrecombatMain => "precombat_main",
        Step::BeginningOfCombat
        | Step::DeclareAttackers
        | Step::DeclareBlockers
        | Step::FirstStrikeDamage
        | Step::CombatDamage
        | Step::EndOfCombat => "combat",
        Step::PostcombatMain => "postcombat_main",
        Step::End | Step::Cleanup => "ending",
    }
}

/// An object's combat status (CR 506.4).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CombatView {
    /// The player, planeswalker or battle it's attacking.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attacking: Option<EntityRef>,
    /// For an attacking creature: whether it's blocked (CR 509.1h).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocked: Option<bool>,
    /// For an attacking creature: the creatures blocking it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blockers: Vec<u32>,
    /// For a blocking creature: the creatures it's blocking.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocking: Vec<u32>,
}

/// A card's or permanent's characteristics.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CardView {
    /// None for an object with no name (a face-down permanent, CR 708.2a).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mana_cost: Option<String>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub mana_value: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub colors: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supertypes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub types: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subtypes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub power: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub toughness: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loyalty: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defense: Option<i32>,
    /// Keyword abilities ("Flying", "Ward {2}", ...).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
    /// The rules text (Oracle text of the current face).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub text: String,
    /// Abilities it has that its rules text doesn't show (granted by effects).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub granted: Vec<String>,
}

impl CardView {
    pub fn from_chars(c: &Characteristics, mana_value: u32) -> CardView {
        let text = c.rules_text.to_string();
        let mut keywords = Vec::new();
        let mut granted = Vec::new();
        for a in &c.abilities {
            if let AbilityKind::Keyword(k) = &a.kind {
                let t = if a.text.trim().is_empty() {
                    k.kind.name().to_string()
                } else {
                    a.text.trim().to_string()
                };
                if !keywords.contains(&t) {
                    keywords.push(t);
                }
            }
            let t = a.text.trim();
            if !t.is_empty()
                && !text.contains(t)
                && !matches!(a.kind, AbilityKind::Unsupported(_))
                && !granted.iter().any(|g: &String| g == t)
            {
                granted.push(t.to_string());
            }
        }
        CardView {
            name: (!c.name.is_empty()).then(|| c.name.to_string()),
            mana_cost: c.mana_cost.as_ref().map(|m| m.to_string()),
            mana_value,
            colors: c.colors.iter().map(|x| x.word().to_string()).collect(),
            supertypes: c
                .supertypes
                .iter()
                .map(|s| format!("{s:?}").to_lowercase())
                .collect(),
            types: c.card_types.iter().map(|t| t.word().to_string()).collect(),
            subtypes: c.subtypes.iter().map(|s| s.to_string()).collect(),
            power: c.power,
            toughness: c.toughness,
            loyalty: c.loyalty,
            defense: c.defense,
            keywords,
            text,
            granted,
        }
    }
}

/// A game object as the viewer sees it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ObjectView {
    pub id: u32,
    pub zone: String,
    pub owner: u8,
    pub controller: u8,
    /// Its characteristics as the viewer sees them: for a face-down object, those of a
    /// face-down object (CR 708.2).
    #[serde(flatten)]
    pub card: CardView,
    #[serde(default, skip_serializing_if = "is_false")]
    pub token: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub face_down: bool,
    /// For a face-down object the viewer may look at (CR 708.5): what it really is.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub face_down_card: Option<Box<CardView>>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub counters: BTreeMap<String, u32>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub tapped: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub flipped: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub phased_out: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub transformed: bool,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub damage: u32,
    /// It came under its controller's control since their most recent turn began (it
    /// can't attack or use {T} abilities unless it has haste, CR 302.6).
    #[serde(default, skip_serializing_if = "is_false")]
    pub summoning_sick: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attached_to: Option<EntityRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub combat: Option<CombatView>,
}

/// An object whose identity the viewer can't see: only that it exists.
fn hidden_object(o: &GameObject) -> ObjectView {
    ObjectView {
        id: o.id.0,
        zone: zone_name(o.zone).into(),
        owner: o.owner.0,
        controller: o.controller.0,
        face_down: o.face_down,
        ..Default::default()
    }
}

fn combat_view(c: Option<&CombatState>, id: ObjectId) -> Option<CombatView> {
    let c = c?;
    let mut v = CombatView::default();
    if let Some(a) = c.attacker(id) {
        v.attacking = a.target.map(EntityRef::from);
        v.blocked = Some(a.blocked);
        v.blockers = a.blockers.iter().map(|b| b.0).collect();
    }
    if let Some(b) = c.blockers.iter().find(|b| b.id == id) {
        v.blocking = b.blocking.iter().map(|a| a.0).collect();
    }
    (v != CombatView::default()).then_some(v)
}

/// The object `id` as `viewer` sees it.
pub fn object_view(g: &Game, viewer: Viewer, id: ObjectId) -> ObjectView {
    let o = g.obj(id);
    let visible = can_see(g, viewer, id);
    let in_hidden_zone = matches!(o.zone, Zone::Library(_) | Zone::Hand(_) | Zone::Outside(_));
    if !visible && (in_hidden_zone || !matches!(o.zone, Zone::Battlefield | Zone::Stack)) {
        // A card in a hidden zone, or a face-down card elsewhere (exile, command zone):
        // nothing but its existence (CR 406.3a).
        return hidden_object(o);
    }
    let chars = if o.face_down && !visible {
        mtg_engine::facedown::face_down_characteristics(g, id)
    } else {
        o.chars.clone()
    };
    let mv = if o.face_down { 0 } else { g.mana_value_of(id) };
    let face_down_card = (o.face_down && visible && viewer.is_some()
        || o.face_down && viewer.is_none())
    .then(|| {
        let real = mtg_engine::facedown::revealed_characteristics(g, id);
        Box::new(CardView::from_chars(&real, real.mana_value()))
    });
    let on_bf = o.zone == Zone::Battlefield;
    ObjectView {
        id: id.0,
        zone: zone_name(o.zone).into(),
        owner: o.owner.0,
        controller: o.controller.0,
        card: CardView::from_chars(&chars, mv),
        token: o.is_token(),
        face_down: o.face_down,
        face_down_card,
        counters: o
            .counters
            .iter()
            .filter(|(_, n)| **n > 0)
            .map(|(k, n)| (k.to_string(), *n))
            .collect(),
        tapped: on_bf && o.tapped,
        flipped: on_bf && o.flipped,
        phased_out: on_bf && o.phased_out,
        transformed: on_bf && o.face == mtg_engine::object::FaceState::Back,
        damage: if on_bf { o.damage } else { 0 },
        summoning_sick: on_bf && o.summoning_sick && o.chars.is_creature(),
        attached_to: if on_bf {
            o.attached_to.map(EntityRef::from)
        } else {
            None
        },
        attachments: if on_bf {
            g.attachments_of(Entity::Object(id))
                .into_iter()
                .map(|a| a.0)
                .collect()
        } else {
            vec![]
        },
        combat: if on_bf {
            combat_view(g.combat.as_ref(), id)
        } else {
            None
        },
    }
}

/// A spell or ability on the stack.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StackObjectView {
    pub id: u32,
    /// "spell", "activated_ability" or "triggered_ability".
    pub kind: String,
    pub controller: u8,
    /// A readable name: the spell's name, or "ability of <source>".
    pub name: String,
    /// For an ability: the object it came from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<u32>,
    /// The text of the spell or ability.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub text: String,
    /// The modes chosen for it (CR 700.2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<i32>,
    /// Its targets, one list per target slot.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<Vec<EntityRef>>,
    /// Readable names of the targets, in the same order (flattened).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_names: Vec<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub face_down: bool,
    /// For a spell: its characteristics as the viewer sees them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub card: Option<ObjectView>,
}

fn mode_texts(
    chars: &Characteristics,
    ability: Option<&AbilityKind>,
    picks: &[usize],
) -> Vec<String> {
    let modal = match ability {
        Some(AbilityKind::Activated(a)) => a.body.modal.as_ref(),
        Some(AbilityKind::Triggered(t)) => t.body.modal.as_ref(),
        _ => chars.abilities.iter().find_map(|a| match &a.kind {
            AbilityKind::Spell(s) => s.body.modal.as_ref(),
            _ => None,
        }),
    };
    let Some(modal) = modal else {
        return vec![];
    };
    picks
        .iter()
        .filter_map(|i| modal.modes.get(*i).map(|m| m.text.clone()))
        .collect()
}

pub fn stack_object_view(g: &Game, viewer: Viewer, id: ObjectId) -> StackObjectView {
    let o = g.obj(id);
    let visible = can_see(g, viewer, id);
    let Some(si) = o.stack.as_deref() else {
        return StackObjectView {
            id: id.0,
            kind: "spell".into(),
            controller: o.controller.0,
            name: crate::describe::object_name(g, viewer, id),
            ..Default::default()
        };
    };
    let picks: Vec<usize> = si.chosen.iter().filter_map(|c| c.mode).collect();
    let targets: Vec<Vec<EntityRef>> = si
        .chosen
        .iter()
        .flat_map(|c| c.targets.iter())
        .map(|slot| slot.iter().map(|e| EntityRef::from(*e)).collect())
        .collect();
    let target_names: Vec<String> = si
        .chosen
        .iter()
        .flat_map(|c| c.targets.iter().flatten())
        .map(|e| crate::describe::entity_name(g, viewer, *e))
        .collect();
    let targets = if targets.iter().all(Vec::is_empty) {
        vec![]
    } else {
        targets
    };
    match &si.kind {
        StackKind::Spell => StackObjectView {
            id: id.0,
            kind: "spell".into(),
            controller: o.controller.0,
            name: crate::describe::object_name(g, viewer, id),
            source: None,
            text: if o.face_down && !visible {
                String::new()
            } else {
                o.chars.rules_text.to_string()
            },
            modes: if visible {
                mode_texts(&o.chars, None, &picks)
            } else {
                vec![]
            },
            x: si.x.or(si.cast.x),
            targets,
            target_names,
            face_down: o.face_down,
            card: Some(object_view(g, viewer, id)),
        },
        StackKind::Activated { source, ability } | StackKind::Triggered { source, ability } => {
            let activated = matches!(si.kind, StackKind::Activated { .. });
            let src_name = crate::describe::object_name(g, viewer, *source);
            StackObjectView {
                id: id.0,
                kind: if activated {
                    "activated_ability"
                } else {
                    "triggered_ability"
                }
                .into(),
                controller: o.controller.0,
                name: format!("ability of {src_name}"),
                source: Some(source.0),
                text: ability.text.clone(),
                modes: mode_texts(&o.chars, Some(&ability.kind), &picks),
                x: si.x,
                targets,
                target_names,
                face_down: false,
                card: None,
            }
        }
    }
}

/// A player's state as the viewer sees it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PlayerView {
    pub id: u8,
    pub name: String,
    pub life: i32,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub poison: u32,
    /// All player counters (poison, energy, experience, rad, ...).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub counters: BTreeMap<String, u32>,
    /// Mana in the pool, by type ("W", "U", "B", "R", "G", "C").
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub mana_pool: BTreeMap<String, u32>,
    pub hand_count: u32,
    /// The cards in the hand the viewer may see (all of them for the viewer's own hand).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hand: Vec<ObjectView>,
    pub library_count: u32,
    /// Cards in the library the viewer may see, with their position from the top
    /// (0 = top): a revealed top card, the top card its owner may look at (CR 401.5); in
    /// the omniscient view, the whole library in order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub library_known: Vec<(u32, ObjectView)>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub graveyard: Vec<ObjectView>,
    /// The cards in exile this player owns; face-down ones only show their identity to
    /// a player allowed to look at them (CR 406.3).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exile: Vec<ObjectView>,
    /// The objects in the command zone this player owns (commanders, emblems, ...).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub command: Vec<ObjectView>,
    pub lands_played_this_turn: u32,
    pub land_plays: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_hand_size: Option<i32>,
    pub team: u8,
    #[serde(default, skip_serializing_if = "is_false")]
    pub has_lost: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub has_won: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub left_game: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub monarch: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub initiative: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub citys_blessing: bool,
    /// Speed (CR 702.179).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<u32>,
    /// How many times the Ring has tempted this player, and their Ring-bearer (CR 701.54).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub ring_tempted: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ring_bearer: Option<u32>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub dungeons_completed: u32,
    /// The dungeon this player is venturing through and the room their venture marker is
    /// in (CR 309).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub venture: Option<(u32, u32)>,
    /// Commander damage dealt to this player, by commander name (CR 903.10a).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub commander_damage: BTreeMap<String, u32>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub mulligans: u32,
}

fn player_view(g: &Game, viewer: Viewer, p: PlayerId) -> PlayerView {
    let pl = g.player(p);
    let mut mana_pool: BTreeMap<String, u32> = BTreeMap::new();
    for m in &pl.mana_pool.mana {
        *mana_pool.entry(format!("{:?}", m.ty)).or_default() += 1;
    }
    let seen = |ids: &[ObjectId]| -> Vec<ObjectView> {
        ids.iter()
            .filter(|c| can_see(g, viewer, **c))
            .map(|c| object_view(g, viewer, *c))
            .collect()
    };
    // `library` is stored bottom to top.
    let lib = &pl.library;
    let library_known: Vec<(u32, ObjectView)> = lib
        .iter()
        .rev()
        .enumerate()
        .filter(|(_, c)| can_see(g, viewer, **c))
        .map(|(i, c)| (i as u32, object_view(g, viewer, *c)))
        .collect();
    let owned = |zone: &[ObjectId]| -> Vec<ObjectView> {
        zone.iter()
            .filter(|c| g.obj(**c).owner == p)
            .map(|c| object_view(g, viewer, *c))
            .collect()
    };
    PlayerView {
        id: p.0,
        name: pl.name.clone(),
        life: pl.life,
        poison: pl.poison(),
        counters: pl
            .counters
            .iter()
            .filter(|(_, n)| **n > 0)
            .map(|(k, n)| (k.to_string(), *n))
            .collect(),
        mana_pool,
        hand_count: pl.hand.len() as u32,
        hand: seen(&pl.hand),
        library_count: lib.len() as u32,
        library_known,
        graveyard: pl
            .graveyard
            .iter()
            .rev()
            .map(|c| object_view(g, viewer, *c))
            .collect(),
        exile: owned(&g.exile),
        command: owned(&g.command),
        lands_played_this_turn: pl.lands_played_this_turn,
        land_plays: pl.land_plays,
        max_hand_size: pl.max_hand_size,
        team: pl.team,
        has_lost: pl.has_lost,
        has_won: pl.has_won,
        left_game: pl.left_game,
        monarch: g.monarch == Some(p),
        initiative: g.initiative == Some(p),
        citys_blessing: pl.has_citys_blessing,
        speed: pl.speed,
        ring_tempted: pl.ring_level,
        ring_bearer: pl.ring_bearer.map(|o| o.0),
        dungeons_completed: pl.dungeons_completed,
        venture: pl.venture.map(|(d, r)| (d.0, r as u32)),
        commander_damage: pl
            .commander_damage
            .iter()
            .map(|(k, n)| (k.to_string(), *n))
            .collect(),
        mulligans: pl.mulligans,
    }
}

/// The result of a finished game.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultView {
    /// The winning players (a team wins together).
    Win(Vec<u8>),
    Draw,
}

impl From<&GameResult> for ResultView {
    fn from(r: &GameResult) -> Self {
        match r {
            GameResult::Win(ws) => ResultView::Win(ws.iter().map(|p| p.0).collect()),
            GameResult::Draw => ResultView::Draw,
        }
    }
}

/// Everything a player can see of the game (or everything, for `viewer: None`).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    /// The player whose view this is; absent for the omniscient view.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub viewer: Option<u8>,
    pub turn: u32,
    pub active_player: u8,
    pub phase: String,
    pub step: String,
    /// The player who has priority, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<u8>,
    pub players: Vec<PlayerView>,
    /// Permanents on the battlefield, with their current characteristics (CR 613).
    pub battlefield: Vec<ObjectView>,
    /// The stack, top first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stack: Vec<StackObjectView>,
    /// "day", "night", or absent if it's neither (CR 731).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub day_night: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub monarch: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initiative: Option<u8>,
    /// Objects in the command zone owned by no player in particular, or ante (rare).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ante: Vec<ObjectView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<ResultView>,
}

/// What `viewer` can see of the game now.
pub fn observe(g: &Game, viewer: Viewer) -> Observation {
    Observation {
        viewer: viewer.map(|p| p.0),
        turn: g.turn.number,
        active_player: g.turn.active.0,
        phase: phase_name(g.turn.step).into(),
        step: step_name(g.turn.step).into(),
        priority: g.turn.priority.map(|p| p.0),
        players: g
            .player_ids()
            .into_iter()
            .map(|p| player_view(g, viewer, p))
            .collect(),
        battlefield: g
            .battlefield
            .iter()
            .map(|id| object_view(g, viewer, *id))
            .collect(),
        stack: g
            .stack
            .iter()
            .rev()
            .map(|id| stack_object_view(g, viewer, *id))
            .collect(),
        day_night: g.day.map(|d| if d { "day" } else { "night" }.to_string()),
        monarch: g.monarch.map(|p| p.0),
        initiative: g.initiative.map(|p| p.0),
        ante: g
            .ante
            .iter()
            .map(|id| object_view(g, viewer, *id))
            .collect(),
        result: g.result.as_ref().map(ResultView::from),
    }
}
