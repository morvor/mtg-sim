//! Fatespinner: "At the beginning of each opponent's upkeep, that player chooses draw
//! step, main phase, or combat phase. The player skips each instance of the chosen step
//! or phase this turn." (CR 614.1b, 614.10).

use super::{map_effect, parse, ManualAbility};
use crate::ability::{Effect, PlayerRef};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::turn::Step;
use crate::types::PlayerId;

const CHOOSE: &str = "card:Fatespinner:that player chooses a step or phase to skip";
/// Rows `[player, turn, choice]`, choice 0 = draw step, 1 = main phase, 2 = combat phase.
const SKIPS: &str = "card:Fatespinner:chosen skips";
const TEXT: &str = "At the beginning of each opponent's upkeep, that player chooses draw step, main phase, or combat phase. The player skips each instance of the chosen step or phase this turn.";

inventory::submit! { ManualAbility {
    card: "Fatespinner",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "At the beginning of each opponent's upkeep, that player draws a card.")
            .iter()
            .map(|a| map_effect(a, TEXT, |_| Effect::Custom(CHOOSE.into())))
            .collect()
    },
    reason: "an opponent chooses a step or phase to skip each instance of this turn: unique",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != CHOOSE {
            return false;
        }
        let Some(p) = g.eval_player(&PlayerRef::TriggerPlayer, ctx) else {
            return true;
        };
        let choice = g.ask_option(
            p,
            ctx.source,
            "Choose a step or phase to skip this turn",
            vec![
                "draw step".to_string(),
                "main phase".to_string(),
                "combat phase".to_string(),
            ],
        );
        let turn = g.turn.number as i64;
        g.cards.retain(SKIPS, |r| r[1] == turn);
        g.cards.push(SKIPS, vec![p.0 as i64, turn, choice as i64]);
        g.log(|_| format!("{p} chooses to skip the {}", ["draw step", "main phase", "combat phase"][choice]));
        true
    }

    /// Each instance this turn, even one added after the ability resolved.
    fn skips_step(&self, g: &Game, step: Step, active: PlayerId) -> bool {
        let turn = g.turn.number as i64;
        g.cards
            .get(SKIPS)
            .iter()
            .filter(|r| r[0] == active.0 as i64 && r[1] == turn)
            .any(|r| match r[2] {
                0 => step == Step::Draw,
                1 => step.is_main(),
                _ => step.is_combat(),
            })
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
