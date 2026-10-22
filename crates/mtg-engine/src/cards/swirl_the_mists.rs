//! Swirl the Mists: "As ~ enters, choose a color word." and "All instances of color words
//! in the text of spells and permanents are changed to the chosen color word."
//!
//! A text-changing static ability (CR 612.2) whose replacement word is the one chosen as
//! it entered; it applies in layer 3 to every spell and permanent, including ones that
//! appear later, for as long as ~ is on the battlefield (rulings).

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Characteristics;
use crate::types::{Color, ObjectId};

const SWIRL: &str = "card:Swirl the Mists:color words are changed to the chosen color word";
const STATIC_TEXT: &str = "All instances of color words in the text of spells and permanents are changed to the chosen color word.";

inventory::submit! { ManualAbility {
    card: "Swirl the Mists",
    face: 0,
    text: "As ~ enters, choose a color word.",
    // The choice of a color word: one of the five color words (CR 105.1).
    build: |ctx| {
        parse(ctx, "As ~ enters, choose a color.")
            .iter()
            .map(|a| map_effect(a, "As ~ enters, choose a color word.", |e| e))
            .collect()
    },
    reason: "choosing a color word for a text change: only this card",
} }

inventory::submit! { ManualAbility {
    card: "Swirl the Mists",
    face: 0,
    text: STATIC_TEXT,
    build: |_ctx| {
        vec![AbilityDef::new(
            AbilityKind::Static(StaticAbility {
                condition: None,
                effect: StaticEffect::Continuous {
                    affected: Filter::Or(vec![Filter::Permanent, Filter::SpellOnStack]),
                    mods: vec![Modification::Custom {
                        name: SWIRL.into(),
                        layer: Layer::L3Text,
                    }],
                },
                zone: FunctionZone::Battlefield,
                is_cda: false,
            }),
            STATIC_TEXT,
        )]
    },
    reason: "a text change of every color word to a word chosen as it entered: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_modification(
        &self,
        g: &Game,
        name: &str,
        chars: &mut Characteristics,
        ctx: &Ctx,
        _target: ObjectId,
    ) -> bool {
        if name != SWIRL {
            return false;
        }
        let Some(to) = ctx.source.and_then(|s| g.obj(s).choices.color) else {
            // No color word was chosen: nothing changes.
            return true;
        };
        for from in Color::ALL {
            if from != to {
                crate::text_change::change_text(chars, from.word(), to.word());
            }
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
