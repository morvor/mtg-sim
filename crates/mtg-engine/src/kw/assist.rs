//! CR 702.132 Assist: a static ability that modifies the rules of paying for the spell
//! with assist (CR 601.2g–h). If the spell's total cost includes a generic mana component,
//! before its caster activates mana abilities they may choose another player, who may pay
//! for any amount of the generic mana in the total cost (with mana from their own mana
//! abilities and mana pool) before the caster begins to pay the rest (CR 702.132a).
//!
//! By default a teammate is chosen (if one could pay) and pays as much as they can; an
//! opponent pays nothing unless their agent says otherwise. For the check whether a spell
//! could be cast, only teammates' mana is counted. The mana the other player pays is
//! recorded as mana spent to cast the spell, with the caster's.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::Illegal;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::mana::{ManaCost, SpendContext};
use crate::types::*;

pub struct Assist;

/// The spending context of mana spent on `spell` (by any player).
fn spend_for(g: &Game, spell: ObjectId) -> SpendContext {
    let chars = &g.obj(spell).chars;
    SpendContext {
        is_spell: true,
        is_ability: false,
        card_types: chars.card_types,
        subtypes: chars.subtypes.to_vec(),
        all_creature_types: super::changeling::every_creature_type(chars),
        has_x: chars.mana_cost.as_ref().is_some_and(|m| m.has_x()),
        source: Some(spell),
        any_color: vec![],
        check_only: false,
        class_level: false,
    }
}

impl KeywordRules for Assist {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Assist]
    }

    fn pay_mana_otherwise(
        &self,
        g: &mut Game,
        p: PlayerId,
        spell: ObjectId,
        _kw: &Keyword,
        cost: &mut Cost,
    ) -> Result<(), Illegal> {
        let generic = cost.mana.as_ref().map_or(0, |m| m.generic_amount());
        if generic == 0 {
            return Ok(());
        }
        let others: Vec<PlayerId> = g
            .player_ids()
            .into_iter()
            .filter(|q| *q != p && g.player(*q).in_game())
            .collect();
        if others.is_empty() {
            return Ok(());
        }
        let teammates = g.teammates(p);
        let default_helper = teammates
            .iter()
            .copied()
            .find(|q| g.max_mana_available(*q) > 0);
        let chosen = match g.ask(
            p,
            crate::decision::Decision::ChooseEntities {
                source: Some(spell),
                prompt: "Choose a player to assist".into(),
                candidates: others.iter().map(|q| Entity::Player(*q)).collect(),
                min: 0,
                max: 1,
            },
        ) {
            crate::decision::Answer::Entities(v) if v.len() <= 1 => v
                .first()
                .and_then(|e| e.player())
                .filter(|q| others.contains(q)),
            _ => default_helper,
        };
        let Some(helper) = chosen else {
            return Ok(());
        };
        let can = generic.min(g.max_mana_available(helper));
        let default = if teammates.contains(&helper) { can } else { 0 };
        let n = match g.ask(
            helper,
            crate::decision::Decision::ChooseNumber {
                source: Some(spell),
                prompt: format!("Assist: pay how much of the {generic} generic mana?"),
                min: 0,
                max: can as i64,
            },
        ) {
            crate::decision::Answer::Number(n) if (0..=can as i64).contains(&n) => n as u32,
            _ => default,
        };
        if n == 0 {
            return Ok(());
        }
        let snapshot = g.clone();
        let spend = spend_for(g, spell);
        let ctx = Ctx::new(Some(spell), helper);
        match g.pay_total_cost(
            helper,
            &Cost::mana(ManaCost::generic(n)),
            Some(spell),
            &spend,
            &ctx,
        ) {
            Ok(paid) => {
                if let Some(m) = cost.mana.as_mut() {
                    m.reduce_generic(n);
                }
                // That mana was spent to cast the spell too (e.g. "the amount of mana spent
                // to cast this spell").
                if let Some(si) = g.objects[spell.0 as usize].stack.as_mut() {
                    si.cast.mana_spent.extend(paid.mana_spent.iter().cloned());
                }
                g.log(|g| format!("{helper} assists with {n} mana for {}", g.describe(spell)));
            }
            Err(_) => {
                // The helper couldn't pay after all: nothing was paid.
                g.roll_back(snapshot);
            }
        }
        Ok(())
    }

    fn payable_otherwise(
        &self,
        g: &Game,
        p: PlayerId,
        _card: ObjectId,
        _kw: &Keyword,
        _method: &crate::object::CastMethod,
        cost: &mut Cost,
    ) {
        let help = g
            .teammates(p)
            .into_iter()
            .map(|q| g.max_mana_available(q))
            .max()
            .unwrap_or(0);
        if let Some(m) = cost.mana.as_mut() {
            m.reduce_generic(help);
        }
    }
}

inventory::submit! { KeywordRegistration(&Assist) }
