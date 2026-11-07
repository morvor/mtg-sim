//! Gluntch, the Bestower: "At the beginning of your end step, choose a player. They put two
//! +1/+1 counters on a creature they control. Choose a second player to draw a card. Then
//! choose a third player to create two Treasure tokens." The three players are different
//! (any one may be you); with two players, no third is chosen (rulings).

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{CardType, Entity};

const CHOOSE: &str = "card:Gluntch, the Bestower:choose player ";
const CHOSEN: [Var; 3] = [vars::USER + 4420, vars::USER + 4421, vars::USER + 4422];
const TEXT: &str = "At the beginning of your end step, choose a player. They put two +1/+1 counters on a creature they control. Choose a second player to draw a card. Then choose a third player to create two Treasure tokens.";

/// The effect of a triggered ability with the instruction `text`.
fn instruction(ctx: &crate::oracle::CompileContext, text: &str) -> Option<Effect> {
    parse(ctx, &format!("At the beginning of your end step, {text}"))
        .iter()
        .find_map(|a| match &a.kind {
            AbilityKind::Triggered(t) => Some(t.body.effect.clone()),
            _ => None,
        })
}

inventory::submit! { ManualAbility {
    card: "Gluntch, the Bestower",
    face: 0,
    text: TEXT,
    build: |ctx| {
        let Some(treasures) = instruction(ctx, "create two Treasure tokens.") else {
            return parse(ctx, "create two Treasure tokens.");
        };
        let counters = Effect::AddCounters {
            what: Sel::Choose {
                chooser: PlayerRef::You,
                filter: Filter::And(vec![
                    Filter::Type(CardType::Creature),
                    Filter::ControlledBy(PlayerRel::You),
                ]),
                count: Value::Const(1),
                up_to: false,
                store: None,
            },
            kind: "+1/+1".into(),
            n: Value::Const(2),
        };
        let draw = Effect::Draw {
            who: PlayerRef::You,
            n: Value::Const(1),
        };
        let mut steps = Vec::new();
        for (i, e) in [counters, draw, treasures].into_iter().enumerate() {
            steps.push(Effect::Custom(format!("{CHOOSE}{i}").into()));
            steps.push(Effect::AsPlayer {
                who: PlayerRef::Var(CHOSEN[i]),
                effect: Box::new(e),
            });
        }
        parse(ctx, "At the beginning of your end step, draw a card.")
            .iter()
            .map(|a| map_effect(a, TEXT, |_| Effect::Seq(steps.clone())))
            .collect()
    },
    reason: "three successive choices of different players for three different effects: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some(i) = name.strip_prefix(CHOOSE).and_then(|n| n.parse::<usize>().ok()) else {
            return false;
        };
        let taken: Vec<Entity> = CHOSEN[..i.min(3)]
            .iter()
            .flat_map(|v| ctx.vars.get(v).cloned().unwrap_or_default())
            .collect();
        let candidates: Vec<Entity> = g
            .players_in_game()
            .into_iter()
            .map(Entity::Player)
            .filter(|e| !taken.contains(e))
            .collect();
        let prompt = ["Choose a player to put two +1/+1 counters on a creature they control",
            "Choose a second player to draw a card",
            "Choose a third player to create two Treasure tokens"][i.min(2)];
        let chosen = g.ask_entities(ctx.controller, ctx.source, prompt, candidates, 1, 1);
        ctx.set_var(CHOSEN[i.min(2)], chosen);
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
