//! Noxious Vapors: "Each player reveals their hand, chooses one card of each color from
//! it, then discards all other nonland cards."

use super::{spell, ManualAbility};
use crate::ability::Effect;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{CardType, Color, ObjectId, PlayerId};

const RESOLVE: &str = "card:Noxious Vapors:keep one card of each color, discard the other nonland cards";
const TEXT: &str = "Each player reveals their hand, chooses one card of each color from it, then discards all other nonland cards.";

inventory::submit! { ManualAbility {
    card: "Noxious Vapors",
    face: 0,
    text: TEXT,
    build: |_| vec![spell(vec![], Effect::Custom(RESOLVE.into()), TEXT)],
    reason: "each player keeps one card of each color and discards the other nonland cards: unique",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != RESOLVE {
            return false;
        }
        // Each player reveals their hand and makes their choices in turn order
        // (CR 101.4); then they discard at the same time.
        let mut discards: Vec<(PlayerId, Vec<ObjectId>)> = Vec::new();
        for p in g.apnap() {
            let hand = g.player(p).hand.clone();
            crate::reveal::reveal(g, p, &hand, ctx.source);
            let mut kept: Vec<ObjectId> = Vec::new();
            for c in Color::ALL {
                let of_color: Vec<ObjectId> = hand
                    .iter()
                    .copied()
                    .filter(|id| g.obj(*id).chars.colors.contains(c))
                    .collect();
                if of_color.is_empty() {
                    continue;
                }
                let chosen = g.ask_objects(
                    p,
                    ctx.source,
                    &format!("Choose a {c:?} card to keep"),
                    of_color,
                    1,
                    1,
                );
                kept.extend(chosen);
            }
            let rest = hand
                .into_iter()
                .filter(|id| !kept.contains(id) && !g.obj(*id).is(CardType::Land))
                .collect();
            discards.push((p, rest));
        }
        for (p, cards) in discards {
            for c in cards {
                g.discard(p, c, ctx.source);
            }
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
