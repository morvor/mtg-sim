//! CR 702.65 Aura swap. "Aura swap [cost]" means "[Cost]: You may exchange this
//! permanent with an Aura card in your hand." (CR 702.65a). If either half of the
//! exchange can't be completed, the ability has no effect (CR 702.65b, 701.12a): cards can
//! be exchanged between zones only if they're owned by the same player (CR 701.12d), so
//! the permanent must be one you own, and the Aura card from your hand becomes attached
//! to what the permanent was attached to (CR 701.12e), so it must be able to enchant it.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::Zone;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::*;
use smol_str::SmolStr;

/// `Effect::Custom`: "you may exchange this permanent with an Aura card in your hand".
const EXCHANGE: &str = "aura swap:exchange this permanent with an Aura card in your hand";

pub struct AuraSwap;

impl KeywordRules for AuraSwap {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::AuraSwap]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let act = ActivatedAbility::new(
            kw.cost.clone().unwrap_or_default(),
            Body::effect(Effect::Custom(SmolStr::new(EXCHANGE))),
        );
        Some(vec![AbilityDef::new(
            AbilityKind::Activated(act),
            KeywordKind::AuraSwap.name(),
        )])
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != EXCHANGE {
            return false;
        }
        exchange(g, ctx);
        true
    }
}

fn exchange(g: &mut Game, ctx: &mut Ctx) {
    let p = ctx.controller;
    let Some(src) = ctx.source else {
        return;
    };
    // This permanent: still on the battlefield, owned by you, attached to something.
    if !g.is_live(src) || g.obj(src).zone != Zone::Battlefield || g.obj(src).owner != p {
        return;
    }
    let Some(host) = g.obj(src).attached_to else {
        return;
    };
    // An Aura card in your hand that could enchant it.
    let candidates: Vec<ObjectId> = g
        .player(p)
        .hand
        .iter()
        .copied()
        .filter(|c| {
            let o = g.obj(*c);
            o.owner == p
                && o.chars.is(CardType::Enchantment)
                && o.chars.has_subtype("Aura")
                && crate::attach::can_attach(g, *c, host)
        })
        .collect();
    if candidates.is_empty() {
        return;
    }
    if !g.ask_yes_no(p, Some(src), "Exchange this Aura with an Aura card in your hand?", true) {
        return;
    }
    let chosen = g.ask_objects(
        p,
        Some(src),
        "Choose an Aura card to exchange",
        candidates.clone(),
        1,
        1,
    );
    let Some(card) = chosen.first().copied().filter(|c| candidates.contains(c)) else {
        return;
    };
    // The exchange is simultaneous (Arcanum Wings ruling).
    g.move_objects(vec![
        MoveEv {
            obj: src,
            to: Zone::Hand(p),
            pos: LibraryPosition::Top,
            cause: MoveCause::Return,
            by: Some(p),
            etb: EtbInfo::default(),
            source: Some(src),
        },
        MoveEv {
            obj: card,
            to: Zone::Battlefield,
            pos: LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(p),
            etb: EtbInfo {
                controller: Some(p),
                attach_to: Some(host),
                ..Default::default()
            },
            source: Some(src),
        },
    ]);
}

inventory::submit! { KeywordRegistration(&AuraSwap) }
