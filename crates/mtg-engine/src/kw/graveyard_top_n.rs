//! "The top N [quality] cards of [whose] graveyard" (CR 404.2: a graveyard's order is
//! fixed; its top card is the one put there most recently): "one of the top two cards of
//! your graveyard" (Phyrexian Grimoire), "exile the top three black cards of your
//! graveyard" (Spinning Darkness). The text is parsed in
//! `oracle/patterns/card_flow_5.rs`; the single "top [type] card" is
//! `kw/graveyard_order.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{Filter, PlayerRel, ZoneKind};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::types::*;
use smol_str::SmolStr;

/// `Filter::Custom` name prefix, followed by "N:quality" (quality: empty, a color word or
/// a card type word): a card among the N topmost cards of that quality in its graveyard.
pub const AMONG_TOP_OF_GRAVEYARD: &str = "among the top cards of its graveyard:";

/// The quality word's filter (None if it isn't one).
pub fn quality(word: &str) -> Option<Filter> {
    if word.is_empty() {
        return Some(Filter::Any);
    }
    if let Some(c) = Color::from_word(word) {
        return Some(Filter::Color(c));
    }
    CardType::from_word(word).map(Filter::Type)
}

/// "the top N [quality] cards of [whose] graveyard".
pub fn top_cards_of_graveyard(n: u32, quality_word: &str, whose: PlayerRel) -> Option<Filter> {
    let q = quality(quality_word)?;
    Some(Filter::and(vec![
        Filter::Card,
        Filter::InZone(ZoneKind::Graveyard),
        Filter::OwnedBy(whose),
        q,
        Filter::Custom(SmolStr::new(format!(
            "{AMONG_TOP_OF_GRAVEYARD}{n}:{quality_word}"
        ))),
    ]))
}

pub struct GraveyardTopN;

impl KeywordRules for GraveyardTopN {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        let (n, word) = name.strip_prefix(AMONG_TOP_OF_GRAVEYARD)?.split_once(':')?;
        let n: usize = n.parse().ok()?;
        let q = quality(word)?;
        let Zone::Graveyard(p) = g.obj(id).zone else {
            return Some(false);
        };
        // The graveyard's last card is its top card.
        Some(
            g.player(p)
                .graveyard
                .iter()
                .rev()
                .filter(|x| g.matches(**x, &q, ctx))
                .take(n)
                .any(|x| *x == id),
        )
    }
}

inventory::submit! { KeywordRegistration(&GraveyardTopN) }
