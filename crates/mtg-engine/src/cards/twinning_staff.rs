//! Twinning Staff: "If you would copy a spell one or more times, instead copy it that many
//! times plus an additional time. You may choose new targets for the additional copy."
//! Each Twinning Staff adds a copy; copying a card rather than a spell doesn't (rulings).
//! It applies to the effects that copy spells (`Effect::CopySpell`, storm, replicate, ...;
//! see `copy::additional_copies`).

use super::{active, marker, ManualAbility};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::PlayerId;

const TWIN: &str = "card:Twinning Staff:copying a spell makes an additional copy";
const TEXT: &str = "If you would copy a spell one or more times, instead copy it that many times plus an additional time. You may choose new targets for the additional copy.";

inventory::submit! { ManualAbility {
    card: "Twinning Staff",
    face: 0,
    text: TEXT,
    build: |_| vec![marker(TWIN, TEXT)],
    reason: "a replacement effect of copying a spell (one more copy): only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn additional_spell_copies(&self, g: &Game, controller: PlayerId) -> u32 {
        active(g, TWIN)
            .into_iter()
            .filter(|(_, you)| *you == controller)
            .count() as u32
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
