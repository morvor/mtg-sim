//! Guile: "If a spell or ability you control would counter a spell, instead exile that
//! spell and you may play that card without paying its mana cost." A replacement of the
//! counter event itself (CR 701.6, 614.1a): a spell that can't be countered isn't
//! affected (it wouldn't be countered); the exile is mandatory, the cast optional and only
//! right then (CR 608.2g); the spell was never countered, and the countering spell or
//! ability goes on resolving (rulings).

use super::{active, marker, ManualAbility};
use crate::event_causes::Cause;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::CastMethod;
use crate::types::ObjectId;

const TEXT: &str = "If a spell or ability you control would counter a spell, instead exile that spell and you may play that card without paying its mana cost.";
const REPLACE: &str = "card:Guile:countering a spell exiles it instead";

inventory::submit! { ManualAbility {
    card: "Guile",
    face: 0,
    text: TEXT,
    build: |_| vec![marker(REPLACE, TEXT)],
    reason: "replaces countering a spell (CR 701.6, 614.1a): no other card replaces the counter event",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn replace_counter(&self, g: &mut Game, spell: ObjectId, cause: Cause) -> bool {
        let Some(by) = cause
            .by
            .or_else(|| cause.obj.and_then(|o| g.try_obj(o)).map(|o| o.controller))
        else {
            return false;
        };
        // Countering by a game rule isn't countering by a spell or ability you control.
        if cause.obj.is_none() {
            return false;
        }
        let Some((src, you)) = active(g, REPLACE).into_iter().find(|(_, c)| *c == by) else {
            return false;
        };
        g.log(|g| format!("{} exiles {} instead", g.describe(src), g.describe(spell)));
        let Some(card) = g.exile_object(spell, Some(src)) else {
            return true;
        };
        if g.is_live(card)
            && g.obj(card).zone == crate::object::Zone::Exile
            && g.ask_yes_no(
                you,
                Some(card),
                "Cast that card without paying its mana cost?",
                true,
            )
        {
            let _ = crate::casting::cast_during_resolution(g, you, card, CastMethod::Free);
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
