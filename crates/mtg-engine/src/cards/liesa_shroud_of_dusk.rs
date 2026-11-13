//! Liesa, Shroud of Dusk: "Rather than pay {2} for each previous time you've cast ~ from the
//! command zone this game, pay 2 life that many times." Its commander tax (CR 903.8) is
//! paid in life, not mana (not optional, ruling).

use super::{marker, ManualAbility};
use crate::ability::{AbilityKind, FunctionZone};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{ObjectId, PlayerId};

const LIFE_TAX: &str = "card:Liesa, Shroud of Dusk:commander tax paid in life";
const TEXT: &str = "Rather than pay {2} for each previous time you've cast ~ from the command zone this game, pay 2 life that many times.";

inventory::submit! { ManualAbility {
    card: "Liesa, Shroud of Dusk",
    face: 0,
    text: TEXT,
    build: |_| {
        let mut m = marker(LIFE_TAX, TEXT);
        // It applies as the spell is cast (from the command zone, CR 113.6).
        if let AbilityKind::Static(s) = &mut std::sync::Arc::make_mut(&mut m).kind {
            s.zone = FunctionZone::Anywhere;
        }
        vec![m]
    },
    reason: "its commander tax is paid with 2 life for each previous cast instead of {2} (CR 903.8): only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn commander_tax_paid_with_life(&self, g: &Game, _p: PlayerId, card: ObjectId) -> bool {
        super::has_marker(g, card, LIFE_TAX)
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
