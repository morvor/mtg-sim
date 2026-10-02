//! Opposition Agent: "While an opponent is searching their library, they exile each card
//! they find. You may play those cards for as long as they remain exiled, and you may
//! spend mana as though it were mana of any color to cast them."
//!
//! The found cards are exiled instead of being put where the effect says; the rest of the
//! effect still happens (ruling). With several Opposition Agents, the one that most
//! recently entered applies (its controller controls the searching player, CR 723).
//! The mana permission is that of CR 118.14 (mana of any type), which includes "any
//! color".

use super::{active, marker, ManualAbility};
use crate::ability::Duration;
use crate::events::MoveCause;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::{ObjectId, PlayerId};

const EXILE_FOUND: &str = "card:Opposition Agent:opponents exile the cards they find";
const TEXT: &str = "While an opponent is searching their library, they exile each card they find. You may play those cards for as long as they remain exiled, and you may spend mana as though it were mana of any color to cast them.";

inventory::submit! { ManualAbility {
    card: "Opposition Agent",
    face: 0,
    text: TEXT,
    build: |_| vec![marker(EXILE_FOUND, TEXT)],
    reason: "opponents exile the cards they find while searching and you may play them: unique search replacement",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn search_found(
        &self,
        g: &mut Game,
        searcher: PlayerId,
        owner: PlayerId,
        found: &[ObjectId],
    ) -> bool {
        if searcher != owner {
            return false;
        }
        let Some((src, you)) = active(g, EXILE_FOUND)
            .into_iter()
            .filter(|(_, c)| g.are_opponents(*c, searcher))
            .max_by_key(|(s, _)| g.obj(*s).timestamp)
        else {
            return false;
        };
        let mut exiled = Vec::new();
        for &card in found {
            if let Some(n) = g.move_object_ev(MoveEv {
                obj: card,
                to: Zone::Exile,
                pos: crate::ability::LibraryPosition::Top,
                cause: MoveCause::Effect,
                by: Some(searcher),
                etb: EtbInfo::default(),
                source: Some(src),
            }) {
                if g.obj(n).zone == Zone::Exile {
                    exiled.push(n);
                }
            }
        }
        // "For as long as they remain exiled": each permission ends when its card leaves
        // exile, as it becomes a new object.
        crate::casting::grant_play_permission(
            g,
            you,
            exiled.clone(),
            Duration::Permanent,
            false,
            Some(src),
        );
        let turn = g.turn.number;
        for n in exiled {
            g.special
                .any_type_mana
                .push((you, n, Duration::Permanent, Some(src), turn));
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
