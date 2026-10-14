//! Mana Maze: "Players can't cast spells that share a color with the spell most recently
//! cast this turn." (CR 601.3: a rule prohibiting casting a spell).

use super::{active, marker, ManualAbility};
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Characteristics;
use crate::types::{ObjectId, PlayerId};

const MAZE: &str = "card:Mana Maze:can't cast spells sharing a color with the last spell";
/// One row `[turn, colors]`: the colors of the spell most recently cast this turn, as it
/// was cast.
const LAST: &str = "card:Mana Maze:last spell cast";
const TEXT: &str = "Players can't cast spells that share a color with the spell most recently cast this turn.";

inventory::submit! { ManualAbility {
    card: "Mana Maze",
    face: 0,
    text: TEXT,
    build: |_| vec![marker(MAZE, TEXT)],
    reason: "can't cast spells sharing a color with the spell most recently cast this turn: unique",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn on_event(&self, g: &mut Game, ev: &Event) {
        if let Event::SpellCast { spell, .. } = ev {
            let colors = g.obj(*spell).chars.colors.0 as i64;
            let turn = g.turn.number as i64;
            g.cards.rows.insert(LAST.to_string(), vec![vec![turn, colors]]);
        }
    }
    fn cast_prohibited(
        &self,
        g: &Game,
        _p: PlayerId,
        _card: ObjectId,
        chars: &Characteristics,
    ) -> bool {
        let Some(last) = g.cards.get(LAST).first() else {
            return false;
        };
        last[0] == g.turn.number as i64
            && chars.colors.0 as i64 & last[1] != 0
            && !active(g, MAZE).is_empty()
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
