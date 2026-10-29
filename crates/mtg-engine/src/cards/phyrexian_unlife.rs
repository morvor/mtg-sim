//! Phyrexian Unlife: "As long as you have 0 or less life, all damage is dealt to you as
//! though its source had infect." (CR 120.3b, 702.90b).

use super::{active, marker, ManualAbility};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{ObjectId, PlayerId};

const INFECT: &str = "card:Phyrexian Unlife:damage dealt to you as though its source had infect";

inventory::submit! { ManualAbility {
    card: "Phyrexian Unlife",
    face: 0,
    text: "As long as you have 0 or less life, all damage is dealt to you as though its source had infect.",
    build: |_| vec![marker(
        INFECT,
        "As long as you have 0 or less life, all damage is dealt to you as though its source had infect.",
    )],
    reason: "damage dealt to you as though its source had infect while you have 0 or less life: unique",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    /// The life total is checked as the damage is dealt: damage that brings it from
    /// positive to 0 or less is dealt normally.
    fn damage_as_though_infect(&self, g: &Game, _source: ObjectId, p: PlayerId) -> bool {
        g.player(p).life <= 0 && active(g, INFECT).iter().any(|(_, c)| *c == p)
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
