//! # mtg-engine
//!
//! A Magic: The Gathering rules engine for simulations, implementing the Comprehensive
//! Rules (see `data/comprehensive-rules.txt`) over Scryfall oracle card data.
//!
//! ## Architecture
//!
//! * [`game::Game`] holds the entire game state. Objects live in an arena indexed by
//!   [`types::ObjectId`]; every zone change creates a new object (CR 400.7) and the old one
//!   remains as last known information.
//! * [`ability`] is the data language card text compiles into; [`oracle`] is the
//!   compiler from Scryfall oracle text; [`resolve`] interprets effects.
//! * Characteristics are computed by the layer system in [`layers`] (CR 613).
//! * Events ([`events`]) drive triggered abilities ([`triggers`], CR 603); proposed events
//!   pass through replacement effects ([`replacement`], CR 614–616) before happening.
//! * [`turn`] runs the turn structure and priority loop; [`sba`] performs state-based
//!   actions; [`casting`] handles casting/activation/costs; [`combat`] handles combat.
//! * Players make choices through the [`decision::Agent`] trait.
//! * Keyword abilities are implemented in [`keyword_impls`] and the [`kw`] registry.
//! * [`testing`] provides a harness for rules tests.

pub mod ability;
pub mod actions;
pub mod agents;
pub mod ante;
pub mod apnap;
pub mod as_though;
pub mod attach;
pub mod battle;
pub mod behold;
pub mod card;
pub mod casting;
pub mod choices;
pub mod combat;
pub mod copy;
pub mod copy_rules;
pub mod cost_choices;
pub mod cost_rules;
pub mod counter_rules;
pub mod custom;
pub mod decision;
pub mod deck;
pub mod designations;
pub mod dfc;
pub mod dice;
pub mod draw_rules;
pub mod dungeons;
pub mod eval;
pub mod events;
pub mod excess_damage;
pub mod facedown;
pub mod game;
pub mod game_end;
pub mod game_terms;
pub mod keyword_actions;
pub mod keyword_actions_impl;
pub mod keyword_impls;
pub mod keywords;
pub mod kw;
pub mod layers;
pub mod library;
pub mod life_totals;
pub mod mana;
pub mod mana_abilities;
pub mod match_play;
pub mod merge;
pub mod mulligan;
pub mod multiplayer;
pub mod next_spell;
pub mod object;
pub mod opening_hand;
pub mod oracle;
pub mod oracle_ext;
pub mod piles;
pub mod planechase;
pub mod prevention;
pub mod replacement;
pub mod resolve;
pub mod restart;
pub mod reveal;
pub mod rooms;
pub mod saga;
pub mod sba;
pub mod skip;
pub mod special_actions;
pub mod splice;
pub mod stack;
pub mod start;
pub mod stickers;
pub mod target_rules;
pub mod teams;
pub mod testing;
pub mod text_change;
pub mod tokens;
pub mod tokens_predefined;
pub mod triggers;
pub mod turn;
pub mod turn_structure;
pub mod types;
pub mod untap_limits;
pub mod until;
pub mod variants;
pub mod zones;

pub use card::{card, CardDb, CardDef};
pub use decision::{Action, Agent, Answer, Decision};
pub use game::{Game, GameConfig, GameResult};
pub use types::{Entity, ObjectId, PlayerId};

/// Re-export of the data crate (CR, Scryfall) and its citation macros.
pub use mtg_data;
pub use mtg_data::{cr, ruling};
