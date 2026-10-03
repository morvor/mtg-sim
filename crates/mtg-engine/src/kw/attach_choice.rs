//! "You may attach any number of Auras and Equipment you control to target permanent or
//! player" (Ardenn, Intrepid Archaeologist), "you may attach an Equipment you control to
//! it" (Nahiri, the Lithomancer): the permanents to attach are chosen as the instruction
//! is performed (CR 608.2c), among those that could legally be attached to that object or
//! player (CR 701.3a: an Aura or Equipment can't be attached to something it couldn't
//! enchant or equip). The oracle pattern is in `oracle/patterns/attach_control_grammar.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// The object or player the chosen permanents are attached to, stored before they're
/// chosen.
pub const RECIPIENT: Var = vars::USER + 8301;

/// `Filter::Custom`: a permanent that could legally be attached to [`RECIPIENT`].
pub const CAN_ATTACH_TO_RECIPIENT: &str = "attach grammar: can be attached to the recipient";

/// Attaches the permanents `choose` chooses among those matching its filter that could be
/// attached to `to` (evaluated first).
pub fn attach_chosen(choose: Sel, to: Sel) -> Effect {
    let what = match choose {
        Sel::Choose {
            chooser,
            filter,
            count,
            up_to,
            store,
        } => Sel::Choose {
            chooser,
            filter: Filter::and(vec![filter, Filter::Custom(CAN_ATTACH_TO_RECIPIENT.into())]),
            count,
            up_to,
            store,
        },
        other => other,
    };
    Effect::Seq(vec![
        Effect::Store {
            var: RECIPIENT,
            sel: to,
        },
        Effect::Attach {
            what,
            to: Sel::Var(RECIPIENT),
        },
    ])
}

/// `Filter::Custom`: an object with an Aura other than the source attached to it ("Enchant
/// creature with another Aura attached to it", Daybreak Coronet; CR 303.4c makes the
/// source go to the graveyard once it's the only Aura left).
pub const ANOTHER_AURA_ATTACHED: &str = "attach grammar: with another Aura attached to it";

pub struct AttachChoice;

impl KeywordRules for AttachChoice {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name == ANOTHER_AURA_ATTACHED {
            // Not the source (nor what it became as it resolved).
            let is_source = |a: ObjectId| {
                ctx.source
                    .is_some_and(|s| s == a || g.current(s) == a || g.current(a) == s)
            };
            return Some(
                g.attachments_of(Entity::Object(id))
                    .iter()
                    .any(|a| !is_source(*a) && g.obj(*a).chars.has_subtype("Aura")),
            );
        }
        if name != CAN_ATTACH_TO_RECIPIENT {
            return None;
        }
        let to = ctx.vars.get(&RECIPIENT).and_then(|v| v.first().copied());
        Some(to.is_some_and(|to| to != Entity::Object(id) && crate::attach::can_attach(g, id, to)))
    }
}

inventory::submit! { KeywordRegistration(&AttachChoice) }
