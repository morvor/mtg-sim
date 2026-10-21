//! Ashiok, Wicked Manipulator: "If you would pay life while your library has at least that
//! many cards in it, exile that many cards from the top of your library instead." A
//! replacement effect of paying life (CR 119.4, 614.1a): not optional, and with fewer cards
//! in your library you pay the life (rulings).

use super::{active, marker, ManualAbility};
use crate::ability::LibraryPosition;
use crate::events::MoveCause;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::PlayerId;

const EXILE_INSTEAD: &str = "card:Ashiok, Wicked Manipulator:exile cards from your library instead of paying life";
const TEXT: &str = "If you would pay life while your library has at least that many cards in it, exile that many cards from the top of your library instead.";

inventory::submit! { ManualAbility {
    card: "Ashiok, Wicked Manipulator",
    face: 0,
    text: TEXT,
    build: |_| vec![marker(EXILE_INSTEAD, TEXT)],
    reason: "paying life is replaced by exiling that many cards from the top of your library: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn pay_life_instead(&self, g: &mut Game, p: PlayerId, n: u32) -> bool {
        let Some((src, _)) = active(g, EXILE_INSTEAD).into_iter().find(|(_, you)| *you == p)
        else {
            return false;
        };
        let lib = &g.player(p).library;
        if (lib.len() as u32) < n {
            return false;
        }
        let top: Vec<_> = lib.iter().rev().take(n as usize).copied().collect();
        let moves = top
            .into_iter()
            .map(|c| MoveEv {
                obj: c,
                to: Zone::Exile,
                pos: LibraryPosition::Top,
                cause: MoveCause::Exile,
                by: Some(p),
                etb: EtbInfo::default(),
                source: Some(src),
            })
            .collect();
        g.move_objects(moves);
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
