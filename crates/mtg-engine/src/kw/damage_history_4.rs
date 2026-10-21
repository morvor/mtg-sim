//! Amounts of damage dealt this turn that the history values don't tally event by event:
//! "the greatest amount of damage dealt by a source to a permanent or player this turn"
//! (Impact Resonance): the damage each source dealt to each permanent or player, summed
//! over the turn (CR 120.3), and the greatest such sum.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;
use std::collections::BTreeMap;

/// `Value::Custom`: the greatest total of damage one source dealt to one permanent or
/// player this turn.
pub const GREATEST_DAMAGE_BY_A_SOURCE: &str =
    "damage history:greatest damage dealt by a source to a permanent or player this turn";

/// `Filter::Custom`: the source itself or the spell it was (CR 400.7): "the first Otter
/// spell other than Alania you've cast this turn" (Alania, Divergent Storm) isn't Alania
/// as the spell it was cast as.
pub const SOURCE_OR_ITS_SPELL: &str = "history:the source or the spell it was";

/// `Filter::Custom`: shares a creature type with a creature that convoked the spell the
/// context refers to (CR 702.51c), each as it was tapped for it if it has left the
/// battlefield since (CR 608.2h): "all creatures except those that share a creature type
/// with a creature that convoked this spell" (Everything Comes to Dust).
pub const SHARES_TYPE_WITH_CONVOKER: &str = "history:shares a creature type with a creature that convoked it";

/// `Filter::Custom`: a permanent that entered the battlefield face down this turn (CR
/// 708.2a): it's face down now, or it was turned face up this turn, and it wasn't turned
/// face down after it entered.
pub const ENTERED_FACE_DOWN: &str = "history:entered the battlefield face down this turn";

/// Whether `id` (a permanent that entered this turn) entered face down.
fn entered_face_down(g: &Game, id: ObjectId) -> bool {
    let mut turned_up = false;
    for ev in g.turn_events.iter().chain(g.events.iter()) {
        match ev {
            Event::TurnedFaceDown { obj, .. } if *obj == id => return false,
            Event::TurnedFaceUp { obj, .. } if *obj == id => turned_up = true,
            _ => {}
        }
    }
    g.obj(id).face_down || turned_up
}

/// The greatest amount of damage one source dealt to one recipient this turn.
pub fn greatest_damage_by_a_source(g: &Game) -> i64 {
    let mut totals: BTreeMap<(ObjectId, Entity), i64> = BTreeMap::new();
    for ev in g.turn_events.iter().chain(g.events.iter()) {
        if let Event::Damage {
            source,
            target,
            amount,
            ..
        } = ev
        {
            *totals.entry((*source, *target)).or_insert(0) += *amount as i64;
        }
    }
    totals.values().copied().max().unwrap_or(0)
}

/// The creatures that convoked the spell, while [`SHARES_TYPE_WITH_CONVOKER`] is checked.
const CONVOKERS: crate::ability::Var = crate::ability::vars::USER + 31_402;

struct DamageHistory;

impl KeywordRules for DamageHistory {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, _ctx: &Ctx) -> Option<i64> {
        (name == GREATEST_DAMAGE_BY_A_SOURCE).then(|| greatest_damage_by_a_source(g))
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name == ENTERED_FACE_DOWN {
            return Some(entered_face_down(g, id));
        }
        if name == SHARES_TYPE_WITH_CONVOKER {
            let convokers: Vec<Entity> = g
                .cast_info(ctx)
                .map(|c| c.convoked.iter().map(|o| Entity::Object(*o)).collect())
                .unwrap_or_default();
            let mut c = ctx.clone();
            c.set_var(CONVOKERS, convokers);
            let shares = crate::ability::Filter::SharesCreatureType(Box::new(
                crate::ability::Sel::Var(CONVOKERS),
            ));
            return Some(g.matches(id, &shares, &c));
        }
        if name != SOURCE_OR_ITS_SPELL {
            return None;
        }
        let src = ctx.source?;
        Some(id == src || g.obj(src).prev == Some(id))
    }
}

inventory::submit! { KeywordRegistration(&DamageHistory) }
