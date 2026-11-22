//! "Spells you cast cost {1} less to cast for each card type they share with cards exiled
//! with ~." (Cemetery Prowler): `Value::Custom` [`SHARED_WITH_LINKED_EXILE`], the number of
//! card types the spell being cast has that some card exiled with the source has (CR 205.2a;
//! the card types among them, not the number of cards: Cemetery Prowler's ruling).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// `Value::Custom`: card types the spell (`ctx.event.spell`) shares with the cards exiled
/// with the source.
pub const SHARED_WITH_LINKED_EXILE: &str = "spell cost:card types shared with cards exiled with ~";

struct SharedCardTypes;

impl KeywordRules for SharedCardTypes {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name != SHARED_WITH_LINKED_EXILE {
            return None;
        }
        let Some(spell) = ctx.event.as_ref().and_then(|e| e.spell) else {
            return Some(0);
        };
        let exiled = Filter::and(vec![
            Filter::Card,
            Filter::In(Box::new(Sel::Linked)),
            Filter::InZone(ZoneKind::Exile),
        ]);
        let mut among = CardTypeSet::NONE;
        for o in g.objects_matching(&exiled, ctx) {
            among = among.union(g.obj(o).chars.card_types);
        }
        let mine = g
            .try_obj(spell)
            .map_or(CardTypeSet::NONE, |o| o.chars.card_types);
        Some(mine.iter().filter(|t| among.contains(*t)).count() as i64)
    }
}

inventory::submit! { KeywordRegistration(&SharedCardTypes) }
