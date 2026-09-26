//! The game state and core bookkeeping.

use crate::ability::*;
use crate::card::CardDef;
use crate::combat::CombatState;
use crate::decision::{Agent, Answer, Decision, PassiveAgent};
use crate::events::Event;
use crate::mana::ManaPool;
use crate::object::*;
use crate::turn::TurnState;
use crate::types::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

/// Game variants (CR 9xx) and multiplayer options (CR 8xx).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Variant {
    /// Two-player (or free-for-all multiplayer) constructed/limited Magic.
    Standard,
    /// CR 810.
    TwoHeadedGiant,
    /// CR 903.
    Commander,
    /// CR 901.
    Planechase,
    /// CR 904.
    Archenemy,
    /// CR 902.
    Vanguard,
    /// CR 809.
    Emperor,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameConfig {
    pub starting_life: i32,
    pub starting_hand_size: u32,
    pub variant: Variant,
    /// Multiplayer: range of influence (CR 801), `None` = unlimited.
    pub range_of_influence: Option<u32>,
    /// CR 802: attack multiple players.
    pub attack_multiple_players: bool,
    /// Teams by player index (CR 808, 810). Players with the same team id are teammates.
    pub teams: Option<Vec<u8>>,
    /// Starting player; `None` = random (CR 103.1).
    pub starting_player: Option<PlayerId>,
    /// Skip mulligans (keep opening hands).
    pub skip_mulligans: bool,
    /// Whether the starting player skips their first draw (CR 103.8a) — true for two
    /// player games.
    pub starting_player_skips_draw: Option<bool>,
    /// Commander: amount of combat damage from a single commander that loses the game.
    pub commander_damage_limit: u32,
    /// Commander: the Brawl option (CR 903.12).
    pub brawl: bool,
    /// Maximum number of turns before the game is declared a draw (simulation safety).
    pub max_turns: u32,
    /// Max decisions per game (safety valve for infinite loops).
    pub max_actions: u64,
    pub seed: u64,
    /// Planechase: play with a single communal planar deck (CR 901.15).
    #[serde(default)]
    pub single_planar_deck: bool,
    /// Limited play (CR 100.2b) rather than constructed play.
    #[serde(default)]
    pub limited: bool,
    /// The player who chooses who takes the first turn — in a match, the loser of the
    /// previous game (CR 103.1). `None` = determined at random.
    #[serde(default)]
    pub first_turn_chooser: Option<PlayerId>,
    /// Playing for ante, an optional variation (CR 407).
    #[serde(default)]
    pub ante: bool,
}

impl Default for GameConfig {
    fn default() -> Self {
        GameConfig {
            starting_life: 20,
            starting_hand_size: 7,
            variant: Variant::Standard,
            range_of_influence: None,
            attack_multiple_players: true,
            teams: None,
            starting_player: None,
            skip_mulligans: false,
            starting_player_skips_draw: None,
            commander_damage_limit: 21,
            brawl: false,
            max_turns: 200,
            max_actions: 200_000,
            seed: 0,
            single_planar_deck: false,
            limited: false,
            first_turn_chooser: None,
            ante: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub life: i32,
    pub library: Vec<ObjectId>,
    pub hand: Vec<ObjectId>,
    pub graveyard: Vec<ObjectId>,
    pub sideboard: Vec<ObjectId>,
    pub mana_pool: ManaPool,
    /// Player counters: poison, energy, experience, rad, ticket (CR 122.1).
    pub counters: BTreeMap<CounterKind, u32>,
    pub lands_played_this_turn: u32,
    pub has_lost: bool,
    pub has_won: bool,
    /// Left the game (lost or conceded in multiplayer, CR 800.4).
    pub left_game: bool,
    /// Attempted to draw from an empty library since the last SBA check (CR 704.5b).
    pub drew_from_empty_library: bool,
    pub team: u8,
    pub mulligans: u32,
    /// Speed (CR 702.179).
    pub speed: Option<u32>,
    pub speed_increased_this_turn: bool,
    /// Commander(s) owned by this player (the card objects' current ids are tracked by
    /// `is_commander` on objects).
    pub commander_names: Vec<SmolStr>,
    /// Combat damage dealt to this player by each commander (by commander name).
    pub commander_damage: BTreeMap<SmolStr, u32>,
    /// Times this player has cast each of their commanders from the command zone, by card
    /// name (CR 903.8; see `kw::partner::commander_key`).
    pub commander_casts: BTreeMap<SmolStr, u32>,
    /// Skip next N of the given step kind.
    pub skips: Vec<StepKind>,
    /// The Ring's level (CR 701.54), dungeons completed, etc.
    pub ring_level: u32,
    pub ring_bearer: Option<ObjectId>,
    pub dungeons_completed: u32,
    /// Venture marker: (dungeon object, room index).
    pub venture: Option<(ObjectId, usize)>,
    pub initiative_count: u32,
    pub has_citys_blessing: bool,
    /// Max hand size after effects (computed). `None` = no maximum.
    pub max_hand_size: Option<i32>,
    /// Land plays allowed this turn (computed, CR 305.2).
    pub land_plays: u32,
    /// Player-level static effects currently applying (computed).
    pub mods: Vec<PlayerModification>,
    /// Number of times this player has "expended" (mana spent this turn) etc.
    pub mana_spent_this_turn: u32,
    /// Timestamps of the beginnings of this player's two most recent upkeep steps, oldest
    /// first ("since the beginning of your last upkeep", CR 702.30a).
    #[serde(default)]
    pub upkeeps_begun: Vec<Timestamp>,
}

impl Player {
    pub fn new(id: PlayerId, name: String, life: i32) -> Player {
        Player {
            id,
            name,
            life,
            library: vec![],
            hand: vec![],
            graveyard: vec![],
            sideboard: vec![],
            mana_pool: ManaPool::default(),
            counters: BTreeMap::new(),
            lands_played_this_turn: 0,
            has_lost: false,
            has_won: false,
            left_game: false,
            drew_from_empty_library: false,
            team: id.0,
            mulligans: 0,
            speed: None,
            speed_increased_this_turn: false,
            commander_names: vec![],
            commander_damage: BTreeMap::new(),
            commander_casts: BTreeMap::new(),
            skips: vec![],
            ring_level: 0,
            ring_bearer: None,
            dungeons_completed: 0,
            venture: None,
            initiative_count: 0,
            has_citys_blessing: false,
            max_hand_size: Some(7),
            land_plays: 1,
            mods: vec![],
            mana_spent_this_turn: 0,
            upkeeps_begun: vec![],
        }
    }
    pub fn counter(&self, k: &str) -> u32 {
        self.counters.get(k).copied().unwrap_or(0)
    }
    pub fn poison(&self) -> u32 {
        self.counter(counters::POISON)
    }
    pub fn in_game(&self) -> bool {
        !self.left_game && !self.has_lost
    }
    pub fn has_mod(&self, f: impl Fn(&PlayerModification) -> bool) -> bool {
        self.mods.iter().any(f)
    }
}

/// What a resolved continuous effect applies to.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Affected {
    /// A fixed set of objects, locked in when the effect began (CR 611.2c).
    Objects(Vec<ObjectId>),
    /// Objects matching a filter, evaluated continuously (used for effects created by
    /// static-like resolved abilities such as "creatures you control get +1/+1 for as long as ...").
    Filter(Filter),
}

/// Special layer 1 effects.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Layer1 {
    /// Become a copy of the given copiable values (CR 707), with exceptions applied on top.
    Copy {
        values: Box<Characteristics>,
        exceptions: Vec<Modification>,
    },
    /// Changes to copiable values made by an "as [this] enters" or "as [this] is turned
    /// face up" ability that sets power and toughness (CR 613.2a).
    Copiable(Vec<Modification>),
}

/// A continuous effect generated by a resolving spell or ability (CR 611.2).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContinuousEffect {
    pub id: u32,
    pub source: Option<ObjectId>,
    pub controller: PlayerId,
    pub timestamp: Timestamp,
    pub duration: Duration,
    pub affected: Affected,
    pub mods: Vec<Modification>,
    pub layer1: Option<Layer1>,
    /// Turn number the effect was created (for "until your next turn").
    pub created_turn: u32,
}

/// A rule-modifying effect from a resolved spell/ability (CR 613.11).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleEffect {
    pub id: u32,
    pub source: Option<ObjectId>,
    pub controller: PlayerId,
    pub timestamp: Timestamp,
    pub duration: Duration,
    pub restriction: Restriction,
    /// Objects the restriction was locked to, if the effect named specific objects.
    pub objects: Option<Vec<ObjectId>>,
}

/// A player-affecting effect from a resolved spell/ability (CR 613.10).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerEffect {
    pub id: u32,
    pub players: Vec<PlayerId>,
    pub controller: PlayerId,
    pub timestamp: Timestamp,
    pub duration: Duration,
    pub source: Option<ObjectId>,
    pub effect: PlayerModification,
}

/// A replacement or prevention effect from a resolved spell/ability (shields,
/// regeneration, "the next time ... this turn").
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplacementInstance {
    pub id: u32,
    pub source: Option<ObjectId>,
    pub controller: PlayerId,
    pub timestamp: Timestamp,
    pub duration: Duration,
    pub def: ReplacementDef,
    /// Remaining uses (None = unlimited during duration).
    pub uses: Option<u32>,
    /// Objects this effect protects/affects, if locked.
    pub objects: Option<Vec<ObjectId>>,
    /// Remaining prevention amount for "prevent the next N damage" shields.
    pub remaining: Option<u32>,
}

/// A delayed triggered ability (CR 603.7).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DelayedTrigger {
    pub id: u32,
    pub source: Option<ObjectId>,
    pub controller: PlayerId,
    pub trigger: TriggerCond,
    pub body: Body,
    pub once: bool,
    /// Context captured when created (targets/vars from the creating ability).
    pub ctx: crate::resolve::SavedCtx,
    pub created_turn: u32,
    /// For "at the beginning of the next end step": don't fire in the step it was created in.
    pub created_step: Option<crate::turn::Step>,
    /// A delayed trigger that can trigger more than once lasts "for the rest of the game"
    /// rather than for the turn (e.g. epic, CR 702.50a).
    #[serde(default)]
    pub for_rest_of_game: bool,
}

/// A triggered ability waiting to be put on the stack (CR 603.3).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PendingTrigger {
    pub source: ObjectId,
    pub controller: PlayerId,
    pub ability: Ability,
    pub event: EventInfo,
    pub source_lki: Option<Box<Characteristics>>,
    /// For delayed triggers: the saved context.
    pub saved: Option<crate::resolve::SavedCtx>,
    /// Body override (delayed triggers carry their own body).
    pub body: Option<Body>,
    pub order: u64,
}

/// Facts about the current turn used by abilities that look back ("if a creature died
/// this turn", storm count, etc.).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TurnHistory {
    /// (caster, spell object id) in cast order.
    pub spells_cast: Vec<(PlayerId, ObjectId)>,
    pub creatures_died: Vec<ObjectId>,
    pub permanents_left: Vec<ObjectId>,
    pub cards_drawn: BTreeMap<PlayerId, u32>,
    pub life_gained: BTreeMap<PlayerId, u32>,
    pub life_lost: BTreeMap<PlayerId, u32>,
    pub damage_dealt_to_players: BTreeMap<PlayerId, u32>,
    pub players_attacked: BTreeSet<PlayerId>,
    pub attackers: Vec<ObjectId>,
    pub objects_dealt_damage: BTreeSet<ObjectId>,
    /// (source, object) pairs: objects dealt damage this turn and by what.
    pub damage_by_source: BTreeSet<(ObjectId, ObjectId)>,
    pub lands_played: BTreeMap<PlayerId, u32>,
    pub tokens_created: BTreeMap<PlayerId, u32>,
    pub cards_left_graveyard: BTreeMap<PlayerId, u32>,
    pub sacrificed: Vec<(PlayerId, ObjectId)>,
    pub crimes: BTreeMap<PlayerId, u32>,
    pub counters_put: u32,
    pub descended: BTreeMap<PlayerId, u32>,
    /// Mana each player spent this turn to cast spells (CR 700.14, "expend").
    pub spell_mana_spent: BTreeMap<PlayerId, u32>,
    /// Combat damage dealt to players this turn, with what its source was as it dealt
    /// the damage (e.g. for prowl, CR 702.76a). Recorded by `kw/prowl.rs`.
    #[serde(default)]
    pub combat_damage_to_players: Vec<crate::kw::prowl::CombatDamageRecord>,
    /// Creatures tapped this turn to pay the cost of a Vehicle's crew ability, which
    /// "crewed" it (CR 702.122b–c). Recorded by `kw/crew.rs`.
    #[serde(default)]
    pub crewed: Vec<crate::kw::crew::CrewRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameResult {
    Win(Vec<PlayerId>),
    Draw,
}

/// A log line for debugging and replays.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub turn: u32,
    pub text: String,
}

/// Shared handle to the agents making decisions. Cloning a [`Game`] shares the agents;
/// replace them with [`Game::set_agents`] before simulating on a clone.
#[derive(Clone)]
pub struct Agents(pub Arc<Mutex<Vec<Box<dyn Agent>>>>);

impl std::fmt::Debug for Agents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Agents")
    }
}

/// Active static abilities after layer computation, for fast rule queries.
#[derive(Clone, Debug, Default)]
pub struct ActiveStatics {
    /// (source, controller, timestamp, effect)
    pub restrictions: Vec<(ObjectId, PlayerId, Restriction)>,
    pub cost_modifiers: Vec<(ObjectId, PlayerId, CostModifier)>,
    pub replacements: Vec<(ObjectId, PlayerId, Timestamp, Ability, ReplacementDef)>,
    pub play_permissions: Vec<(ObjectId, PlayerId, PlayPermission)>,
    pub flash_permissions: Vec<(ObjectId, PlayerId, PlayerRel, Filter)>,
    pub customs: Vec<(ObjectId, PlayerId, SmolStr)>,
    pub other: Vec<(ObjectId, PlayerId, StaticEffect)>,
}

#[derive(Clone, Debug)]
pub struct Game {
    pub config: GameConfig,
    pub objects: Vec<GameObject>,
    pub players: Vec<Player>,
    pub battlefield: Vec<ObjectId>,
    /// Top of the stack is the last element.
    pub stack: Vec<ObjectId>,
    pub exile: Vec<ObjectId>,
    pub command: Vec<ObjectId>,
    pub ante: Vec<ObjectId>,
    pub turn: TurnState,
    pub combat: Option<CombatState>,
    pub effects: Vec<ContinuousEffect>,
    pub rule_effects: Vec<RuleEffect>,
    pub player_effects: Vec<PlayerEffect>,
    pub replacements: Vec<ReplacementInstance>,
    pub delayed_triggers: Vec<DelayedTrigger>,
    pub pending_triggers: Vec<PendingTrigger>,
    pub statics: ActiveStatics,
    pub history: TurnHistory,
    /// The previous turn's history ("if no spells were cast last turn").
    pub last_turn_history: TurnHistory,
    pub result: Option<GameResult>,
    pub rng: ChaCha8Rng,
    pub agents: Agents,
    /// Events emitted since the last trigger check.
    pub events: Vec<Event>,
    /// All events this turn (for look-back queries).
    pub turn_events: Vec<Event>,
    pub log: Vec<LogEntry>,
    pub logging: bool,
    pub next_timestamp: Timestamp,
    pub next_effect_id: u32,
    pub trigger_order: u64,
    pub actions_taken: u64,
    /// Characteristic recomputation needed.
    pub dirty: bool,
    /// Game-level designations.
    pub monarch: Option<PlayerId>,
    pub initiative: Option<PlayerId>,
    /// Day/night (CR 731): None = neither.
    pub day: Option<bool>,
    /// Number of spells the active player cast last turn, for day/night (with shared
    /// team turns, the most any player of the active team cast, CR 502.2a).
    pub spells_cast_last_turn_by_active: u32,
    /// Extra turns queued (CR 500.7): taken after the current turn, most recent first.
    pub extra_turns: Vec<PlayerId>,
    /// Pending "the next time state-based actions are checked" flags etc.
    pub sba_flags: BTreeSet<SmolStr>,
    /// Count of state-based-action checks performed (for debugging/tests).
    pub sba_checks: u64,
    /// Planechase: planar die rolls this turn (CR 901.9).
    pub planar_die_rolls: u32,
    /// Replacement effects already applied to the event currently being replaced (for
    /// "instead" effects that generate nested events, CR 614.5).
    pub repl_context: Vec<Vec<crate::replacement::ReplKey>>,
    /// Effects to run after an event that a replacement modified with "and also ...".
    pub post_replacement_effects: Vec<(crate::eval::Ctx, Effect)>,
    /// Link id (CR 607) used when recording exiled cards on the source.
    pub current_link: u16,
    /// State triggers that have triggered and haven't left the stack (CR 603.8).
    pub state_triggers_active: BTreeSet<(ObjectId, u64)>,
    /// Resolution contexts saved for stack objects (activated-ability costs, delayed triggers).
    pub saved_ctx: BTreeMap<ObjectId, crate::eval::Ctx>,
    /// "You may play/cast that card" permissions from resolved effects.
    pub play_grants: Vec<crate::casting::PlayGrant>,
    /// The permanent whose mana ability is resolving (for "tapped for mana" triggers).
    pub mana_ability_resolving: Option<ObjectId>,
    /// Set while several players lose the game simultaneously, so the game's result is
    /// determined only once all of them have lost (CR 104.4a).
    pub losing_simultaneously: bool,
    /// Cards exiled before the game began by an ability of a card with a given name:
    /// (player who exiled it, that card's name, the exiled card) (CR 607.2n).
    pub named_exiles: Vec<(PlayerId, SmolStr, ObjectId)>,
    /// Hint of which mana types an automatic payment needs (for "any color" choices).
    pub mana_hint: Option<Vec<crate::mana::ManaType>>,
    /// Commanders that moved to graveyard/exile since the last SBA check (CR 704.6d).
    pub commander_moved_since_last_sba: BTreeSet<ObjectId>,
    /// When a search's decision is answered with the default, find the first matches.
    pub search_finds_by_default: bool,
    /// One-shot effects waiting "until" an event to undo themselves (CR 610.3, 610.4).
    pub untils: Vec<crate::until::UntilEffect>,
    /// Continuous effects waiting for the next spell a player casts (CR 611.2f).
    pub next_spell_effects: Vec<crate::next_spell::NextSpellEffect>,
    /// Actions to take as the first thing in the next step that occurs, after a skip
    /// (CR 614.10b).
    pub step_start_actions: Vec<(crate::eval::Ctx, Effect)>,
    /// Objects entering the battlefield simultaneously right now: effects modifying how
    /// they enter can't choose them to change zones (CR 614.13a, 614.13c).
    pub entering: Vec<ObjectId>,
    /// While tokens are being created "attached to" an object or player: what they enter
    /// attached to (`Some(None)`: an undefined object or player), CR 303.4f–i, 301.5e.
    pub token_attach: Option<Option<Entity>>,
    /// While tokens are being created that "enter with" counters (incubate, CR 701.53a):
    /// the counters each of them enters with (CR 122.6).
    pub token_counters: Vec<(CounterKind, u32)>,
    /// State kept by keyword actions (CR 701): see `kwa/`.
    pub kwa: crate::kwa::KwaState,
    /// Continuous effects on permanent spells that keep applying to the permanents they
    /// become (CR 611.3d).
    pub carried_effects: Vec<u32>,
    /// The continuous effects that are stickers on objects (CR 123).
    pub stickers: Vec<u32>,
    /// Game-ending bookkeeping: draws for individual players, mandatory loops (CR 104).
    pub end: crate::game_end::EndState,
    /// Choices made so far by players choosing at the same time (CR 101.4).
    pub apnap_choices: Vec<crate::apnap::ApnapChoice>,
    /// What happened while starting the game (CR 103).
    pub start: crate::start::StartState,
    /// Special actions allowed by effects and effects being ignored (CR 116.2c, 116.2d).
    pub special: crate::special_actions::SpecialState,
    /// Coins and dice (CR 705, 706).
    pub dice: crate::dice::DiceState,
    /// Zone bookkeeping: face-down exiled cards players may look at, revealed top cards
    /// of libraries (CR 401.5, 401.6, 406.3).
    pub zones: crate::zones::ZoneState,
    /// Cards currently revealed and for how long (CR 701.20).
    pub reveals: crate::reveal::RevealState,
    /// Library searches in progress (CR 701.23h).
    pub searches: crate::search_rules::SearchState,
    /// When permanents last transformed (CR 701.27f).
    pub transforms: crate::transform_rules::TransformState,
    /// Players controlling other players (CR 723).
    pub player_control: crate::player_control::PlayerControlState,
    /// Merged permanents that left the battlefield (CR 730.3).
    pub merges: crate::merge::MergeState,
    /// Subgames (CR 729).
    pub subgames: crate::subgame::SubgameState,
    /// Shortcuts and loops (CR 732).
    pub shortcuts: crate::shortcuts::ShortcutState,
    /// Modes chosen for modal abilities ("choose one that hasn't been chosen", CR 700.2).
    pub modal_history: crate::modal_history::ModalHistory,
}

impl Game {
    /// Creates a game with the given decks (one per player) and agents. Cards are put into
    /// libraries (not shuffled); call [`Game::start`] to shuffle, draw, and mulligan.
    pub fn new(
        config: GameConfig,
        decks: Vec<Vec<Arc<CardDef>>>,
        agents: Vec<Box<dyn Agent>>,
    ) -> Game {
        let n = decks.len();
        assert!(n >= 1, "a game needs at least one player");
        let mut agents = agents;
        while agents.len() < n {
            agents.push(Box::new(PassiveAgent));
        }
        let mut g = Game {
            rng: ChaCha8Rng::seed_from_u64(config.seed),
            players: (0..n)
                .map(|i| {
                    Player::new(
                        PlayerId(i as u8),
                        format!("Player {}", i + 1),
                        config.starting_life,
                    )
                })
                .collect(),
            config,
            objects: vec![],
            battlefield: vec![],
            stack: vec![],
            exile: vec![],
            command: vec![],
            ante: vec![],
            turn: TurnState::new(PlayerId(0)),
            combat: None,
            effects: vec![],
            rule_effects: vec![],
            player_effects: vec![],
            replacements: vec![],
            delayed_triggers: vec![],
            pending_triggers: vec![],
            statics: ActiveStatics::default(),
            history: TurnHistory::default(),
            last_turn_history: TurnHistory::default(),
            result: None,
            agents: Agents(Arc::new(Mutex::new(agents))),
            events: vec![],
            turn_events: vec![],
            log: vec![],
            logging: false,
            next_timestamp: 1,
            next_effect_id: 1,
            trigger_order: 0,
            actions_taken: 0,
            dirty: true,
            monarch: None,
            initiative: None,
            day: None,
            spells_cast_last_turn_by_active: 0,
            extra_turns: vec![],
            sba_flags: BTreeSet::new(),
            sba_checks: 0,
            planar_die_rolls: 0,
            repl_context: vec![],
            post_replacement_effects: vec![],
            current_link: 0,
            state_triggers_active: BTreeSet::new(),
            saved_ctx: BTreeMap::new(),
            play_grants: vec![],
            mana_ability_resolving: None,
            losing_simultaneously: false,
            named_exiles: vec![],
            mana_hint: None,
            commander_moved_since_last_sba: BTreeSet::new(),
            search_finds_by_default: true,
            untils: vec![],
            next_spell_effects: vec![],
            step_start_actions: vec![],
            entering: vec![],
            token_attach: None,
            token_counters: vec![],
            kwa: Default::default(),
            carried_effects: vec![],
            stickers: vec![],
            end: Default::default(),
            apnap_choices: vec![],
            start: Default::default(),
            special: Default::default(),
            dice: Default::default(),
            zones: Default::default(),
            reveals: Default::default(),
            searches: Default::default(),
            transforms: Default::default(),
            player_control: Default::default(),
            merges: Default::default(),
            subgames: Default::default(),
            shortcuts: Default::default(),
            modal_history: Default::default(),
        };
        if let Some(teams) = g.config.teams.clone() {
            for (i, t) in teams.iter().enumerate() {
                if let Some(p) = g.players.get_mut(i) {
                    p.team = *t;
                }
            }
        }
        // CR 119.1: starting life totals (which depend on the variant and teams).
        crate::life_totals::set_starting_life_totals(&mut g);
        for (i, deck) in decks.into_iter().enumerate() {
            let pid = PlayerId(i as u8);
            for card in deck {
                // Nontraditional cards aren't part of the deck (CR 108.2a, 108.5).
                // Conspiracies can't be included in a deck; they start in the sideboard
                // (CR 315.3, 905.4).
                if card.front().chars.card_types.contains(CardType::Conspiracy) {
                    g.add_to_sideboard(pid, vec![card]);
                    continue;
                }
                if crate::variants::is_nontraditional(&card) {
                    let (zone, face_down) = crate::variants::nontraditional_start(&card, pid);
                    let id = g.create_card_object(card, pid, zone);
                    g.objects[id.0 as usize].face_down = face_down;
                    if let Some(list) = g.zone_list_mut(zone) {
                        list.push(id);
                    }
                    continue;
                }
                let id = g.create_card_object(card, pid, Zone::Library(pid));
                g.players[i].library.push(id);
            }
        }
        g.recompute();
        g
    }

    /// Replaces the agents (e.g. on a cloned game used for search).
    pub fn set_agents(&mut self, agents: Vec<Box<dyn Agent>>) {
        self.agents = Agents(Arc::new(Mutex::new(agents)));
    }

    pub fn set_agent(&mut self, p: PlayerId, agent: Box<dyn Agent>) {
        self.agents.0.lock().unwrap()[p.idx()] = agent;
    }

    // ------------------------------------------------------------------
    // Basic accessors
    // ------------------------------------------------------------------

    pub fn obj(&self, id: ObjectId) -> &GameObject {
        &self.objects[id.0 as usize]
    }

    pub fn obj_mut(&mut self, id: ObjectId) -> &mut GameObject {
        self.dirty = true;
        &mut self.objects[id.0 as usize]
    }

    pub fn try_obj(&self, id: ObjectId) -> Option<&GameObject> {
        self.objects.get(id.0 as usize)
    }

    pub fn player(&self, p: PlayerId) -> &Player {
        &self.players[p.idx()]
    }

    pub fn player_mut(&mut self, p: PlayerId) -> &mut Player {
        &mut self.players[p.idx()]
    }

    pub fn player_ids(&self) -> Vec<PlayerId> {
        self.players.iter().map(|p| p.id).collect()
    }

    /// Players still in the game, in turn order starting from the active player
    /// (APNAP order, CR 101.4).
    pub fn apnap(&self) -> Vec<PlayerId> {
        let n = self.players.len();
        let a = self.turn.active.idx();
        (0..n)
            .map(|i| PlayerId(((a + i) % n) as u8))
            .filter(|p| self.player(*p).in_game())
            .collect()
    }

    pub fn players_in_game(&self) -> Vec<PlayerId> {
        self.players
            .iter()
            .filter(|p| p.in_game())
            .map(|p| p.id)
            .collect()
    }

    pub fn active_player(&self) -> PlayerId {
        self.turn.active
    }

    /// Opponents of a player (CR 102.3): in team games, players not on their team.
    pub fn opponents(&self, p: PlayerId) -> Vec<PlayerId> {
        let team = self.player(p).team;
        self.players
            .iter()
            .filter(|q| q.in_game() && q.team != team)
            .map(|q| q.id)
            .collect()
    }

    pub fn are_opponents(&self, a: PlayerId, b: PlayerId) -> bool {
        self.player(a).team != self.player(b).team
    }

    pub fn teammates(&self, p: PlayerId) -> Vec<PlayerId> {
        let team = self.player(p).team;
        self.players
            .iter()
            .filter(|q| q.in_game() && q.team == team && q.id != p)
            .map(|q| q.id)
            .collect()
    }

    /// Next player in turn order who is still in the game.
    pub fn next_player(&self, p: PlayerId) -> PlayerId {
        let n = self.players.len();
        for i in 1..=n {
            let q = PlayerId(((p.idx() + i) % n) as u8);
            if self.player(q).in_game() {
                return q;
            }
        }
        p
    }

    pub fn is_over(&self) -> bool {
        self.result.is_some()
    }

    pub fn new_timestamp(&mut self) -> Timestamp {
        let t = self.next_timestamp;
        self.next_timestamp += 1;
        t
    }

    pub fn new_effect_id(&mut self) -> u32 {
        let t = self.next_effect_id;
        self.next_effect_id += 1;
        t
    }

    /// The ids of objects currently in a zone (ordered for ordered zones; for libraries,
    /// the last element is the top card).
    pub fn zone_objects(&self, zone: Zone) -> Vec<ObjectId> {
        match zone {
            Zone::Library(p) => self.player(p).library.clone(),
            Zone::Hand(p) => self.player(p).hand.clone(),
            Zone::Graveyard(p) => self.player(p).graveyard.clone(),
            Zone::Battlefield => self.battlefield.clone(),
            Zone::Stack => self.stack.clone(),
            Zone::Exile => self.exile.clone(),
            Zone::Command => self.command.clone(),
            Zone::Ante => self.ante.clone(),
            Zone::Outside(p) => self.player(p).sideboard.clone(),
            Zone::Nowhere => vec![],
        }
    }

    pub(crate) fn zone_list_mut(&mut self, zone: Zone) -> Option<&mut Vec<ObjectId>> {
        Some(match zone {
            Zone::Library(p) => &mut self.players[p.idx()].library,
            Zone::Hand(p) => &mut self.players[p.idx()].hand,
            Zone::Graveyard(p) => &mut self.players[p.idx()].graveyard,
            Zone::Battlefield => &mut self.battlefield,
            Zone::Stack => &mut self.stack,
            Zone::Exile => &mut self.exile,
            Zone::Command => &mut self.command,
            Zone::Ante => &mut self.ante,
            Zone::Outside(p) => &mut self.players[p.idx()].sideboard,
            Zone::Nowhere => return None,
        })
    }

    /// Battlefield permanents that are phased in (phased-out permanents are treated as
    /// though they don't exist, CR 702.26b).
    pub fn permanents(&self) -> impl Iterator<Item = &GameObject> + '_ {
        self.battlefield
            .iter()
            .map(|id| self.obj(*id))
            .filter(|o| !o.phased_out)
    }

    pub fn permanent_ids(&self) -> Vec<ObjectId> {
        self.permanents().map(|o| o.id).collect()
    }

    pub fn creatures(&self) -> Vec<ObjectId> {
        self.permanents()
            .filter(|o| o.is_creature())
            .map(|o| o.id)
            .collect()
    }

    pub fn permanents_controlled_by(&self, p: PlayerId) -> Vec<ObjectId> {
        self.permanents()
            .filter(|o| o.controller == p)
            .map(|o| o.id)
            .collect()
    }

    /// Top card of a player's library.
    pub fn library_top(&self, p: PlayerId) -> Option<ObjectId> {
        self.player(p).library.last().copied()
    }

    /// Current incarnation of an object that may have changed zones (follows `next`).
    pub fn current(&self, mut id: ObjectId) -> ObjectId {
        while let Some(n) = self.obj(id).next {
            id = n;
        }
        id
    }

    /// Whether the object is still the same object in the given zone (CR 400.7).
    pub fn is_in_zone(&self, id: ObjectId, zone: Zone) -> bool {
        let o = self.obj(id);
        o.zone == zone && o.next.is_none()
    }

    /// Finds objects by name in a zone.
    pub fn find_in_zone(&self, zone: Zone, name: &str) -> Vec<ObjectId> {
        self.zone_objects(zone)
            .into_iter()
            .filter(|id| self.obj(*id).chars.name.eq_ignore_ascii_case(name))
            .collect()
    }

    // ------------------------------------------------------------------
    // Object creation
    // ------------------------------------------------------------------

    fn push_object(&mut self, mut obj: GameObject) -> ObjectId {
        let id = ObjectId(self.objects.len() as u32);
        obj.id = id;
        obj.timestamp = self.new_timestamp();
        obj.control_since = obj.timestamp;
        obj.entered_turn = self.turn.number;
        self.objects.push(obj);
        self.dirty = true;
        id
    }

    /// Creates a card object in a zone (does not add it to the zone list).
    pub fn create_card_object(
        &mut self,
        card: Arc<CardDef>,
        owner: PlayerId,
        zone: Zone,
    ) -> ObjectId {
        let base = card.characteristics(FaceState::Front);
        let mut obj = GameObject::new(ObjectId(0), ObjKind::Card, owner, zone, base);
        obj.card = Some(card);
        self.push_object(obj)
    }

    /// Creates a new object that is the next incarnation of `old` in `zone` (CR 400.7).
    pub(crate) fn create_incarnation(&mut self, old: ObjectId, zone: Zone) -> ObjectId {
        let o = self.obj(old).clone();
        let mut n = GameObject::new(ObjectId(0), o.kind, o.owner, zone, o.base.clone());
        n.card = o.card.clone();
        n.face = match o.face {
            // Cards return to their front face when they leave the battlefield/stack
            // (CR 712.14, 709.4), except that some effects put cards into zones transformed.
            FaceState::Half(_)
            | FaceState::Fused
            | FaceState::Flipped
            | FaceState::Back
            | FaceState::Melded => FaceState::Front,
            f => f,
        };
        if let Some(card) = &n.card {
            n.base = card.characteristics(n.face);
            // Until characteristics are next computed (never, for most cards in a
            // library): the card's own (e.g. a transformed card's front face, CR 712.8a).
            n.chars = n.base.clone();
            n.copiable = n.base.clone();
        }
        n.is_commander = o.is_commander;
        // CR 607.2p: a choice made before the game began follows the card.
        if let Some(c) = o.linked_choices.get(&crate::ability::PREGAME_LINK) {
            n.linked_choices
                .insert(crate::ability::PREGAME_LINK, c.clone());
        }
        n.prev = Some(old);
        let id = self.push_object(n);
        self.objects[old.0 as usize].next = Some(id);
        crate::stickers::follow(self, old, id, zone);
        crate::rooms::entering(self, old, id, zone);
        crate::merge::incarnation(self, old, id);
        id
    }

    /// Creates a token object on the battlefield directly (no replacement effects).
    /// Most code should use [`Game::create_tokens`] instead.
    pub(crate) fn create_token_object(
        &mut self,
        chars: Characteristics,
        owner: PlayerId,
    ) -> ObjectId {
        let obj = GameObject::new(ObjectId(0), ObjKind::Token, owner, Zone::Nowhere, chars);
        self.push_object(obj)
    }

    // ------------------------------------------------------------------
    // Decisions
    // ------------------------------------------------------------------

    /// Asks a player's agent to make a decision.
    pub fn ask(&mut self, player: PlayerId, decision: Decision) -> Answer {
        if self.dirty {
            self.recompute();
        }
        self.actions_taken += 1;
        // CR 723.5: the decisions of a player controlled by another player are made by
        // that other player.
        let decider = crate::player_control::decider(self, player);
        let decision = if decider != player {
            crate::player_control::for_controller(self, decision)
        } else {
            decision
        };
        let agents = self.agents.clone();
        let mut guard = agents
            .0
            .lock()
            .expect("agent mutex poisoned (re-entrant ask?)");
        let answer = guard[decider.idx()].decide(self, player, &decision);
        drop(guard);
        if decider != player {
            crate::player_control::check_answer(answer)
        } else {
            answer
        }
    }

    /// Asks a yes/no question; `Default` answers `default`.
    pub fn ask_yes_no(
        &mut self,
        player: PlayerId,
        source: Option<ObjectId>,
        prompt: &str,
        default: bool,
    ) -> bool {
        match self.ask(
            player,
            Decision::YesNo {
                source,
                prompt: prompt.to_string(),
            },
        ) {
            Answer::Bool(b) => b,
            _ => default,
        }
    }

    /// Asks a player to choose between `min` and `max` entities from `candidates`.
    /// Invalid or default answers choose the first `min` candidates.
    pub fn ask_entities(
        &mut self,
        player: PlayerId,
        source: Option<ObjectId>,
        prompt: &str,
        candidates: Vec<Entity>,
        min: u32,
        max: u32,
    ) -> Vec<Entity> {
        let max = max.min(candidates.len() as u32);
        let min = min.min(max);
        if candidates.is_empty() || max == 0 {
            return vec![];
        }
        let ans = self.ask(
            player,
            Decision::ChooseEntities {
                source,
                prompt: prompt.to_string(),
                candidates: candidates.clone(),
                min,
                max,
            },
        );
        if let Answer::Entities(v) = ans {
            let mut seen = BTreeSet::new();
            let ok = v.len() as u32 >= min
                && v.len() as u32 <= max
                && v.iter().all(|e| candidates.contains(e) && seen.insert(*e));
            if ok {
                return v;
            }
        }
        candidates.into_iter().take(min as usize).collect()
    }

    pub fn ask_objects(
        &mut self,
        player: PlayerId,
        source: Option<ObjectId>,
        prompt: &str,
        candidates: Vec<ObjectId>,
        min: u32,
        max: u32,
    ) -> Vec<ObjectId> {
        let c = candidates.into_iter().map(Entity::Object).collect();
        self.ask_entities(player, source, prompt, c, min, max)
            .into_iter()
            .filter_map(Entity::object)
            .collect()
    }

    pub fn ask_option(
        &mut self,
        player: PlayerId,
        source: Option<ObjectId>,
        prompt: &str,
        options: Vec<String>,
    ) -> usize {
        if options.len() <= 1 {
            return 0;
        }
        let n = options.len();
        match self.ask(
            player,
            Decision::ChooseOption {
                source,
                prompt: prompt.to_string(),
                options,
            },
        ) {
            Answer::Index(i) if i < n => i,
            _ => 0,
        }
    }

    pub fn ask_number(
        &mut self,
        player: PlayerId,
        source: Option<ObjectId>,
        prompt: &str,
        min: i64,
        max: i64,
    ) -> i64 {
        match self.ask(
            player,
            Decision::ChooseNumber {
                source,
                prompt: prompt.to_string(),
                min,
                max,
            },
        ) {
            Answer::Number(n) if n >= min && n <= max => n,
            _ => min,
        }
    }

    /// Orders items; returns a permutation of indices.
    pub fn ask_order(&mut self, player: PlayerId, prompt: &str, items: Vec<String>) -> Vec<usize> {
        let n = items.len();
        if n <= 1 {
            return (0..n).collect();
        }
        if let Answer::Indices(v) = self.ask(
            player,
            Decision::Order {
                prompt: prompt.to_string(),
                items,
            },
        ) {
            let mut sorted = v.clone();
            sorted.sort_unstable();
            if sorted == (0..n).collect::<Vec<_>>() {
                return v;
            }
        }
        (0..n).collect()
    }

    // ------------------------------------------------------------------
    // Randomness
    // ------------------------------------------------------------------

    pub fn shuffle_library(&mut self, p: PlayerId) {
        use rand::seq::SliceRandom;
        let mut lib = std::mem::take(&mut self.players[p.idx()].library);
        lib.shuffle(&mut self.rng);
        self.players[p.idx()].library = lib;
        // CR 401.6: a revealed top card stops being revealed.
        crate::zones::library_shuffled(self, p);
        // CR 701.20d: revealed cards that are reordered stop being revealed.
        crate::reveal::library_reordered(self, p);
        // CR 701.23h: a search of this library is over.
        crate::search_rules::library_shuffled(self, p);
        self.emit(Event::Shuffled { player: p });
    }

    pub fn random_range(&mut self, lo: u32, hi_inclusive: u32) -> u32 {
        self.rng.gen_range(lo..=hi_inclusive)
    }

    // ------------------------------------------------------------------
    // Logging and events
    // ------------------------------------------------------------------

    pub fn log(&mut self, text: impl FnOnce(&Game) -> String) {
        if self.logging {
            let t = text(self);
            self.log.push(LogEntry {
                turn: self.turn.number,
                text: t,
            });
        }
    }

    pub fn emit(&mut self, e: Event) {
        self.events.push(e);
    }

    /// Describes an object for logs.
    pub fn describe(&self, id: ObjectId) -> String {
        let o = self.obj(id);
        if o.face_down {
            format!("face-down {}", id)
        } else {
            format!("{} {}", o.chars.name, id)
        }
    }

    // ------------------------------------------------------------------
    // Game end (CR 104)
    // ------------------------------------------------------------------

    /// Makes a player lose the game (CR 104.3), subject to "can't lose" effects and
    /// replacement effects (applied by the caller in SBA processing).
    pub fn player_loses(&mut self, p: PlayerId) {
        if self.player(p).has_lost || self.result.is_some() {
            return;
        }
        self.log(|_g| format!("{p} loses the game"));
        self.players[p.idx()].has_lost = true;
        self.emit(Event::PlayerLost { player: p });
        // CR 603.10f: abilities that trigger when a player loses the game look back in
        // time, before the player's objects leave the game (CR 800.4a).
        self.flush_events();
        self.after_player_leaves(p);
        self.on_player_lost(p);
        if !self.losing_simultaneously {
            self.check_game_over();
        }
    }

    /// An effect says that a player wins the game (CR 104.2b); see [`Game::players_win`].
    pub fn player_wins(&mut self, p: PlayerId) {
        self.players_win(&[p]);
    }

    /// Handles a player leaving a multiplayer game (CR 800.4a).
    pub(crate) fn after_player_leaves(&mut self, p: PlayerId) {
        self.players[p.idx()].left_game = true;
        // CR 708.9: their face-down permanents and spells are revealed.
        crate::facedown::reveal_all(self, Some(p));
        if self.players_in_game().len() <= 1 {
            return;
        }
        crate::multiplayer::remove_player_objects(self, p);
    }

    /// Ends the game if only one team (or no player) is left (CR 104.2a, 104.4a).
    pub fn check_game_over(&mut self) {
        let over = self.result.is_some();
        self.decide_game_over();
        if !over && self.result.is_some() {
            // CR 708.9: at the end of the game, face-down objects are revealed.
            crate::facedown::reveal_all(self, None);
            // CR 407.2: the winner becomes the owner of the cards in the ante.
            crate::ante::game_ended(self);
        }
    }

    /// Forces a draw (CR 104.4).
    pub fn draw_game(&mut self) {
        if self.result.is_none() {
            self.result = Some(GameResult::Draw);
        }
    }
}
