//! Opposition Agent: "While an opponent is searching their library, they exile each card
//! they find. You may play those cards for as long as they remain exiled, and you may
//! spend mana as though it were mana of any color to cast them."
//!
//! The found cards are exiled instead of being put where the effect says; the rest of the
//! effect still happens (ruling). With several Opposition Agents controlled by different
//! players, the controller of the one that most recently entered controls the searching
//! player (CR 723) and, for them, chooses whose Agent's effect applies to each found card
//! (CR 616.1; ruling): by default their own.
//! Mana may be spent as though it were mana of any color (CR 609.4b), not of any type:
//! colorless symbols still need colorless mana.

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
        // The Agents whose effect would apply, the most recently entered first.
        let mut agents: Vec<(ObjectId, PlayerId)> = active(g, EXILE_FOUND)
            .into_iter()
            .filter(|(_, c)| g.are_opponents(*c, searcher))
            .collect();
        if agents.is_empty() {
            return false;
        }
        agents.sort_by_key(|(s, _)| std::cmp::Reverse(g.obj(*s).timestamp));
        // One Agent per controller (the latest): which player gets the permissions.
        let mut options: Vec<(ObjectId, PlayerId)> = Vec::new();
        for a in &agents {
            if !options.iter().any(|(_, c)| *c == a.1) {
                options.push(*a);
            }
        }
        // Each found card's owner chooses which of the Agents' replacement effects applies
        // to it (CR 616.1); the owner is the searcher, controlled during the search by the
        // latest Agent's controller, who makes that choice (ruling, CR 723.5). Default: the
        // latest Agent's (the first listed).
        let chooser = options[0].1;
        for &card in found {
            let i = if options.len() > 1 {
                let names = options
                    .iter()
                    .map(|(_, c)| format!("P{}'s Opposition Agent", c.idx()))
                    .collect();
                g.ask_option(
                    chooser,
                    Some(options[0].0),
                    "Which Opposition Agent's effect applies to the found card?",
                    names,
                )
            } else {
                0
            };
            let (src, you) = options[i.min(options.len() - 1)];
            let Some(n) = g.move_object_ev(MoveEv {
                obj: card,
                to: Zone::Exile,
                pos: crate::ability::LibraryPosition::Top,
                cause: MoveCause::Effect,
                by: Some(searcher),
                etb: EtbInfo::default(),
                source: Some(src),
            }) else {
                continue;
            };
            if g.obj(n).zone != Zone::Exile {
                continue;
            }
            // "For as long as they remain exiled": each permission ends when its card
            // leaves exile, as it becomes a new object.
            crate::casting::grant_play_permission(
                g,
                you,
                vec![n],
                Duration::Permanent,
                false,
                Some(src),
            );
            let turn = g.turn.number;
            g.special
                .any_color_mana
                .push((you, n, Duration::Permanent, Some(src), turn));
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
