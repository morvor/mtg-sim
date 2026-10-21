//! Susan Foreman: "If you would planeswalk, instead look at the top two cards of your
//! planar deck, put one on the bottom of your planar deck and the other on top, then
//! planeswalk." (CR 701.31, 901)

use super::{active, marker, ManualAbility};
use crate::decision::{Answer, Decision};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{Entity, PlayerId};

const SUSAN: &str = "card:Susan Foreman:look at the top two cards of your planar deck before planeswalking";
const TEXT: &str = "If you would planeswalk, instead look at the top two cards of your planar deck, put one on the bottom of your planar deck and the other on top, then planeswalk.";

inventory::submit! { ManualAbility {
    card: "Susan Foreman",
    face: 0,
    text: TEXT,
    build: |_ctx| vec![marker(SUSAN, TEXT)],
    reason: "a replacement of planeswalking: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn before_planeswalk(&self, g: &mut Game, p: PlayerId) {
        // Each such effect applies once to the planeswalk (CR 614.5).
        for (src, _) in active(g, SUSAN).into_iter().filter(|(_, c)| *c == p) {
            let top: Vec<_> = crate::planechase::planar_deck(g, p)
                .into_iter()
                .take(2)
                .collect();
            if top.len() < 2 {
                continue;
            }
            let ans = g.ask(
                p,
                Decision::ChooseEntities {
                    source: Some(src),
                    prompt: "Put a card on the bottom of your planar deck".into(),
                    candidates: top.iter().map(|c| Entity::Object(*c)).collect(),
                    min: 1,
                    max: 1,
                },
            );
            // Default: keep the top card on top.
            let bottom = match ans {
                Answer::Entities(es) => es
                    .first()
                    .and_then(|e| e.object())
                    .filter(|o| top.contains(o))
                    .unwrap_or(top[1]),
                _ => top[1],
            };
            // The planar deck is the face-down planar cards of the command zone, top first.
            g.command.retain(|c| *c != bottom);
            g.command.push(bottom);
            g.dirty = true;
        }
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
