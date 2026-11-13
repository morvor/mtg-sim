//! `Filter::Custom` "has protection from [color]": an object with a protection ability
//! whose quality is exactly that color (CR 702.16a), for "As long as an opponent controls
//! a creature with protection from a color ..., ~ has protection from that color"
//! (Escaped Shapeshifter: "The same is true for ... protection from any color").

use super::{KeywordRegistration, KeywordRules};
use crate::ability::Filter;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::{Color, ObjectId};

/// Prefix of the filter's name, followed by the color word.
pub const PREFIX: &str = "has protection from color:";

/// The filter for objects with protection from `c`.
pub fn filter(c: Color) -> Filter {
    Filter::Custom(format!("{PREFIX}{}", c.word()).into())
}

pub struct ProtectionFromColorFilter;

impl KeywordRules for ProtectionFromColorFilter {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        let c = Color::from_word(name.strip_prefix(PREFIX)?)?;
        Some(
            g.obj(id)
                .chars
                .keywords_of(KeywordKind::Protection)
                .any(|k| matches!(&k.filter, Some(Filter::Color(x)) if *x == c)),
        )
    }
}

inventory::submit! { KeywordRegistration(&ProtectionFromColorFilter) }
