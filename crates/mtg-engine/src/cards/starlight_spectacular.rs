//! Starlight Spectacular: "Parade! — At the beginning of combat on your turn, choose
//! creatures you control one at a time until each creature you control has been chosen.
//! Each of those creatures gets +1/+1 until end of turn for each creature chosen before
//! it." Each Starlight Spectacular lines them up separately (ruling).

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{Entity, ObjectId};

const PARADE: &str = "card:Starlight Spectacular:line up your creatures";
const TEXT: &str = "Parade! — At the beginning of combat on your turn, choose creatures you control one at a time until each creature you control has been chosen. Each of those creatures gets +1/+1 until end of turn for each creature chosen before it.";
const IN_LINE: Var = vars::USER + 4440;

inventory::submit! { ManualAbility {
    card: "Starlight Spectacular",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "At the beginning of combat on your turn, draw a card.")
            .iter()
            .map(|a| map_effect(a, TEXT, |_| Effect::Custom(PARADE.into())))
            .collect()
    },
    reason: "creatures chosen one at a time, each pumped by the number chosen before it: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != PARADE {
            return false;
        }
        let you = ctx.controller;
        let creatures: Vec<ObjectId> = g
            .permanents_controlled_by(you)
            .into_iter()
            .filter(|o| g.obj(*o).is_creature())
            .collect();
        let names = creatures.iter().map(|c| g.describe(*c)).collect();
        let order = g.ask_order(you, "Line up your creatures (first gets +0/+0)", names);
        for (i, k) in order.into_iter().enumerate() {
            if i == 0 {
                continue;
            }
            ctx.set_var(IN_LINE, vec![Entity::Object(creatures[k])]);
            let pump = Effect::Modify {
                what: Sel::Var(IN_LINE),
                mods: vec![Modification::ModifyPT(Value::Const(i as i32), Value::Const(i as i32))],
                duration: Duration::EndOfTurn,
            };
            g.exec(&pump, ctx);
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
