//! CR 702.143 Foretell: "Any time a player has priority during their turn, that player
//! may pay {2} and exile a card with foretell from their hand face down" — a special
//! action (CR 116.2h, 702.143b). The card may be cast after the current turn has ended by
//! paying its foretell cost rather than its mana cost (CR 702.143a).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::{CastOption, Illegal};
use crate::decision::{Action, SpecialAction};
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::*;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::*;

pub struct Foretell;

/// The cost of foretelling a card (CR 702.143a).
fn foretell_action_cost() -> Cost {
    Cost::mana(crate::mana::ManaCost::parse("{2}").expect("mana"))
}

fn can_foretell(g: &Game, p: PlayerId, card: ObjectId) -> bool {
    let o = g.obj(card);
    o.zone == Zone::Hand(p)
        && o.chars.has_keyword(KeywordKind::Foretell)
        && g.turn.active == p
        && g.turn.priority == Some(p)
}

/// The foretell cost printed on a card (CR 702.143a).
fn foretell_cost(g: &Game, card: ObjectId) -> Option<Cost> {
    let d = g.obj(card).card.as_ref()?;
    d.characteristics(FaceState::Front)
        .keywords()
        .find(|k| k.kind == KeywordKind::Foretell)
        .and_then(|k| k.cost.clone())
}

impl KeywordRules for Foretell {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Foretell]
    }

    fn special_actions(&self, g: &Game, p: PlayerId) -> Vec<Action> {
        let cost = foretell_action_cost();
        g.player(p)
            .hand
            .iter()
            .copied()
            .filter(|c| can_foretell(g, p, *c))
            .filter(|c| g.can_pay_cost(p, &cost, Some(*c), &Ctx::new(Some(*c), p)))
            .map(|card| Action::Special(SpecialAction::Foretell { card }))
            .collect()
    }

    fn perform_special_action(
        &self,
        g: &mut Game,
        p: PlayerId,
        sa: &SpecialAction,
    ) -> Option<Result<(), Illegal>> {
        let SpecialAction::Foretell { card } = sa else {
            return None;
        };
        let card = *card;
        if !g.is_live(card) || !can_foretell(g, p, card) {
            return Some(Err(Illegal("can't foretell that card".into())));
        }
        if !crate::special_actions::pay(
            g,
            p,
            &foretell_action_cost(),
            Some(card),
            &Ctx::new(Some(card), p),
        ) {
            return Some(Err(Illegal("can't pay {2}".into())));
        }
        let new = g.move_object_ev(MoveEv {
            obj: card,
            to: Zone::Exile,
            pos: LibraryPosition::Top,
            cause: MoveCause::Exile,
            by: Some(p),
            etb: EtbInfo {
                face_down: Some(KeywordKind::Foretell),
                ..Default::default()
            },
            source: None,
        });
        if let Some(new) = new {
            crate::special_actions::mark(g, new, KeywordKind::Foretell);
        }
        Some(Ok(()))
    }

    fn global_cast_options(&self, g: &Game, p: PlayerId, card: ObjectId) -> Vec<CastOption> {
        let o = g.obj(card);
        if o.zone != Zone::Exile || o.owner != p || !o.face_down {
            return vec![];
        }
        // After the turn it was foretold has ended.
        let Some(turn) = crate::special_actions::marked(g, card, KeywordKind::Foretell) else {
            return vec![];
        };
        if turn >= g.turn.number {
            return vec![];
        }
        let Some(cost) = foretell_cost(g, card) else {
            return vec![];
        };
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::Keyword(KeywordKind::Foretell);
        opt.alt_cost = Some(cost);
        opt.tag = Some("foretell");
        vec![opt]
    }
}

inventory::submit! { KeywordRegistration(&Foretell) }
