//! CR 702.177 Exhaust: a keyword that adds rules to the activated ability that follows
//! it. "Exhaust — [Cost]: [Effect]" means "[Cost]: [Effect]. Activate only once."
//! (CR 702.177a).
//!
//! The oracle compiler turns "Exhaust — [cost]: [effect]" into an activated ability whose
//! text starts with "Exhaust" (see `oracle/patterns/k702_168_177.rs`), so it's an
//! "exhaust ability" ([`is_exhaust`]). How many times each ability of an object has been
//! activated is counted over the object's existence (`GameObject::activations`): once a
//! permanent's exhaust ability has been activated, it can't be activated again, until
//! the permanent becomes a new object (CR 400.7).
//!
//! An effect may allow a player to activate exhaust abilities as long as they haven't
//! activated an exhaust ability this turn (CR 702.177b): "During your turn, as long as
//! you haven't activated an exhaust ability this turn, you may activate exhaust abilities
//! as though they haven't been activated." ([`AS_THOUGH_NOT_ACTIVATED`]). That allows it
//! only if the player hasn't begun to activate an exhaust ability this turn: activations
//! are recorded as they begin (`TurnHistory::activations_begun`), so while one exhaust
//! ability is being activated (its targets chosen, its costs paid), another can't be
//! activated this way. The ability the player is beginning to activate isn't counted yet
//! as it's checked (CR 602.5).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::{EventInfo, StackKind};
use crate::types::*;

/// `StaticEffect::Custom`: "During your turn, as long as you haven't activated an exhaust
/// ability this turn, you may activate exhaust abilities as though they haven't been
/// activated." (Elvish Refueler; CR 702.177b).
pub const AS_THOUGH_NOT_ACTIVATED: &str =
    "exhaust:during your turn, you may activate exhaust abilities as though they haven't been activated";
/// `TriggerCond::Custom`: "Whenever you activate an exhaust ability".
pub const EXHAUST_ACTIVATED: &str = "exhaust:you activate an exhaust ability";

/// Whether `a` is an exhaust ability (CR 702.177a).
pub fn is_exhaust(a: &AbilityDef) -> bool {
    matches!(a.kind, AbilityKind::Activated(_))
        && crate::keyword_impls::ability_from_keyword(a) == Some(KeywordKind::Exhaust)
}

/// The exhaust ability with uid `uid` of `src`, if it is one.
fn exhaust_uid(g: &Game, src: ObjectId, uid: u64) -> bool {
    g.try_obj(src).is_some_and(|o| {
        o.chars
            .abilities
            .iter()
            .chain(o.base.abilities.iter())
            .any(|a| a.uid == uid && is_exhaust(a))
    })
}

/// Whether `p` has begun to activate an exhaust ability this turn (CR 702.177b), whether
/// or not that activation is complete.
pub fn began_exhaust_this_turn(g: &Game, p: PlayerId) -> bool {
    g.history
        .activations_begun
        .iter()
        .any(|(q, src, uid)| *q == p && exhaust_uid(g, *src, *uid))
}

/// Whether an effect lets `p` activate exhaust abilities as though they haven't been
/// activated right now (CR 702.177b).
fn may_activate_again(g: &Game, p: PlayerId) -> bool {
    g.turn.active == p
        && g.statics
            .customs
            .iter()
            .any(|(_, ctl, name)| name == AS_THOUGH_NOT_ACTIVATED && *ctl == p)
        && !began_exhaust_this_turn(g, p)
}

pub struct Exhaust;

impl KeywordRules for Exhaust {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Exhaust]
    }

    /// "Activate only once" (CR 702.177a), unless an effect lets the player activate it as
    /// though it hadn't been activated (CR 702.177b).
    fn activation_allowed(&self, g: &Game, p: PlayerId, src: ObjectId, a: &Ability) -> bool {
        if !is_exhaust(a) {
            return true;
        }
        let used = g.obj(src).activations.get(&a.uid).copied().unwrap_or(0);
        used == 0 || may_activate_again(g, p)
    }

    /// "Whenever you activate an exhaust ability" (mana abilities included: they're
    /// recorded as they're activated).
    fn custom_trigger(
        &self,
        g: &Game,
        name: &str,
        _src: ObjectId,
        ctl: PlayerId,
        ev: &Event,
    ) -> Option<Vec<EventInfo>> {
        if name != EXHAUST_ACTIVATED {
            return None;
        }
        let Event::AbilityActivated {
            ability,
            source,
            player,
            ..
        } = ev
        else {
            return Some(vec![]);
        };
        let exhaust = match ability {
            Some(ab) => matches!(
                g.obj(*ab).stack.as_deref().map(|s| &s.kind),
                Some(StackKind::Activated { ability, .. }) if is_exhaust(ability)
            ),
            // A mana ability: the one just recorded for that source.
            None => g
                .history
                .activated
                .iter()
                .rev()
                .find(|(_, s, _)| s == source)
                .is_some_and(|(_, s, uid)| exhaust_uid(g, *s, *uid)),
        };
        Some(if exhaust && *player == ctl {
            vec![EventInfo {
                object: Some(*source),
                player: Some(*player),
                spell: *ability,
                ..Default::default()
            }]
        } else {
            vec![]
        })
    }
}

inventory::submit! { KeywordRegistration(&Exhaust) }
