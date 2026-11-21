//! Ghostly Flame: "Black and/or red permanents and spells are colorless sources of
//! damage." The sources keep their colors; only effects looking at the damage's source
//! (prevention, replacement, protection) see it as colorless.

use super::{active, marker, ManualAbility};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;
use crate::types::{Color, ColorSet, ObjectId};

const COLORLESS: &str = "card:Ghostly Flame:black and red sources of damage are colorless";

inventory::submit! { ManualAbility {
    card: "Ghostly Flame",
    face: 0,
    text: "Black and/or red permanents and spells are colorless sources of damage.",
    build: |_| vec![marker(
        COLORLESS,
        "Black and/or red permanents and spells are colorless sources of damage.",
    )],
    reason: "black/red permanents and spells are colorless sources of damage: unique damage-source color rule",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn damage_source_colors(&self, g: &Game, source: ObjectId) -> Option<ColorSet> {
        let o = g.obj(source);
        let br = o.chars.colors.contains(Color::Black) || o.chars.colors.contains(Color::Red);
        (br && matches!(o.zone, Zone::Battlefield | Zone::Stack)
            && !active(g, COLORLESS).is_empty())
        .then_some(ColorSet::NONE)
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
