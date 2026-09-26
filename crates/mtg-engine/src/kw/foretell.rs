//! CR 702.143 Foretell: "Any time a player has priority during their turn, that player
//! may pay {2} and exile a card with foretell from their hand face down" — a special
//! action (CR 116.2h, 702.143b). The card may be cast after the current turn has ended by
//! paying its foretell cost rather than its mana cost (CR 702.143a).
//!
//! Foretelling a card means taking that special action (CR 702.143c): it's reported with
//! the [`FORETOLD`] event ("Whenever you foretell a card"). A foretold card is one put into
//! exile by it, or one an effect says becomes foretold (CR 702.143d), possibly giving it a
//! foretell cost of its own; each is marked with the turn it became foretold and that
//! cost, in the order they were exiled (CR 702.143e). A spell that was a foretold card
//! before it was cast "was foretold", whatever cost it was cast for ([`WAS_FORETOLD`]).
//! Face-down foretold cards are revealed when their owner leaves the game and at the end
//! of the game (CR 702.143f, see `facedown::reveal_all`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::{CastOption, Illegal};
use crate::decision::{Action, SpecialAction};
use crate::eval::Ctx;
use crate::events::{Event, MoveCause};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::*;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::*;
use smol_str::SmolStr;

/// `Event::Custom` name: a player (`player`) foretold a card (`obj`, in exile).
pub const FORETOLD: &str = "foretold";
/// `TriggerCond::Custom`: "Whenever you foretell a card".
pub const YOU_FORETELL: &str = "foretell:you foretell a card";
/// `Condition::Custom`: "if this spell was foretold" (CR 702.143c).
pub const WAS_FORETOLD: &str = "foretell:this spell was foretold";
/// `Effect::Custom`: "[the exiled cards (`vars::IT`)] become foretold" (CR 702.143d).
pub const BECOMES_FORETOLD: &str = "foretell:it becomes foretold";
/// `Effect::Custom` prefix, followed by N: "It becomes foretold. Its foretell cost is its
/// mana cost reduced by {N}."
pub const BECOMES_FORETOLD_REDUCED: &str = "foretell:it becomes foretold with its mana cost reduced by ";

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
        && g.has_priority(p)
}

/// The foretell cost printed on a card (CR 702.143a).
fn foretell_cost(g: &Game, card: ObjectId) -> Option<Cost> {
    let d = g.obj(card).card.as_ref()?;
    d.characteristics(FaceState::Front)
        .keywords()
        .find(|k| k.kind == KeywordKind::Foretell)
        .and_then(|k| k.cost.clone())
}

/// The foretold marks of `obj` (the most recent first), whether or not it's still the same
/// object.
fn foretold_marks(g: &Game, obj: ObjectId) -> impl Iterator<Item = &crate::special_actions::KeywordMark> {
    g.special
        .marks
        .iter()
        .rev()
        .filter(move |m| m.obj == obj && m.kind == KeywordKind::Foretell)
}

/// Whether `card` is a foretold card: in exile, marked foretold as it is now.
pub fn is_foretold(g: &Game, card: ObjectId) -> bool {
    g.is_live(card) && g.obj(card).zone == Zone::Exile && foretold_marks(g, card).next().is_some()
}

/// The face-down foretold cards in exile (owned by `owner`, if given), in the order they
/// were exiled.
pub fn face_down_foretold(g: &Game, owner: Option<PlayerId>) -> Vec<ObjectId> {
    g.exile
        .iter()
        .copied()
        .filter(|c| g.obj(*c).face_down && is_foretold(g, *c))
        .filter(|c| owner.is_none_or(|p| g.obj(*c).owner == p))
        .collect()
}

/// Marks the cards as foretold, giving them `cost` as a foretell cost (CR 702.143d). Their
/// owner may look at them as long as they remain in exile.
fn become_foretold(g: &mut Game, cards: &[ObjectId], reduce_by: Option<u32>) {
    for c in cards.iter().copied() {
        if !g.is_live(c) || g.obj(c).zone != Zone::Exile {
            continue;
        }
        let cost = reduce_by.map(|n| {
            let mut m = g
                .obj(c)
                .card
                .as_ref()
                .and_then(|d| d.characteristics(FaceState::Front).mana_cost)
                .unwrap_or_default();
            m.reduce_generic(n);
            Cost::mana(m)
        });
        crate::special_actions::mark_with_cost(g, c, KeywordKind::Foretell, cost);
        let owner = g.obj(c).owner;
        crate::zones::allow_look(g, owner, c);
    }
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
            // CR 702.143a, 406.3: the player may look at it as long as it remains exiled.
            crate::zones::allow_look(g, p, new);
            // CR 702.143c: the player foretold a card.
            g.emit(Event::Custom {
                name: SmolStr::new(FORETOLD),
                player: Some(p),
                obj: Some(new),
                amount: 0,
            });
        }
        Some(Ok(()))
    }

    fn global_cast_options(&self, g: &Game, p: PlayerId, card: ObjectId) -> Vec<CastOption> {
        let o = g.obj(card);
        if o.zone != Zone::Exile || o.owner != p || !o.face_down {
            return vec![];
        }
        // After the turn it became foretold has ended.
        let Some(mark) = foretold_marks(g, card).next() else {
            return vec![];
        };
        if mark.turn >= g.turn.number || !g.is_live(card) {
            return vec![];
        }
        // Any foretell cost it has: its printed one, and one the effect that made it
        // foretold gave it (CR 702.143d).
        let costs: Vec<Cost> = foretell_cost(g, card)
            .into_iter()
            .chain(mark.cost.clone())
            .collect();
        costs
            .into_iter()
            .map(|cost| {
                let mut opt = CastOption::normal(FaceState::Front);
                opt.method = CastMethod::Keyword(KeywordKind::Foretell);
                opt.alt_cost = Some(cost);
                opt.tag = Some("foretell");
                opt
            })
            .collect()
    }

    fn custom_trigger(
        &self,
        _g: &Game,
        name: &str,
        _src: ObjectId,
        ctl: PlayerId,
        ev: &Event,
    ) -> Option<Vec<EventInfo>> {
        if name != YOU_FORETELL {
            return None;
        }
        Some(match ev {
            Event::Custom {
                name: n,
                player: Some(p),
                obj,
                ..
            } if n == FORETOLD && *p == ctl => vec![EventInfo {
                object: *obj,
                player: Some(*p),
                ..Default::default()
            }],
            _ => vec![],
        })
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != WAS_FORETOLD {
            return None;
        }
        // The spell was a foretold card before it was cast (CR 702.143c).
        let spell = ctx.source?;
        let o = g.obj(spell);
        let was = o.kind == ObjKind::Card
            && o.stack.as_deref().is_some_and(|si| si.cast.was_cast)
            && o.prev.is_some_and(|prev| {
                g.obj(prev).zone == Zone::Exile && foretold_marks(g, prev).next().is_some()
            });
        Some(was)
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let reduce_by = if name == BECOMES_FORETOLD {
            None
        } else if let Some(n) = name.strip_prefix(BECOMES_FORETOLD_REDUCED) {
            match n.parse::<u32>() {
                Ok(n) => Some(n),
                Err(_) => return false,
            }
        } else {
            return false;
        };
        let cards: Vec<ObjectId> = ctx
            .vars
            .get(&vars::IT)
            .map(|v| v.iter().filter_map(|e| e.object()).collect())
            .unwrap_or_default();
        become_foretold(g, &cards, reduce_by);
        true
    }
}

inventory::submit! { KeywordRegistration(&Foretell) }
