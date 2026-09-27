//! CR 702.170 Plot: "Any time you have priority during your main phase while the stack is
//! empty, you may exile this card from your hand and pay [cost]. It becomes a plotted
//! card." — a special action (CR 116.2k, 702.170b). A plotted card may be cast from exile
//! without paying its mana cost during its owner's main phase while the stack is empty,
//! on a later turn, even if it doesn't have plot (CR 702.170d).
//!
//! * Spells and abilities may make a card in exile plotted ("It becomes plotted.",
//!   [`becomes_plotted`], CR 702.170c). Either way, the card reports a
//!   [`BECAME_PLOTTED`] event ("When this card becomes plotted").
//! * Effects that refer to plotting a card mean the special action (CR 702.170e):
//!   "Plotting cards from your hand costs {2} less." ([`from_hand_costs_less`]).
//! * An effect may let plot function in another zone (CR 702.170f): "You may plot nonland
//!   cards from the top of your library." ([`PLOT_FROM_LIBRARY_TOP`]); the card is exiled
//!   from there. "The top card of your library has plot. The plot cost is equal to its
//!   mana cost." ([`TOP_CARD_HAS_PLOT`]) gives the top card a plot cost.

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

pub struct Plot;

/// `StaticEffect::Custom`: "You may plot nonland cards from the top of your library."
/// (Fblthp, Lost on the Range; CR 702.170f).
pub const PLOT_FROM_LIBRARY_TOP: &str =
    "plot:you may plot nonland cards from the top of your library";
/// `StaticEffect::Custom`: "The top card of your library has plot. The plot cost is equal
/// to its mana cost." (Fblthp, Lost on the Range).
pub const TOP_CARD_HAS_PLOT: &str =
    "plot:the top card of your library has plot equal to its mana cost";
/// Prefix of the `StaticEffect::Custom` "Plotting cards from your hand costs {N} less."
/// (Doc Aurlock, Grizzled Genius; CR 702.170e), followed by N.
const FROM_HAND_COSTS_LESS: &str = "plot:plotting cards from your hand costs less:";
/// `Event::Custom` name: a card (`obj`, in exile) became plotted (CR 702.170a, 702.170c).
pub const BECAME_PLOTTED: &str = "became plotted";
/// `Effect::Custom`: each card in [`PLOT_VAR`] that's in exile becomes plotted.
const BECOME_PLOTTED: &str = "plot:becomes plotted";
/// The variable holding the cards that become plotted (see [`becomes_plotted`]).
const PLOT_VAR: Var = vars::USER + 1702;

/// `StaticEffect::Custom` name for "Plotting cards from your hand costs {n} less."
pub fn from_hand_costs_less(n: u32) -> SmolStr {
    SmolStr::new(format!("{FROM_HAND_COSTS_LESS}{n}"))
}

/// "It becomes plotted": each of the cards, if it's in exile (following it there, CR
/// 400.7j), becomes a plotted card (CR 702.170c).
pub fn becomes_plotted(what: Sel) -> Effect {
    Effect::ForEach {
        sel: what,
        var: PLOT_VAR,
        effect: Box::new(Effect::Custom(BECOME_PLOTTED.into())),
    }
}

/// Whether `p` controls a static ability named `name`.
fn controls_custom(g: &Game, p: PlayerId, name: &str) -> bool {
    g.statics
        .customs
        .iter()
        .any(|(_, ctl, n)| *ctl == p && n == name)
}

/// Whether `card` is the top card of `p`'s library.
fn is_library_top(g: &Game, p: PlayerId, card: ObjectId) -> bool {
    g.obj(card).zone == Zone::Library(p) && g.player(p).library.last() == Some(&card)
}

/// The plot cost of `card` for `p`: its own plot ability's, or its mana cost if it's the
/// top card of `p`'s library and an effect gives that card plot.
fn plot_cost(g: &Game, p: PlayerId, card: ObjectId) -> Option<Cost> {
    let o = g.obj(card);
    if let Some(k) = o.chars.keywords().find(|k| k.kind == KeywordKind::Plot) {
        return Some(k.cost.clone().unwrap_or_default());
    }
    if is_library_top(g, p, card) && controls_custom(g, p, TOP_CARD_HAS_PLOT) {
        return o.chars.mana_cost.clone().map(Cost::mana);
    }
    None
}

/// The cost `p` pays to plot `card`, after effects that change the cost of plotting
/// (CR 702.170e).
fn total_plot_cost(g: &Game, p: PlayerId, card: ObjectId) -> Option<Cost> {
    let mut cost = plot_cost(g, p, card)?;
    if g.obj(card).zone == Zone::Hand(p) {
        let less: u32 = g
            .statics
            .customs
            .iter()
            .filter(|(_, ctl, _)| *ctl == p)
            .filter_map(|(_, _, n)| n.strip_prefix(FROM_HAND_COSTS_LESS))
            .filter_map(|n| n.parse::<u32>().ok())
            .sum();
        if less > 0 {
            if let Some(m) = cost.mana.as_mut() {
                m.reduce_generic(less);
            }
        }
    }
    Some(cost)
}

/// Whether `p` could plot `card` now: from their hand, or from the top of their library if
/// an effect lets plot function there (CR 702.170f).
fn can_plot(g: &Game, p: PlayerId, card: ObjectId) -> bool {
    let o = g.obj(card);
    let zone_ok = o.zone == Zone::Hand(p)
        || (is_library_top(g, p, card)
            && !o.chars.is(CardType::Land)
            && controls_custom(g, p, PLOT_FROM_LIBRARY_TOP));
    zone_ok && g.has_priority(p) && g.is_sorcery_timing(p) && plot_cost(g, p, card).is_some()
}

/// Whether `card` is a plotted card (CR 702.170a, 702.170c), and the turn it became one.
pub fn plotted_turn(g: &Game, card: ObjectId) -> Option<u32> {
    crate::special_actions::marked(g, card, KeywordKind::Plot)
}

/// Makes a card in exile a plotted card (CR 702.170a, 702.170c).
pub fn make_plotted(g: &mut Game, card: ObjectId) {
    crate::special_actions::mark(g, card, KeywordKind::Plot);
    let owner = g.obj(card).owner;
    g.log(|g| format!("{} becomes plotted", g.describe(card)));
    g.emit(Event::Custom {
        name: SmolStr::new(BECAME_PLOTTED),
        player: Some(owner),
        obj: Some(card),
        amount: 0,
    });
}

impl KeywordRules for Plot {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Plot]
    }

    fn special_actions(&self, g: &Game, p: PlayerId) -> Vec<Action> {
        let mut cards: Vec<ObjectId> = g.player(p).hand.clone();
        if let Some(top) = g.player(p).library.last() {
            cards.push(*top);
        }
        cards
            .into_iter()
            .filter(|c| can_plot(g, p, *c))
            .filter(|c| {
                let cost = total_plot_cost(g, p, *c).unwrap_or_default();
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
        let cost = total_plot_cost(g, p, card).unwrap_or_default();
        if !crate::special_actions::pay(g, p, &cost, Some(card), &Ctx::new(Some(card), p)) {
            return Some(Err(Illegal("can't pay the plot cost".into())));
        }
        // Exiled from the zone it's in (its owner's hand, or where an effect lets plot
        // function, CR 702.170f).
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

    /// "It becomes plotted" (CR 702.170c).
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != BECOME_PLOTTED {
            return false;
        }
        let cards: Vec<ObjectId> = ctx
            .vars
            .get(&PLOT_VAR)
            .into_iter()
            .flatten()
            .filter_map(|e| e.object())
            .collect();
        for c in cards {
            let now = g.current(c);
            if g.is_live(now) && g.obj(now).zone == Zone::Exile {
                make_plotted(g, now);
            }
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Plot) }
