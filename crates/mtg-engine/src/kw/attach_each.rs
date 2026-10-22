//! "For each Aura and Equipment you control, you may attach it to a creature you control."
//! (Inventory Management): each of those permanents, one at a time, may become attached
//! to an object its controller chooses among those it could legally be attached to
//! (CR 701.3a–b). The oracle pattern is in `oracle/patterns/a701_attach_each.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// The variable holding the permanent being attached (see [`attach_each`]).
pub const ATTACHING: Var = vars::USER + 1703;
/// `Filter::Custom`: an object the permanent in [`ATTACHING`] could legally be attached to.
pub const CAN_ATTACH_IT: &str = "attach each: can attach it";

/// "For each [what], [you may] attach it to [a to]": for each permanent in `what`, its
/// controller (the ability's controller) chooses an object matching `to` it could legally
/// be attached to, and attaches it.
pub fn attach_each(what: Filter, to: Filter, optional: bool) -> Effect {
    let attach = Effect::Attach {
        what: Sel::Var(ATTACHING),
        to: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![
                to,
                Filter::InZone(ZoneKind::Battlefield),
                Filter::Custom(CAN_ATTACH_IT.into()),
            ]),
            count: Value::c(1),
            up_to: false,
            store: None,
        },
    };
    Effect::ForEach {
        sel: Sel::All(Filter::and(vec![
            what,
            Filter::InZone(ZoneKind::Battlefield),
        ])),
        var: ATTACHING,
        effect: Box::new(if optional {
            Effect::May {
                who: PlayerRef::You,
                effect: Box::new(attach),
            }
        } else {
            attach
        }),
    }
}

pub struct AttachEach;

impl KeywordRules for AttachEach {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != CAN_ATTACH_IT {
            return None;
        }
        let it = ctx.vars.get(&ATTACHING).and_then(|v| v.first().copied());
        Some(match it {
            Some(Entity::Object(o)) => {
                o != id && crate::attach::can_attach(g, o, Entity::Object(id))
            }
            _ => false,
        })
    }
}

inventory::submit! { KeywordRegistration(&AttachEach) }
