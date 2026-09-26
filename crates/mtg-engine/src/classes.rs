//! Class cards (CR 716): a class level bar is an activated ability that sets the Class's
//! level ([`crate::ability::Effect::SetClassLevel`]) and a static ability granting the
//! abilities of its section at that level or greater (CR 716.2a; compiled in
//! `oracle/patterns/r107_symbols.rs`).
//!
//! * A level is a designation of the permanent (`GameObject::class_level`): kept even if
//!   it stops being a Class, not copiable, and a permanent without one is treated as
//!   level 1 (CR 716.2b, 716.2d).
//! * "When this Class becomes level N" triggers as its level becomes N ([`BECAME_LEVEL`]).
//! * "To gain a Class level" is to activate a class level bar's ability (CR 716.2c):
//!   mana that may be spent only to gain a Class level can pay only for those.

use crate::ability::*;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::EventInfo;
use crate::types::*;
use smol_str::SmolStr;

/// `Event::Custom` name: a permanent became level `amount` (`obj`).
pub const BECAME_LEVEL: &str = "became class level";
/// `TriggerCond::Custom` name prefix of "When this Class becomes level N".
pub const BECOMES_LEVEL: &str = "class becomes level:";

/// The permanent `id`'s level: a permanent without one is treated as level 1
/// (CR 716.2d).
pub fn level(g: &Game, id: ObjectId) -> u32 {
    g.obj(id).class_level.max(1)
}

/// Sets the level of the permanent `id` (CR 716.2a).
pub fn set_level(g: &mut Game, id: ObjectId, level: u32) {
    if !g.is_live(id) {
        return;
    }
    let o = g.obj_mut(id);
    let changed = o.class_level != level;
    o.class_level = level;
    g.dirty = true;
    if changed {
        let controller = g.obj(id).controller;
        g.log(|g| format!("{} becomes level {level}", g.describe(id)));
        g.emit(Event::Custom {
            name: SmolStr::new(BECAME_LEVEL),
            player: Some(controller),
            obj: Some(id),
            amount: level as i32,
        });
    }
}

/// Whether an activated ability is a class level bar's (CR 716.2c: activating one is
/// gaining a Class level).
pub fn gains_a_level(act: &ActivatedAbility) -> bool {
    matches!(act.body.effect, Effect::SetClassLevel { .. })
}

/// Hooks for Class cards in the keyword registry (a class level bar isn't a keyword in
/// the registry's sense).
pub struct ClassRules;

impl KeywordRules for ClassRules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_trigger(
        &self,
        _g: &Game,
        name: &str,
        src: ObjectId,
        _ctl: PlayerId,
        ev: &Event,
    ) -> Option<Vec<EventInfo>> {
        let n: i32 = name.strip_prefix(BECOMES_LEVEL)?.parse().ok()?;
        Some(match ev {
            Event::Custom {
                name: e,
                player,
                obj: Some(o),
                amount,
            } if e == BECAME_LEVEL && *o == src && *amount == n => vec![EventInfo {
                object: Some(src),
                player: *player,
                amount: n,
                ..Default::default()
            }],
            _ => vec![],
        })
    }
}

inventory::submit! { KeywordRegistration(&ClassRules) }
