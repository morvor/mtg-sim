//! CR 702.76 Prowl: "You may pay [cost] rather than pay this spell's mana cost if a player
//! was dealt combat damage this turn by a source that, at the time it dealt that damage,
//! was under your control and had any of this spell's creature types." (CR 702.76a). A
//! static ability that functions on the stack; casting a spell for its prowl cost follows
//! the rules for alternative costs (CR 601.2b, 601.2f–h).
//!
//! Combat damage dealt to players is recorded as it's dealt, with the source's controller
//! and creature types at that time (`TurnHistory::combat_damage_to_players`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{AbilityKind, Cost, Modification, StaticEffect};
use crate::casting::CastOption;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// The name recorded in `CastInfo::paid` when a spell is cast for its prowl cost.
pub const PROWL: &str = "prowl";

/// Combat damage dealt to a player: what its source was as it dealt the damage.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CombatDamageRecord {
    pub source: ObjectId,
    /// The source's controller at the time.
    pub controller: PlayerId,
    /// The source's creature types at the time.
    pub creature_types: Vec<SmolStr>,
    /// It had every creature type (changeling, CR 702.73a).
    pub every_creature_type: bool,
    /// The player dealt damage.
    pub player: PlayerId,
}

/// The creature types of an object with these characteristics, and whether it has every
/// creature type (changeling works in every zone, CR 702.73a).
fn creature_types(c: &Characteristics) -> (Vec<SmolStr>, bool) {
    let types: Vec<SmolStr> = c
        .subtypes
        .iter()
        .filter(|s| is_creature_type(s))
        .cloned()
        .collect();
    let every = crate::kw::changeling::every_creature_type(c);
    (types, every)
}

/// Whether a player was dealt combat damage this turn by a source that, at the time, was
/// under `p`'s control and had any of the creature types of `spell` (characteristics of the
/// spell as it would be cast).
pub fn prowl_condition(g: &Game, p: PlayerId, spell: &Characteristics) -> bool {
    let (types, every) = creature_types(spell);
    if types.is_empty() && !every {
        return false;
    }
    g.history.combat_damage_to_players.iter().any(|r| {
        r.controller == p
            && ((every && (r.every_creature_type || !r.creature_types.is_empty()))
                || (r.every_creature_type && !types.is_empty())
                || r.creature_types.iter().any(|t| types.contains(t)))
    })
}

/// Whether a permanent or command-zone object has a static ability giving prowl to
/// objects (a quick check before working out what a spell would have as it's cast).
fn spells_may_be_given_prowl(g: &Game) -> bool {
    let grants = |o: &GameObject| {
        o.chars.abilities.iter().any(|a| match &a.kind {
            AbilityKind::Static(s) => matches!(
                &s.effect,
                StaticEffect::Continuous { mods, .. } if mods.iter().any(|m| matches!(m, Modification::AddKeyword(k) if k.kind == KeywordKind::Prowl))
            ),
            _ => false,
        })
    };
    g.permanents().any(grants) || g.command.iter().any(|c| grants(g.obj(*c)))
}

/// Casting `card` for the prowl cost `cost` (of its own prowl ability, or one its spell
/// would have, e.g. "Dinosaur spells you cast have prowl {2}{R}"), if `p` may.
fn prowl_option(g: &Game, p: PlayerId, card: ObjectId, cost: &Cost) -> Option<CastOption> {
    let o = g.obj(card);
    let mut opt = CastOption::normal(FaceState::Front);
    opt.method = CastMethod::Keyword(KeywordKind::Prowl);
    let chars = g.option_characteristics(card, &opt);
    if o.zone != Zone::Hand(p)
        && (!g.permitted_cards(p).contains(&card) || !g.permission_allows(p, card, &chars, false))
    {
        return None;
    }
    if !prowl_condition(g, p, &chars) {
        return None;
    }
    opt.alt_cost = Some(super::modified_keyword_cost(
        g,
        p,
        KeywordKind::Prowl,
        cost,
    ));
    opt.tag = Some(PROWL);
    Some(opt)
}

pub struct Prowl;

impl KeywordRules for Prowl {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Prowl]
    }

    fn cast_options(&self, g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<CastOption> {
        kw.cost
            .as_ref()
            .and_then(|cost| prowl_option(g, p, card, cost))
            .into_iter()
            .collect()
    }

    /// Prowl that the spell would have as it's cast ("Dinosaur spells you cast have prowl
    /// {2}{R}", CR 601.3e), for a card without prowl of its own.
    fn global_cast_options(&self, g: &Game, p: PlayerId, card: ObjectId) -> Vec<CastOption> {
        let o = g.obj(card);
        if o.chars.has_keyword(KeywordKind::Prowl)
            || o.zone == Zone::Battlefield
            || !spells_may_be_given_prowl(g)
        {
            return vec![];
        }
        let as_spell = super::with_granted_spell_keywords(g, p, card, &o.chars);
        let Some(cost) = as_spell
            .keywords()
            .find(|k| k.kind == KeywordKind::Prowl)
            .and_then(|k| k.cost.clone())
        else {
            return vec![];
        };
        prowl_option(g, p, card, &cost).into_iter().collect()
    }

    fn after_damage(
        &self,
        g: &mut Game,
        source: ObjectId,
        target: Entity,
        _amount: u32,
        combat: bool,
    ) {
        let Entity::Player(player) = target else {
            return;
        };
        if !combat {
            return;
        }
        let o = g.obj(source);
        let (creature_types, every_creature_type) = creature_types(&o.chars);
        let controller = o.controller;
        g.history
            .combat_damage_to_players
            .push(CombatDamageRecord {
                source,
                controller,
                creature_types,
                every_creature_type,
                player,
            });
    }
}

inventory::submit! { KeywordRegistration(&Prowl) }
