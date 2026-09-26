//! CR 702.139 Companion. Before the game begins, a player may reveal one card they own
//! from outside the game with a companion ability whose condition their starting deck
//! fulfills (CR 103.2b). Once during the game, any time they have priority and the stack
//! is empty, but only during a main phase of their turn, they may pay {3} and put that
//! card into their hand — a special action (CR 116.2g, 702.139a).
//!
//! The companion conditions themselves (deck-building restrictions) are given by each
//! card's text; [`choose_companion`] records the choice.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::Illegal;
use crate::decision::{Action, SpecialAction};
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::*;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::*;

pub struct Companion;

/// `Filter::Custom`: a card with an even mana value ("only cards with even mana values").
pub const EVEN_MANA_VALUE: &str = "companion:even mana value";
/// `Filter::Custom`: a card with an odd mana value.
pub const ODD_MANA_VALUE: &str = "companion:odd mana value";
/// `Filter::Custom`: a card with an activated ability, including the activated abilities
/// its keywords stand for (e.g. cycling) and a basic land type's intrinsic mana ability.
pub const HAS_ACTIVATED_ABILITY: &str = "companion:has an activated ability";

fn has_activated_ability(g: &Game, id: ObjectId) -> bool {
    let chars = &g.obj(id).chars;
    // CR 305.6: a land with a basic land type has its intrinsic mana ability.
    if chars.is_land()
        && chars
            .subtypes
            .iter()
            .any(|s| crate::layers::intrinsic_mana_ability(s).is_some())
    {
        return true;
    }
    chars.abilities.iter().any(|a| match &a.kind {
        AbilityKind::Activated(_) => true,
        AbilityKind::Keyword(k) => crate::keyword_impls::derived_abilities(k)
            .iter()
            .any(|d| matches!(d.kind, AbilityKind::Activated(_))),
        _ => false,
    })
}

fn companion_cost() -> Cost {
    Cost::mana(crate::mana::ManaCost::generic(3))
}

/// Records `card` (outside the game, with companion) as `p`'s companion (CR 103.2b). A
/// player may reveal no more than one. Returns true if it was chosen.
pub fn choose_companion(g: &mut Game, p: PlayerId, card: ObjectId) -> bool {
    let o = g.obj(card);
    if o.zone != Zone::Outside(p)
        || o.owner != p
        || !o.chars.has_keyword(KeywordKind::Companion)
        || g.special.companions.iter().any(|(q, _, _)| *q == p)
    {
        return false;
    }
    g.special.companions.push((p, card, false));
    g.log(|g| format!("{p} reveals {} as their companion", g.obj(card).chars.name));
    true
}

/// `p`'s companion, if they chose one and haven't put it into their hand.
fn unused_companion(g: &Game, p: PlayerId) -> Option<ObjectId> {
    g.special
        .companions
        .iter()
        .find(|(q, c, used)| {
            *q == p && !*used && g.is_live(*c) && g.obj(*c).zone == Zone::Outside(p)
        })
        .map(|(_, c, _)| *c)
}

impl KeywordRules for Companion {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Companion]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        match name {
            EVEN_MANA_VALUE => Some(g.mana_value_of(id) % 2 == 0),
            ODD_MANA_VALUE => Some(g.mana_value_of(id) % 2 == 1),
            HAS_ACTIVATED_ABILITY => Some(has_activated_ability(g, id)),
            _ => None,
        }
    }

    fn special_actions(&self, g: &Game, p: PlayerId) -> Vec<Action> {
        let Some(card) = unused_companion(g, p) else {
            return vec![];
        };
        // CR 903.11a: in a Commander game, not a card the rules don't let in.
        if !crate::commander_rules::may_bring_in(g, card, Zone::Hand(p)) {
            return vec![];
        }
        if !g.has_priority(p)
            || !g.is_sorcery_timing(p)
            || !g.can_pay_cost(p, &companion_cost(), Some(card), &Ctx::new(Some(card), p))
        {
            return vec![];
        }
        vec![Action::Special(SpecialAction::CompanionToHand { card })]
    }

    fn perform_special_action(
        &self,
        g: &mut Game,
        p: PlayerId,
        sa: &SpecialAction,
    ) -> Option<Result<(), Illegal>> {
        let SpecialAction::CompanionToHand { card } = sa else {
            return None;
        };
        if unused_companion(g, p) != Some(*card) || !g.has_priority(p) || !g.is_sorcery_timing(p) {
            return Some(Err(Illegal(
                "can't put that companion into hand now".into(),
            )));
        }
        if !crate::special_actions::pay(
            g,
            p,
            &companion_cost(),
            Some(*card),
            &Ctx::new(Some(*card), p),
        ) {
            return Some(Err(Illegal("can't pay {3}".into())));
        }
        for c in g.special.companions.iter_mut() {
            if c.0 == p {
                c.2 = true;
            }
        }
        g.move_object_ev(MoveEv {
            obj: *card,
            to: Zone::Hand(p),
            pos: LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(p),
            etb: EtbInfo::default(),
            source: None,
        });
        Some(Ok(()))
    }
}

inventory::submit! { KeywordRegistration(&Companion) }
