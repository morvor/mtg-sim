//! Conjure (a keyword action of digital and some tabletop cards; not in the Comprehensive
//! Rules): "conjure a card named [name] into your hand" / "into your library" / "onto the
//! battlefield". The player instructed to conjure creates a new card with that name
//! (its Oracle characteristics) and puts it into that zone. A conjured card isn't a token:
//! it's a card like any other, owned by that player, that moves between zones and lasts
//! for the rest of the game. It needn't come from the player's deck or obey deck-building
//! rules.
//!
//! The effect is an `Effect::Custom` named `"conjure:[count]:[zone]:[names]"` (see
//! [`effect_name`]); the oracle phrase is parsed in `oracle/patterns/conjure.rs`. The
//! conjured cards are "it"/"that card" for the instructions that follow.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::*;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::*;

const PREFIX: &str = "conjure:";

/// Where conjured cards are put.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConjureZone {
    Hand,
    Library,
    Graveyard,
    Battlefield,
}

impl ConjureZone {
    fn word(self) -> &'static str {
        match self {
            ConjureZone::Hand => "hand",
            ConjureZone::Library => "library",
            ConjureZone::Graveyard => "graveyard",
            ConjureZone::Battlefield => "battlefield",
        }
    }

    fn from_word(w: &str) -> Option<Self> {
        Some(match w {
            "hand" => ConjureZone::Hand,
            "library" => ConjureZone::Library,
            "graveyard" => ConjureZone::Graveyard,
            "battlefield" => ConjureZone::Battlefield,
            _ => return None,
        })
    }
}

/// How the names are used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConjureNames {
    /// `count` cards of the one name.
    Each,
    /// One card of each name ("conjure the Power Nine").
    All,
    /// `count` cards, each of a name chosen at random among them.
    Random,
}

/// The `Effect::Custom` name for conjuring: `count` cards named `names` into `zone`.
pub fn effect_name(count: u32, zone: ConjureZone, how: ConjureNames, names: &[String]) -> String {
    let how = match how {
        ConjureNames::Each => "each",
        ConjureNames::All => "all",
        ConjureNames::Random => "random",
    };
    format!("{PREFIX}{count}:{}:{how}:{}", zone.word(), names.join("|"))
}

pub struct Conjure;

impl KeywordRules for Conjure {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some(spec) = name.strip_prefix(PREFIX) else {
            return false;
        };
        let mut parts = spec.splitn(4, ':');
        let (Some(count), Some(zone), Some(how), Some(names)) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return true;
        };
        let (Ok(count), Some(zone)) = (count.parse::<u32>(), ConjureZone::from_word(zone)) else {
            return true;
        };
        let names: Vec<&str> = names.split('|').filter(|n| !n.is_empty()).collect();
        if names.is_empty() {
            return true;
        }
        let chosen: Vec<&str> = match how {
            "all" => names.clone(),
            "random" => (0..count)
                .map(|_| names[g.random_range(0, names.len() as u32 - 1) as usize])
                .collect(),
            _ => (0..count).map(|_| names[0]).collect(),
        };
        let conjured = conjure(g, ctx, &chosen, zone);
        ctx.set_var(vars::IT, conjured);
        true
    }
}

/// The player instructed to conjure (the controller of `ctx`) conjures one card of each of
/// `names` into `zone`, one at a time. Returns the conjured cards where they are.
fn conjure(g: &mut Game, ctx: &Ctx, names: &[&str], zone: ConjureZone) -> Vec<Entity> {
    let p = ctx.controller;
    let mut out = Vec::new();
    for name in names {
        let Some(def) = crate::card::CardDb::global().get(name) else {
            continue;
        };
        // A new card owned by that player (it isn't a token).
        let card = g.create_card_object(def, p, Zone::Nowhere);
        let to = match zone {
            ConjureZone::Hand => Zone::Hand(p),
            ConjureZone::Library => Zone::Library(p),
            ConjureZone::Graveyard => Zone::Graveyard(p),
            ConjureZone::Battlefield => Zone::Battlefield,
        };
        let moved = g.move_object_ev(MoveEv {
            obj: card,
            to,
            pos: LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(p),
            etb: EtbInfo {
                controller: Some(p),
                ..Default::default()
            },
            source: ctx.source,
        });
        if let Some(c) = moved {
            g.log(|g| format!("{p} conjures {}", g.describe(c)));
            out.push(Entity::Object(c));
        }
    }
    out
}

inventory::submit! { KeywordRegistration(&Conjure) }
