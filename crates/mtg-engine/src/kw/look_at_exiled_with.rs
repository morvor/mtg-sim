//! "You may look at cards exiled with ~, and you may play lands and cast spells from among
//! those cards." (Kheru Mind-Eater, Colfenor's Plans): whoever controls the permanent may
//! look at the face-down cards linked to it in exile (CR 406.3, 607.2a). A new controller
//! can look at them too, and a player who has looked at such a card may keep looking at
//! it for as long as it stays exiled, even after the permanent has left the battlefield
//! (CR 406.3b). Like hideaway's permission, this isn't a state-based action: it's kept up
//! to date whenever state-based actions are checked.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{AbilityKind, StaticEffect};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::types::*;

/// `StaticEffect::Custom` name of the ability.
pub const MAY_LOOK_AT_LINKED: &str = "may look at cards exiled with this";

pub struct LookAtExiledWith;

impl KeywordRules for LookAtExiledWith {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn state_based_actions(&self, g: &mut Game) -> bool {
        let mut allow: Vec<(PlayerId, ObjectId)> = Vec::new();
        for o in g.permanents() {
            let has = o.chars.abilities.iter().any(|a| {
                matches!(&a.kind, AbilityKind::Static(s)
                    if matches!(&s.effect, StaticEffect::Custom(n) if n == MAY_LOOK_AT_LINKED))
            });
            if !has {
                continue;
            }
            for c in o.linked.values().flatten() {
                let c = g.current(*c);
                let e = g.obj(c);
                if e.zone == Zone::Exile
                    && e.face_down
                    && !crate::zones::may_look(g, o.controller, c)
                {
                    allow.push((o.controller, c));
                }
            }
        }
        for (p, c) in allow {
            crate::zones::allow_look(g, p, c);
        }
        false
    }
}

inventory::submit! { KeywordRegistration(&LookAtExiledWith) }
