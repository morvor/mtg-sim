//! Rules for two phrases of the Clockwork creatures (Antiquities, Alliances):
//!
//! * the condition "if ~ attacked or blocked this combat": the source was declared as an
//!   attacking or a blocking creature in the current combat (CR 508.1, 509.1), even if it
//!   has since been removed from combat. A creature put onto the battlefield attacking
//!   never attacked (CR 508.4).
//! * "Put up to X [kind] counters on ~. This ability can't cause the total number of
//!   [kind] counters on ~ to be greater than N.": the controller chooses a number from 0
//!   to X, limited by how many more counters of that kind ~ can have (CR 122.1).

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::types::*;

/// `Condition::Custom`: the source attacked or blocked this combat.
pub const ATTACKED_OR_BLOCKED_THIS_COMBAT: &str = "combat:this attacked or blocked this combat";

/// `Effect::Custom` prefix: "put up to X [kind] counters on ~", as
/// `counters:up to x:<kind>` or, with a cap on the total, `counters:up to x:<kind>:<cap>`.
pub const PUT_UP_TO_X: &str = "counters:up to x:";

/// The name of the "put up to X [kind] counters on ~" effect, with an optional cap.
pub fn put_up_to_x(kind: &str, cap: Option<u32>) -> String {
    match cap {
        Some(c) => format!("{PUT_UP_TO_X}{kind}:{c}"),
        None => format!("{PUT_UP_TO_X}{kind}"),
    }
}

fn attacked_or_blocked_this_combat(g: &Game, id: ObjectId) -> bool {
    g.combat.as_ref().is_some_and(|c| {
        c.declared_attackers.iter().any(|(a, _)| *a == id) || c.declared_blockers.contains(&id)
    })
}

pub struct CombatCounterPhrases;

impl KeywordRules for CombatCounterPhrases {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        (name == ATTACKED_OR_BLOCKED_THIS_COMBAT)
            .then(|| ctx.source.is_some_and(|s| attacked_or_blocked_this_combat(g, s)))
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some(spec) = name.strip_prefix(PUT_UP_TO_X) else {
            return false;
        };
        let (kind, cap) = match spec.rsplit_once(':') {
            Some((k, c)) if c.parse::<u32>().is_ok() => (k, c.parse::<u32>().ok()),
            _ => (spec, None),
        };
        let Some(src) = ctx.source else {
            return true;
        };
        if !g.is_live(src) || g.obj(src).zone != Zone::Battlefield {
            return true;
        }
        let x = ctx.x.max(0) as u32;
        let have = g.obj(src).counter(kind);
        let most = cap.map_or(x, |c| c.saturating_sub(have).min(x));
        if most == 0 {
            return true;
        }
        let n = g.ask_number(
            ctx.controller,
            Some(src),
            &format!("Put how many {kind} counters (up to {most})?"),
            0,
            most as i64,
        );
        if n > 0 {
            g.add_counters(Entity::Object(src), kind, n as u32, Some(src));
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&CombatCounterPhrases) }
