//! CR 702.142 Boast: a keyword that adds rules to the activated ability that follows it.
//! "Boast — [Cost]: [Effect]" means "[Cost]: [Effect]. Activate only if this creature
//! attacked this turn and only once each turn." (CR 702.142a).
//!
//! The oracle compiler turns "Boast — [cost]: [effect]" into an activated ability whose
//! text starts with "Boast" and whose condition is that its source attacked this turn
//! ([`boast_ability`]); the once-each-turn limit is checked here
//! ([`KeywordRules::activation_allowed`]) so that an effect can raise it ("Creatures you
//! control can boast twice during each of your turns rather than once", [`BOAST_TWICE`]).
//!
//! Effects may refer to boast abilities; a creature boasting means its boast ability
//! being activated (CR 702.142b): "Whenever you activate a boast ability"
//! ([`BOAST_ACTIVATED`]), "Boast abilities you activate cost {1} less to activate"
//! (a cost modifier for the keyword, see `keyword_impls::ability_from_keyword`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::{EventInfo, StackKind};
use crate::types::*;

/// `StaticEffect::Custom`: "Creatures you control can boast twice during each of your
/// turns rather than once."
pub const BOAST_TWICE: &str = "boast:creatures you control can boast twice during each of your turns";
/// `TriggerCond::Custom`: "Whenever you activate a boast ability" (CR 702.142b).
pub const BOAST_ACTIVATED: &str = "boast:you activate a boast ability";

/// Whether `a` is a boast ability.
pub fn is_boast(a: &AbilityDef) -> bool {
    matches!(a.kind, AbilityKind::Activated(_))
        && crate::keyword_impls::ability_from_keyword(a) == Some(KeywordKind::Boast)
}

/// Adds boast's "Activate only if this creature attacked this turn" to an activated
/// ability (CR 702.142a).
pub fn boast_ability(mut act: ActivatedAbility) -> ActivatedAbility {
    let attacked = Condition::SelMatches(Sel::This, Filter::AttackedThisTurn);
    act.condition = Some(match act.condition.take() {
        Some(c) => Condition::And(vec![c, attacked]),
        None => attacked,
    });
    act
}

/// How many times each boast ability of `src` may be activated this turn.
fn boasts_per_turn(g: &Game, src: ObjectId) -> u32 {
    let o = g.obj(src);
    let twice = o.is_creature()
        && g.turn.active == o.controller
        && g.statics
            .customs
            .iter()
            .any(|(_, ctl, name)| name == BOAST_TWICE && *ctl == o.controller);
    if twice {
        2
    } else {
        1
    }
}

pub struct Boast;

impl KeywordRules for Boast {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Boast]
    }

    /// "... and only once each turn" (CR 702.142a).
    fn activation_allowed(&self, g: &Game, _p: PlayerId, src: ObjectId, a: &Ability) -> bool {
        if !is_boast(a) {
            return true;
        }
        let used = g
            .obj(src)
            .activations_this_turn
            .get(&a.uid)
            .copied()
            .unwrap_or(0);
        used < boasts_per_turn(g, src)
    }

    fn custom_trigger(
        &self,
        g: &Game,
        name: &str,
        _src: ObjectId,
        ctl: PlayerId,
        ev: &Event,
    ) -> Option<Vec<EventInfo>> {
        if name != BOAST_ACTIVATED {
            return None;
        }
        let Event::AbilityActivated {
            ability: Some(ability),
            source,
            player,
            ..
        } = ev
        else {
            return Some(vec![]);
        };
        let boast = matches!(
            g.obj(*ability).stack.as_deref().map(|s| &s.kind),
            Some(StackKind::Activated { ability, .. }) if is_boast(ability)
        );
        Some(if boast && *player == ctl {
            vec![EventInfo {
                object: Some(*source),
                player: Some(*player),
                ..Default::default()
            }]
        } else {
            vec![]
        })
    }
}

inventory::submit! { KeywordRegistration(&Boast) }
