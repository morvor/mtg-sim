//! Aeon Engine: "{T}, Exile ~: Reverse the game's turn order." After it resolves, play
//! proceeds the other way around the table: the next turn, APNAP order (CR 101.4) and
//! every "next player in turn order" go to the right instead of to the left
//! ([`Game::seat_in_turn_order`]). Seats don't move, so "the player to your left" is
//! still the same player. Reversing it again restores the original direction. With two
//! players the order is the same either way (CR 102.2).

use super::{map_effect, parse, ManualAbility};
use crate::ability::Effect;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};

const REVERSE: &str = "card:Aeon Engine:reverse the game's turn order";
const TEXT: &str = "{T}, Exile ~: Reverse the game's turn order.";

inventory::submit! { ManualAbility {
    card: "Aeon Engine",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "{T}, Exile ~: You gain 1 life.")
            .iter()
            .map(|a| map_effect(a, TEXT, |_| Effect::Custom(REVERSE.into())))
            .collect()
    },
    reason: "reverses the game's turn order: the only legal card that does",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, _ctx: &mut Ctx) -> bool {
        if name != REVERSE {
            return false;
        }
        g.turn_order_reversed = !g.turn_order_reversed;
        g.log(|g| {
            format!(
                "Turn order reversed: play now proceeds to the {}",
                if g.turn_order_reversed { "right" } else { "left" }
            )
        });
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
