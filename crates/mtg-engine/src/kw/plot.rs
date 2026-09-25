//! CR 702.170 Plot: "Any time you have priority during your main phase while the stack is
//! empty, you may exile this card from your hand and pay [cost]. It becomes a plotted
//! card." — a special action (CR 116.2k, 702.170b). A plotted card may be cast from exile
//! without paying its mana cost during its owner's main phase while the stack is empty,
//! on a later turn (CR 702.170d).

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

pub struct Plot;

fn plot_cost(g: &Game, card: ObjectId) -> Option<Cost> {
    g.obj(card)
        .chars
        .keywords()
        .find(|k| k.kind == KeywordKind::Plot)
        .map(|k| k.cost.clone().unwrap_or_default())
}

fn can_plot(g: &Game, p: PlayerId, card: ObjectId) -> bool {
    g.obj(card).zone == Zone::Hand(p)
        && g.turn.priority == Some(p)
        && g.is_sorcery_timing(p)
        && plot_cost(g, card).is_some()
}

/// Whether `card` is a plotted card (CR 702.170a, 702.170c), and the turn it became one.
pub fn plotted_turn(g: &Game, card: ObjectId) -> Option<u32> {
    crate::special_actions::marked(g, card, KeywordKind::Plot)
}

/// Makes a card in exile a plotted card (CR 702.170c).
pub fn make_plotted(g: &mut Game, card: ObjectId) {
    crate::special_actions::mark(g, card, KeywordKind::Plot);
}

impl KeywordRules for Plot {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Plot]
    }

    fn special_actions(&self, g: &Game, p: PlayerId) -> Vec<Action> {
        g.player(p)
            .hand
            .iter()
            .copied()
            .filter(|c| can_plot(g, p, *c))
            .filter(|c| {
                let cost = plot_cost(g, *c).unwrap_or_default();
                g.can_pay_cost(p, &cost, Some(*c), &Ctx::new(Some(*c), p))
            })
            .map(|card| Action::Special(SpecialAction::Plot { card }))
            .collect()
    }

    fn perform_special_action(
        &self,
        g: &mut Game,
        p: PlayerId,
        sa: &SpecialAction,
    ) -> Option<Result<(), Illegal>> {
        let SpecialAction::Plot { card } = sa else {
            return None;
        };
        let card = *card;
        if !g.is_live(card) || !can_plot(g, p, card) {
            return Some(Err(Illegal("can't plot that card".into())));
        }
        let cost = plot_cost(g, card).unwrap_or_default();
        if !crate::special_actions::pay(g, p, &cost, Some(card), &Ctx::new(Some(card), p)) {
            return Some(Err(Illegal("can't pay the plot cost".into())));
        }
        let new = g.move_object_ev(MoveEv {
            obj: card,
            to: Zone::Exile,
            pos: LibraryPosition::Top,
            cause: MoveCause::Exile,
            by: Some(p),
            etb: EtbInfo::default(),
            source: None,
        });
        if let Some(new) = new {
            make_plotted(g, new);
        }
        Some(Ok(()))
    }

    fn global_cast_options(&self, g: &Game, p: PlayerId, card: ObjectId) -> Vec<CastOption> {
        let o = g.obj(card);
        if o.zone != Zone::Exile || o.owner != p || o.face_down {
            return vec![];
        }
        let Some(turn) = plotted_turn(g, card) else {
            return vec![];
        };
        // A later turn, during its owner's main phase while the stack is empty.
        if turn >= g.turn.number || !g.is_sorcery_timing(p) {
            return vec![];
        }
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::Keyword(KeywordKind::Plot);
        opt.alt_cost = Some(Cost::free());
        opt.tag = Some("plot");
        vec![opt]
    }
}

inventory::submit! { KeywordRegistration(&Plot) }
