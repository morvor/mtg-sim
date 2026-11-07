//! CR 702.185 Warp.
//!
//! * "Warp [cost]" means "You may cast this card from your hand by paying [cost] rather
//!   than its mana cost" and "If this spell's warp cost was paid, exile the permanent this
//!   spell becomes at the beginning of the next end step. Its owner may cast this card
//!   after the current turn has ended for as long as it remains exiled." (CR 702.185a).
//!   The warp cost is an alternative cost (CR 601.2b, 601.2f–h): timing restrictions
//!   apply. The delayed triggered ability is created as the permanent enters
//!   (CR 608.3g) and exiles only that permanent, if it's still on the battlefield.
//! * A card exiled by that delayed triggered ability is a "warped" card in exile
//!   (CR 702.185b), marked with the turn it was exiled ([`is_warped`]): its owner may cast
//!   it (paying its costs) on a later turn. A card with warp exiled some other way isn't.
//! * "A spell was warped this turn": a spell was cast for its warp cost this turn
//!   (CR 702.185c), whether or not it resolved ([`WARPED_THIS_TURN`]). The void ability
//!   word's condition "if a nonland permanent left the battlefield this turn or a spell
//!   was warped this turn" is parsed in `oracle/patterns/k702_179_195.rs`.
//! * "You may cast this card from your graveyard using its warp ability." lets it be cast
//!   for its warp cost from its owner's graveyard too ([`FROM_GRAVEYARD`]).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::CastOption;
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::*;

/// The name recorded in `CastInfo::paid` when a spell is cast for its warp cost.
pub const WARP: &str = "warp";
/// `Effect::Custom`: the warp delayed triggered ability's effect, exiling the permanent
/// (`vars::IT`) and making it a warped card.
const EXILE_WARPED: &str = "warp:exile the permanent";
/// `Condition::Custom`: "a spell was warped this turn" (CR 702.185c).
pub const WARPED_THIS_TURN: &str = "warp:a spell was warped this turn";
/// `Condition::Custom`: "a nonland permanent left the battlefield this turn" (as the
/// permanents that left last existed on the battlefield).
pub const NONLAND_LEFT_THIS_TURN: &str = "warp:a nonland permanent left the battlefield this turn";
/// `Filter::Custom`: a warped card in exile (CR 702.185b).
pub const WARPED_CARD: &str = "warp:warped card in exile";
/// `StaticEffect::Custom` of a card that functions in its owner's graveyard: "You may cast
/// this card from your graveyard using its warp ability."
pub const FROM_GRAVEYARD: &str = "warp:cast from graveyard using warp";

/// Whether the spell was cast for its warp cost.
pub fn warped(g: &Game, spell: ObjectId) -> bool {
    g.obj(spell)
        .stack
        .as_deref()
        .is_some_and(|s| s.cast.paid.iter().any(|x| x == WARP))
}

/// Whether `card` is a warped card in exile (CR 702.185b), and the turn it was exiled.
pub fn is_warped(g: &Game, card: ObjectId) -> Option<u32> {
    if g.obj(card).zone != Zone::Exile {
        return None;
    }
    crate::special_actions::marked(g, card, KeywordKind::Warp)
}

fn casts_from_graveyard_with_warp(o: &GameObject) -> bool {
    o.chars.abilities.iter().any(|a| {
        matches!(&a.kind, AbilityKind::Static(s)
            if matches!(&s.effect, StaticEffect::Custom(n) if n == FROM_GRAVEYARD))
    })
}

pub struct Warp;

impl KeywordRules for Warp {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Warp]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let mut exile = StaticAbility::new(StaticEffect::DelayedTriggerAsEnters {
            condition: Some(Condition::CostPaid(WARP.into())),
            trigger: TriggerCond::BeginningOf {
                step: TriggerStep::End,
                whose: PlayerRel::Any,
            },
            body: Body::effect(Effect::Custom(EXILE_WARPED.into())),
        });
        exile.zone = FunctionZone::Stack;
        Some(vec![AbilityDef::new(
            AbilityKind::Static(exile),
            KeywordKind::Warp.name(),
        )])
    }

    /// "You may cast this card from your hand by paying [cost] rather than its mana cost."
    fn cast_options(&self, g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<CastOption> {
        let o = g.obj(card);
        let Some(cost) = kw.cost.clone() else {
            return vec![];
        };
        let from_graveyard = crate::as_though::in_graveyard_for(g, p, card)
            && o.owner == p
            && casts_from_graveyard_with_warp(o);
        if o.zone != Zone::Hand(p) && !from_graveyard {
            return vec![];
        }
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::Keyword(KeywordKind::Warp);
        opt.alt_cost = Some(super::modified_keyword_cost(
            g,
            p,
            KeywordKind::Warp,
            &cost,
        ));
        opt.tag = Some(WARP);
        vec![opt]
    }

    /// "Its owner may cast this card after the current turn has ended for as long as it
    /// remains exiled."
    fn global_cast_options(&self, g: &Game, p: PlayerId, card: ObjectId) -> Vec<CastOption> {
        let o = g.obj(card);
        if o.owner != p || o.face_down {
            return vec![];
        }
        match is_warped(g, card) {
            Some(turn) if turn < g.turn.number => vec![CastOption::normal(FaceState::Front)],
            _ => vec![],
        }
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != EXILE_WARPED {
            return false;
        }
        let perms: Vec<ObjectId> = ctx
            .vars
            .get(&vars::IT)
            .into_iter()
            .flatten()
            .filter_map(|e| e.object())
            .filter(|o| g.is_live(*o) && g.obj(*o).zone == Zone::Battlefield)
            .collect();
        for perm in perms {
            let new = g.move_object_ev(MoveEv {
                obj: perm,
                to: Zone::Exile,
                pos: LibraryPosition::Top,
                cause: MoveCause::Exile,
                by: Some(ctx.controller),
                etb: EtbInfo::default(),
                source: ctx.source,
            });
            if let Some(new) = new.filter(|n| g.obj(*n).zone == Zone::Exile) {
                crate::special_actions::mark(g, new, KeywordKind::Warp);
            }
        }
        true
    }

    fn custom_condition(&self, g: &Game, name: &str, _ctx: &Ctx) -> Option<bool> {
        match name {
            WARPED_THIS_TURN => Some(
                g.history
                    .spells_cast
                    .iter()
                    .any(|(_, s)| warped(g, *s)),
            ),
            NONLAND_LEFT_THIS_TURN => Some(
                g.history
                    .permanents_left
                    .iter()
                    .any(|o| !g.obj(*o).chars.is_land()),
            ),
            _ => None,
        }
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        (name == WARPED_CARD).then(|| is_warped(g, id).is_some())
    }
}

inventory::submit! { KeywordRegistration(&Warp) }
