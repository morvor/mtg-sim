//! Ogre Enforcer: "~ can't be destroyed by lethal damage unless lethal damage dealt by a
//! single source is marked on it." (an exception to CR 704.5g).

use super::{has_marker, marker, ManualAbility};
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{Entity, ObjectId};

const SINGLE_SOURCE: &str = "card:Ogre Enforcer:lethal damage only from a single source";
/// Rows `[object, source, amount, turn]`: damage dealt to permanents with the ability.
const DEALT: &str = "card:Ogre Enforcer:damage by source";

inventory::submit! { ManualAbility {
    card: "Ogre Enforcer",
    face: 0,
    text: "~ can't be destroyed by lethal damage unless lethal damage dealt by a single source is marked on it.",
    build: |_| vec![marker(
        SINGLE_SOURCE,
        "~ can't be destroyed by lethal damage unless lethal damage dealt by a single source is marked on it.",
    )],
    reason: "destroyed by lethal damage only if a single source dealt it (an exception to CR 704.5g): unique",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    /// Keeps track of the damage each source dealt to a permanent with the ability this
    /// turn (damage marked on it is removed in the cleanup step, CR 514.2).
    fn on_event(&self, g: &mut Game, ev: &Event) {
        let Event::Damage {
            source,
            target: Entity::Object(o),
            amount,
            ..
        } = ev
        else {
            return;
        };
        if *amount == 0 || !g.is_live(*o) || !has_marker(g, *o, SINGLE_SOURCE) {
            return;
        }
        let turn = g.turn.number as i64;
        g.cards.retain(DEALT, |r| r[3] == turn);
        g.cards
            .push(DEALT, vec![o.0 as i64, source.0 as i64, *amount as i64, turn]);
    }

    fn survives_lethal_damage(&self, g: &Game, creature: ObjectId) -> bool {
        if !has_marker(g, creature, SINGLE_SOURCE) {
            return false;
        }
        let lethal = crate::kw::lethal_damage_basis(g, creature).max(1) as i64;
        let turn = g.turn.number as i64;
        let mut by_source: std::collections::BTreeMap<i64, i64> = Default::default();
        for r in g.cards.get(DEALT) {
            if r[0] == creature.0 as i64 && r[3] == turn {
                *by_source.entry(r[1]).or_default() += r[2];
            }
        }
        // Damage marked can't be more than what's left marked on it (e.g. after
        // regeneration removed it).
        let marked = g.obj(creature).damage as i64;
        !by_source.values().any(|&n| n.min(marked) >= lethal)
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
