//! "Whenever you reveal a card ... this way" written after an ability that reveals cards
//! (Priority Boarding: "Whenever you roll a die, you may reveal the top card of your
//! library. Do this only once each turn. Whenever you reveal a card with mana value less
//! than the result this way, ..."): a triggered ability of the permanent, linked to the
//! revealing ability (CR 607.1, 607.2). It triggers on a card its controller reveals
//! (CR 603.2, 701.20a) while that ability — the one on the same object with the same link,
//! or a copy of it, which has the same source (CR 707.10b) — is resolving; a reveal by any
//! other ability isn't "this way". Like any triggered ability of a permanent, it triggers
//! only if the permanent has it as the card is revealed (CR 603.2, 603.6): not if the
//! permanent has left the battlefield or lost its abilities by then.
//!
//! The trigger event carries the revealing ability's own trigger amount ("the result": the
//! die roll that triggered it, `dice::THE_RESULT`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::{EventInfo, StackKind};
use crate::types::*;

/// `TriggerCond::Custom`: "Whenever you reveal a card this way" — revealed by `src`'s
/// ability linked to the triggered ability with this condition.
pub const REVEALED_THIS_WAY: &str = "reveal:you reveal a card this way";

/// Whether `t` is (or wraps) the trigger condition [`REVEALED_THIS_WAY`].
fn is_this_way(t: &TriggerCond) -> bool {
    match t {
        TriggerCond::Custom(n) => n == REVEALED_THIS_WAY,
        TriggerCond::Where { trigger, .. } => is_this_way(trigger),
        _ => false,
    }
}

struct RevealedThisWay;

impl KeywordRules for RevealedThisWay {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_trigger(
        &self,
        g: &Game,
        name: &str,
        src: ObjectId,
        ctl: PlayerId,
        ev: &Event,
    ) -> Option<Vec<EventInfo>> {
        if name != REVEALED_THIS_WAY {
            return None;
        }
        let Event::Custom {
            name: ev_name,
            player: Some(p),
            obj: Some(card),
            ..
        } = ev
        else {
            return Some(vec![]);
        };
        if ev_name != crate::reveal::REVEALED || *p != ctl {
            return Some(vec![]);
        }
        // The spell or ability that revealed it.
        let Some(stack_obj) = g
            .reveals
            .revealed
            .iter()
            .rev()
            .find(|r| r.card == *card && r.by == *p)
            .and_then(|r| r.stack_obj)
        else {
            return Some(vec![]);
        };
        let Some(si) = g.obj(stack_obj).stack.as_deref() else {
            return Some(vec![]);
        };
        let (source, ability) = match &si.kind {
            StackKind::Triggered { source, ability } | StackKind::Activated { source, ability } => {
                (*source, ability)
            }
            StackKind::Spell => return Some(vec![]),
        };
        // Revealed by this object's ability linked to this triggered ability (or a copy of
        // it: same source, CR 707.10b).
        let linked = source == src
            && ability.link != 0
            && g.obj(src).chars.abilities.iter().any(|a| {
                a.link == ability.link
                    && matches!(&a.kind, AbilityKind::Triggered(t) if is_this_way(&t.trigger))
            });
        if !linked {
            return Some(vec![]);
        }
        Some(vec![EventInfo {
            object: Some(*card),
            player: Some(*p),
            // "The result": what the revealing ability's trigger event was about.
            amount: si.event.as_ref().map_or(0, |e| e.amount),
            ..Default::default()
        }])
    }
}

inventory::submit! { KeywordRegistration(&RevealedThisWay) }
