//! CR 702.144 Demonstrate: a triggered ability. "Demonstrate" means "When you cast this
//! spell, you may copy it and you may choose new targets for the copy. If you copy the
//! spell, choose an opponent. That player copies the spell and may choose new targets for
//! that copy." (CR 702.144a).
//!
//! The ability functions on the stack. If the spell has left the stack by the time the
//! ability resolves, it's copied as it last existed there (CR 608.2h). A copy of a
//! permanent spell becomes a token (CR 707.10f).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;
use smol_str::SmolStr;

/// `Effect::Custom`: the demonstrate ability's effect.
pub const DEMONSTRATE: &str = "demonstrate:copy it, then an opponent copies it";

pub struct Demonstrate;

impl KeywordRules for Demonstrate {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Demonstrate]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let mut t = TriggeredAbility::new(
            TriggerCond::CastSpell {
                who: PlayerRel::You,
                filter: Filter::Source,
            },
            Body::effect(Effect::Custom(SmolStr::new(DEMONSTRATE))),
        );
        t.zone = FunctionZone::Stack;
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(t),
            KeywordKind::Demonstrate.name(),
        )])
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != DEMONSTRATE {
            return false;
        }
        let Some(spell) = ctx.source else {
            return true;
        };
        let you = ctx.controller;
        let spell_name = g.obj(spell).chars.name.clone();
        if !g.ask_yes_no(
            you,
            Some(spell),
            &format!("Demonstrate: copy {spell_name}?"),
            false,
        ) {
            return true;
        }
        // "If you copy the spell, choose an opponent. That player copies the spell ..."
        if crate::copy::copy_spell(g, spell, you, true).is_none() {
            return true;
        }
        let opponents: Vec<Entity> = g
            .opponents(you)
            .into_iter()
            .filter(|o| crate::multiplayer::range::player_in_range(g, you, *o))
            .map(Entity::Player)
            .collect();
        let chosen = g.ask_entities(
            you,
            Some(spell),
            "Demonstrate: choose an opponent to copy the spell",
            opponents,
            1,
            1,
        );
        if let Some(Entity::Player(opp)) = chosen.first().copied() {
            g.log(|g| format!("{opp} copies {}", g.describe(spell)));
            crate::copy::copy_spell(g, spell, opp, true);
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Demonstrate) }
