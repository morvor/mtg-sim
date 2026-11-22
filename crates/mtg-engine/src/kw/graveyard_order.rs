//! The order of cards in a graveyard (CR 404.2): "the top creature card of your graveyard"
//! is the creature card put there most recently (Barrow Ghoul, Circling Vultures, Mistmoon
//! Griffin, Bone Dancer). A graveyard's order can't be changed (CR 404.2); cards put into
//! it at the same time are ordered by their owner (CR 404.3). The text is parsed in
//! `oracle/patterns/r404_graveyard_order.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{Filter, PlayerRel, ZoneKind};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::types::*;
use smol_str::SmolStr;

/// `Filter::Custom` name prefix, followed by a card type: a card of that type in a
/// graveyard with no card of that type above it.
pub const TOPMOST_IN_GRAVEYARD: &str = "topmost in its graveyard:";

/// "the top [type] card of [whose] graveyard".
pub fn top_card_of_graveyard(t: CardType, whose: PlayerRel) -> Filter {
    Filter::and(vec![
        Filter::InZone(ZoneKind::Graveyard),
        Filter::OwnedBy(whose),
        Filter::Type(t),
        Filter::Custom(SmolStr::new(format!("{TOPMOST_IN_GRAVEYARD}{}", t.word()))),
    ])
}

pub struct GraveyardOrder;

impl KeywordRules for GraveyardOrder {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        let t = CardType::from_word(name.strip_prefix(TOPMOST_IN_GRAVEYARD)?)?;
        let o = g.obj(id);
        let Zone::Graveyard(p) = o.zone else {
            return Some(false);
        };
        if !o.is(t) {
            return Some(false);
        }
        let gy = &g.player(p).graveyard;
        let Some(i) = gy.iter().position(|x| *x == id) else {
            return Some(false);
        };
        // The graveyard's last card is its top card.
        Some(!gy[i + 1..].iter().any(|x| g.obj(*x).is(t)))
    }
}

inventory::submit! { KeywordRegistration(&GraveyardOrder) }
