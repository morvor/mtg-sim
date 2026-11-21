//! CR 702.49 Ninjutsu: "Ninjutsu [cost]" means "[Cost], Reveal this card from your hand,
//! Return an unblocked attacking creature you control to its owner's hand: Put this card
//! onto the battlefield from your hand tapped and attacking." (CR 702.49a). The card
//! stays revealed while the ability is on the stack (CR 702.49b) and enters attacking what
//! the returned creature was attacking (CR 702.49c). Commander ninjutsu also functions
//! from the command zone (CR 702.49d).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::{StackKind, Zone};
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::*;
use smol_str::SmolStr;

/// The ninjutsu ability's effect.
pub const NINJUTSU: &str = "ninjutsu:put onto the battlefield attacking";

/// Whether a ninjutsu keyword is commander ninjutsu (CR 702.49d).
pub fn is_commander_ninjutsu(kw: &Keyword) -> bool {
    kw.text
        .as_deref()
        .is_some_and(|t| t.to_lowercase().starts_with("commander ninjutsu"))
}

/// Cards revealed because a ninjutsu ability of theirs is on the stack (CR 702.49b).
pub fn revealed_cards(g: &Game) -> Vec<ObjectId> {
    g.stack
        .iter()
        .filter_map(|id| match g.obj(*id).stack.as_deref().map(|s| &s.kind) {
            Some(StackKind::Activated { source, ability })
                if ability.text == KeywordKind::Ninjutsu.name()
                    && g.is_live(*source)
                    && matches!(g.obj(*source).zone, Zone::Hand(_) | Zone::Command) =>
            {
                Some(*source)
            }
            _ => None,
        })
        .collect()
}

pub struct Ninjutsu;

impl KeywordRules for Ninjutsu {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Ninjutsu]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let mut cost = kw.cost.clone().unwrap_or_default();
        cost.parts.push(CostPart::ReturnToHand {
            filter: Filter::And(vec![
                Filter::Type(CardType::Creature),
                Filter::Unblocked,
                Filter::ControlledBy(PlayerRel::You),
            ]),
            count: Value::c(1),
        });
        let zones: &[FunctionZone] = if is_commander_ninjutsu(kw) {
            &[FunctionZone::Hand, FunctionZone::Command]
        } else {
            &[FunctionZone::Hand]
        };
        Some(
            zones
                .iter()
                .map(|z| {
                    let mut act = ActivatedAbility::new(
                        cost.clone(),
                        Body::effect(Effect::Custom(SmolStr::new(NINJUTSU))),
                    );
                    act.zone = *z;
                    // Named after the keyword so "ninjutsu abilities" find it.
                    AbilityDef::new(AbilityKind::Activated(act), KeywordKind::Ninjutsu.name())
                })
                .collect(),
        )
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != NINJUTSU {
            return false;
        }
        // The card must still be where the ability was activated from.
        let Some(card) = ctx.source else {
            return true;
        };
        if !g.is_live(card) || !matches!(g.obj(card).zone, Zone::Hand(_) | Zone::Command) {
            return true;
        }
        // CR 702.49c: what the returned creature was attacking.
        let returned: Vec<ObjectId> = ctx
            .vars
            .get(&(vars::USER + 91))
            .map(|v| v.iter().filter_map(|e| e.object()).collect())
            .unwrap_or_default();
        let target = g.combat.as_ref().and_then(|c| {
            returned.iter().find_map(|r| {
                c.removed_attack_targets
                    .iter()
                    .rev()
                    .find(|(id, _)| id == r)
                    .map(|(_, t)| *t)
            })
        });
        let p = ctx.controller;
        g.move_object_ev(MoveEv {
            obj: card,
            to: Zone::Battlefield,
            pos: LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(p),
            etb: EtbInfo {
                controller: Some(p),
                tapped: true,
                attacking: target,
                ..Default::default()
            },
            source: Some(card),
        });
        true
    }
}

inventory::submit! { KeywordRegistration(&Ninjutsu) }
