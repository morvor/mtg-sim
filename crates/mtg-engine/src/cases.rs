//! Case cards (CR 719).
//!
//! * "To solve — [Condition]" means "At the beginning of your end step, if [condition]
//!   and this Case is not solved, this Case becomes solved" (CR 719.3a).
//! * Solved is a designation (`GameObject::solved`): it stays until the permanent leaves
//!   the battlefield, and it's neither an ability nor copiable (CR 719.3b).
//! * "Solved — [Ability]" functions only while the Case is solved (CR 719.3c, 702.169): a
//!   static ability granting it to the Case with the condition [`SOLVED`].
//!
//! The oracle patterns are in `oracle/patterns/r719_cases.rs`.

use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;
use crate::types::*;
use smol_str::SmolStr;

/// `Effect::Custom`: "this Case becomes solved".
pub const SOLVE: &str = "case: becomes solved";
/// `Condition::Custom`: "this Case is solved".
pub const SOLVED: &str = "case: is solved";
/// `Event::Custom` name: a permanent became solved (`obj`).
pub const BECAME_SOLVED: &str = "became solved";

/// Whether the permanent `id` has the solved designation.
pub fn is_solved(g: &Game, id: ObjectId) -> bool {
    let o = g.obj(id);
    g.is_live(id) && o.zone == Zone::Battlefield && o.solved
}

/// The permanent `id` becomes solved (CR 719.3a, 719.3b). Returns false if it already
/// was, or isn't a permanent.
pub fn solve(g: &mut Game, id: ObjectId) -> bool {
    if !g.is_live(id) || g.obj(id).zone != Zone::Battlefield || g.obj(id).solved {
        return false;
    }
    g.obj_mut(id).solved = true;
    g.dirty = true;
    let controller = g.obj(id).controller;
    g.log(|g| format!("{} becomes solved", g.describe(id)));
    g.emit(Event::Custom {
        name: SmolStr::new(BECAME_SOLVED),
        player: Some(controller),
        obj: Some(id),
        amount: 0,
    });
    true
}

/// Hooks for Case cards in the keyword registry: solving (CR 719.3a) and the solved
/// condition of "Solved —" abilities (CR 702.169, 719.3c).
pub struct CaseRules;

impl KeywordRules for CaseRules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != SOLVE {
            return false;
        }
        if let Some(s) = ctx.source {
            solve(g, s);
        }
        true
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        (name == SOLVED).then(|| ctx.source.is_some_and(|s| is_solved(g, s)))
    }
}

inventory::submit! { KeywordRegistration(&CaseRules) }
