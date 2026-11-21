//! "Damage isn't removed from [permanents] during cleanup steps" (Ancient Adamantoise:
//! "... from this creature ...", Uthgardt Fury and Patient Zero: "... from creatures your
//! opponents control ...", Case of the Market Melee: "... from creatures ..."): an
//! exception to CR 514.2, under which all damage marked on permanents is removed in the
//! cleanup step.
//!
//! Only the cleanup step's removal is affected: effects that remove damage (regeneration,
//! CR 701.19a) still do (Ancient Adamantoise ruling). A phased-out permanent's abilities
//! don't function and it's treated as though it doesn't exist (CR 702.26b), so damage on a
//! phased-out permanent is removed as usual (CR 514.2: "including phased-out
//! permanents").

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::ObjectId;

/// `Filter::Custom`: a permanent with damage marked on it ("damaged", CR 120.3e).
pub const DAMAGED: &str = "damaged";

/// Whether an active "damage isn't removed" ability keeps the damage on `id`.
pub fn keeps_damage(g: &Game, id: ObjectId) -> bool {
    if g.obj(id).phased_out {
        return false;
    }
    super::active_others(g, |e| match e {
        StaticEffect::DamageNotRemoved(f) => Some(f),
        _ => None,
    })
    .any(|(s, c, f)| g.matches(id, f, &Ctx::new(Some(s), c)))
}

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn keeps_damage_in_cleanup(&self, g: &Game, id: ObjectId) -> bool {
        keeps_damage(g, id)
    }
    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        (name == DAMAGED).then(|| g.obj(id).damage > 0)
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
