//! CR 702.94 Miracle: "You may reveal this card from your hand as you draw it if it's the
//! first card you've drawn this turn. When you reveal this card this way, you may cast it
//! by paying [cost] rather than its mana cost." The player looks at the card as they draw
//! it before choosing whether to reveal it (CR 121.9).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::CastOption;
use crate::eval::Ctx;
use crate::game::{Game, PendingTrigger};
use crate::keywords::KeywordKind;
use crate::object::*;
use crate::types::*;

pub struct Miracle;

/// The custom effect of the linked triggered ability.
pub const CAST_MIRACLE: &str = "miracle_cast";

fn miracle_cost(g: &Game, card: ObjectId) -> Option<Cost> {
    g.obj(card)
        .chars
        .keywords()
        .find(|k| k.kind == KeywordKind::Miracle)
        .map(|k| k.cost.clone().unwrap_or_default())
}

impl KeywordRules for Miracle {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Miracle]
    }

    fn after_draw(&self, g: &mut Game, p: PlayerId, card: ObjectId, nth: u32) {
        if nth != 1 || g.obj(card).zone != Zone::Hand(p) || miracle_cost(g, card).is_none() {
            return;
        }
        // CR 121.9: the card is already in the player's hand, so they can look at it
        // before choosing whether to reveal it.
        let name = g.obj(card).chars.name.clone();
        if !g.ask_yes_no(p, Some(card), &format!("Reveal {name} (miracle)?"), true) {
            return;
        }
        // CR 702.94b: they play with it revealed until it leaves their hand or the linked
        // triggered ability resolves or otherwise leaves the stack (CR 701.20a: a card
        // whose revealing made an ability trigger stays revealed until then).
        crate::reveal::reveal(g, p, &[card], None);
        // CR 701.20a: it stays revealed until the triggered ability leaves the stack.
        crate::reveal::reveal(g, p, &[card], None);
        // "When you reveal this card this way, ...": it triggers now and is put on the
        // stack the next time a player would receive priority.
        g.trigger_order += 1;
        let body = Body::effect(Effect::Custom(CAST_MIRACLE.into()));
        let ability = AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::Custom("miracle".into()),
                body.clone(),
            )),
            "Miracle",
        );
        let lki = Box::new(g.obj(card).chars.clone());
        let order = g.trigger_order;
        g.pending_triggers.push(PendingTrigger {
            source: card,
            controller: p,
            ability,
            event: EventInfo {
                object: Some(card),
                player: Some(p),
                ..Default::default()
            },
            source_lki: Some(lki),
            saved: None,
            body: Some(body),
            order,
        });
    }
}

/// The miracle trigger resolving: its controller may cast the card from their hand by
/// paying its miracle cost rather than its mana cost (CR 702.94a).
pub fn cast_miracle(g: &mut Game, ctx: &mut Ctx) {
    let Some(card) = ctx.source else {
        return;
    };
    let p = ctx.controller;
    if !g.is_live(card) || g.obj(card).zone != Zone::Hand(p) {
        return;
    }
    let Some(cost) = miracle_cost(g, card) else {
        return;
    };
    if !g.ask_yes_no(p, Some(card), "Cast it for its miracle cost?", true) {
        return;
    }
    let mut opt = CastOption::normal(FaceState::Front);
    opt.method = CastMethod::Keyword(KeywordKind::Miracle);
    opt.alt_cost = Some(cost);
    opt.any_time = true;
    opt.tag = Some("miracle");
    let _ = g.cast_with_option(p, card, opt);
}

inventory::submit! { KeywordRegistration(&Miracle) }
