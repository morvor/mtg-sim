//! Raiding Party: "Sacrifice an Orc: Each player may tap any number of untapped white
//! creatures they control. For each creature tapped this way, that player chooses up to two
//! Plains. Then destroy all Plains that weren't chosen this way by any player." The
//! creatures are tapped as it resolves; a player may choose fewer Plains (rulings).

use super::{map_effect, parse, ManualAbility};
use crate::ability::Effect;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{CardType, Color, ObjectId};

const RAID: &str = "card:Raiding Party:players save Plains by tapping white creatures";
const TEXT: &str = "Sacrifice an Orc: Each player may tap any number of untapped white creatures they control. For each creature tapped this way, that player chooses up to two Plains. Then destroy all Plains that weren't chosen this way by any player.";

inventory::submit! { ManualAbility {
    card: "Raiding Party",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Sacrifice an Orc: Draw a card.")
            .iter()
            .map(|a| map_effect(a, TEXT, |_| Effect::Custom(RAID.into())))
            .collect()
    },
    reason: "players tap white creatures to save up to two Plains each; the rest are destroyed: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != RAID {
            return false;
        }
        let mut saved: Vec<ObjectId> = Vec::new();
        for p in g.apnap() {
            let white: Vec<ObjectId> = g
                .permanents_controlled_by(p)
                .into_iter()
                .filter(|o| {
                    let c = g.obj(*o);
                    c.is_creature() && !c.tapped && c.chars.colors.contains(Color::White)
                })
                .collect();
            let n = white.len() as u32;
            let chosen = g.ask_objects(p, ctx.source, "Tap white creatures to save Plains", white, 0, n);
            let tapped = chosen.into_iter().filter(|c| g.tap(*c)).count() as u32;
            if tapped == 0 {
                continue;
            }
            let plains: Vec<ObjectId> = g
                .permanents()
                .filter(|o| o.is(CardType::Land) && o.chars.has_subtype("Plains"))
                .map(|o| o.id)
                .collect();
            saved.extend(g.ask_objects(p, ctx.source, "Choose Plains to save", plains, 0, 2 * tapped));
        }
        let doomed: Vec<ObjectId> = g
            .permanents()
            .filter(|o| o.is(CardType::Land) && o.chars.has_subtype("Plains") && !saved.contains(&o.id))
            .map(|o| o.id)
            .collect();
        g.destroy_all(doomed, ctx.source, false);
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
