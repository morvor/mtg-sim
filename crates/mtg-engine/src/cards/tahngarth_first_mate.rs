//! Tahngarth, First Mate: "Whenever an opponent attacks with one or more creatures, if ~ is
//! tapped, you may have that opponent gain control of ~ until end of combat. If you do,
//! choose a player or planeswalker that opponent is attacking. ~ is attacking that player
//! or planeswalker." It becomes an attacking creature without being declared (CR 508.4):
//! attack restrictions don't apply and "whenever ... attacks" doesn't trigger; with nothing
//! to choose, it isn't attacking (rulings).

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{Entity, PlayerId};

const JOIN: &str = "card:Tahngarth, First Mate:join the attacking opponent";
const TEXT: &str = "Whenever an opponent attacks with one or more creatures, if ~ is tapped, you may have that opponent gain control of ~ until end of combat. If you do, choose a player or planeswalker that opponent is attacking. ~ is attacking that player or planeswalker.";

inventory::submit! { ManualAbility {
    card: "Tahngarth, First Mate",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Whenever an opponent attacks with one or more creatures, if ~ is tapped, draw a card.")
            .iter()
            .map(|a| map_effect(a, TEXT, |_| Effect::Custom(JOIN.into())))
            .collect()
    },
    reason: "joins the attacking opponent's side and attack: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != JOIN {
            return false;
        }
        let Some(src) = ctx.source else {
            return true;
        };
        let Some(opp) = ctx.event.as_ref().and_then(|e| e.player) else {
            return true;
        };
        if !g.ask_yes_no(ctx.controller, Some(src), "Have that opponent gain control of it?", false) {
            return true;
        }
        let mut c = ctx.clone();
        c.chosen_player = Some(opp);
        g.exec(
            &Effect::GainControl {
                what: Sel::This,
                who: PlayerRef::Player(opp),
                duration: Duration::EndOfCombat,
            },
            &mut c,
        );
        if g.dirty {
            g.recompute();
        }
        // The players and planeswalkers that opponent's creatures are attacking.
        let mut attacked: Vec<Entity> = Vec::new();
        if let Some(cs) = &g.combat {
            for a in &cs.attackers {
                let ours = g.is_live(a.id) && g.obj(a.id).controller == opp;
                if let (true, Some(t)) = (ours, a.target) {
                    let pw_or_player = match t {
                        Entity::Player(_) => true,
                        Entity::Object(o) => g.obj(o).is(crate::types::CardType::Planeswalker),
                    };
                    if pw_or_player && !attacked.contains(&t) {
                        attacked.push(t);
                    }
                }
            }
        }
        let you: PlayerId = ctx.controller;
        let chosen = g.ask_entities(you, Some(src), "Choose what it attacks", attacked, 1, 1);
        if let Some(t) = chosen.first() {
            crate::combat::make_attacking(g, src, *t);
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
