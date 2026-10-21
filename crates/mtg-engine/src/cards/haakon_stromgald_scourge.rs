//! Haakon, Stromgald Scourge: "You may cast ~ from your graveyard, but not from anywhere
//! else." A permission (compiled) and a prohibition on casting it from any other zone,
//! even with another effect's permission (ruling, CR 601.3).

use super::{marker, parse, ManualAbility};
use crate::ability::{AbilityDef, ZoneKind};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::{Characteristics, Zone};
use crate::types::{ObjectId, PlayerId};

const ONLY_FROM_GRAVEYARD: &str = "card:Haakon, Stromgald Scourge:can't be cast from anywhere but your graveyard";
const TEXT: &str = "You may cast ~ from your graveyard, but not from anywhere else.";

inventory::submit! { ManualAbility {
    card: "Haakon, Stromgald Scourge",
    face: 0,
    text: TEXT,
    build: |ctx| {
        let mut v: Vec<_> = parse(ctx, "You may cast ~ from your graveyard.")
            .iter()
            .map(|a| AbilityDef::new(a.kind.clone(), TEXT))
            .collect();
        let mut m = marker(ONLY_FROM_GRAVEYARD, TEXT);
        // The prohibition functions wherever the card is (CR 113.6).
        if let crate::ability::AbilityKind::Static(s) = &mut std::sync::Arc::make_mut(&mut m).kind {
            s.zone = crate::ability::FunctionZone::Anywhere;
        }
        v.push(m);
        v
    },
    reason: "may be cast from your graveyard and from no other zone: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn cast_prohibited(
        &self,
        g: &Game,
        p: PlayerId,
        card: ObjectId,
        chars: &Characteristics,
    ) -> bool {
        if !chars.abilities.iter().any(|a| super::is_marker(a, ONLY_FROM_GRAVEYARD)) {
            return false;
        }
        // On the stack while its proposal is checked, it's being cast from where it was.
        let o = g.obj(card);
        let from = match o.zone {
            Zone::Stack => o.stack.as_deref().and_then(|si| si.cast.from),
            z => z.kind(),
        };
        // A card in a graveyard is in its owner's: "your graveyard".
        !(from == Some(ZoneKind::Graveyard) && o.owner == p)
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
