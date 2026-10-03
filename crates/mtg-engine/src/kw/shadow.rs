//! CR 702.28 Shadow, an evasion ability (CR 702.28a): "A creature with shadow can't be
//! blocked by creatures without shadow, and a creature without shadow can't be blocked by
//! creatures with shadow" (CR 702.28b). Multiple instances are redundant (CR 702.28c).
//!
//! Some creatures ignore shadow when blocking: "~ can block creatures with shadow as though
//! it had shadow" (it counts as having shadow when blocking a creature with shadow) and
//! "~ can block creatures with shadow as though they didn't have shadow" (the attacker
//! counts as not having shadow — so a blocker with shadow couldn't block it).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// "~ can block creatures with shadow as though it had shadow" (a `StaticEffect::Custom`).
pub const BLOCKS_SHADOW_AS_THOUGH_IT_HAD_SHADOW: &str =
    "can block creatures with shadow as though it had shadow";
/// "~ can block creatures with shadow as though they didn't have shadow".
pub const BLOCKS_SHADOW_AS_THOUGH_THEY_DIDNT: &str =
    "can block creatures with shadow as though they didn't have shadow";

pub struct Shadow;

fn has_static(g: &Game, id: ObjectId, name: &str) -> bool {
    let o = g.obj(id);
    o.chars.abilities.iter().any(|a| match &a.kind {
        AbilityKind::Static(s) => {
            matches!(&s.effect, StaticEffect::Custom(n) if n.as_str() == name)
                && g.ability_functions(o, s.zone, s.is_cda)
        }
        _ => false,
    })
}

impl KeywordRules for Shadow {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Shadow]
    }

    fn block_allowed(&self, g: &Game, blocker: ObjectId, attacker: ObjectId) -> bool {
        let mut attacker_shadow = g.obj(attacker).has_keyword(KeywordKind::Shadow);
        let mut blocker_shadow = g.obj(blocker).has_keyword(KeywordKind::Shadow);
        if attacker_shadow && has_static(g, blocker, BLOCKS_SHADOW_AS_THOUGH_THEY_DIDNT) {
            attacker_shadow = false;
        }
        if attacker_shadow && has_static(g, blocker, BLOCKS_SHADOW_AS_THOUGH_IT_HAD_SHADOW) {
            blocker_shadow = true;
        }
        attacker_shadow == blocker_shadow
    }
}

inventory::submit! { KeywordRegistration(&Shadow) }
