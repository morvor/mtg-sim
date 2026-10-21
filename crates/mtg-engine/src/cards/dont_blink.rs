//! Don't Blink: "Until end of turn, if one or more creatures would enter from exile or
//! after being cast from exile, their owners shuffle them into their libraries instead."

use super::{spell, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;
use crate::types::{CardType, ObjectId};

const FROM_EXILE: &str = "card:Don't Blink:entering from exile or cast from exile";
const TEXT: &str = "Until end of turn, if one or more creatures would enter from exile or after being cast from exile, their owners shuffle them into their libraries instead.";

inventory::submit! { ManualAbility {
    card: "Don't Blink",
    face: 0,
    text: TEXT,
    build: |_| {
        let mut to = Destination::zone(ZoneKind::Library);
        to.position = LibraryPosition::Shuffled;
        vec![spell(
            vec![],
            Effect::AddReplacement {
                def: ReplacementDef {
                    event: ReplacementEvent::ZoneChange {
                        filter: Filter::And(vec![
                            Filter::Type(CardType::Creature),
                            Filter::Custom(FROM_EXILE.into()),
                        ]),
                        from: None,
                        to: Some(ZoneKind::Battlefield),
                    },
                    action: ReplacementAction::MoveInstead(to),
                    self_replacement: false,
                    optional: false,
                },
                duration: Duration::EndOfTurn,
                uses: None,
            },
            TEXT,
        )]
    },
    reason: "creatures entering from exile or after being cast from exile are shuffled into their owners' libraries: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    /// The entering object is in exile, or is a spell cast from exile.
    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        if name != FROM_EXILE {
            return None;
        }
        let o = g.obj(id);
        Some(match o.zone {
            Zone::Exile => true,
            Zone::Stack => o
                .stack
                .as_deref()
                .is_some_and(|s| s.cast.was_cast && s.cast.from == Some(ZoneKind::Exile)),
            _ => false,
        })
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
