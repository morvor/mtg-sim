//! Raging River: "Whenever one or more creatures you control attack, each defending player
//! divides all creatures without flying they control into a "left" pile and a "right"
//! pile. Then, for each attacking creature you control, choose "left" or "right." That
//! creature can't be blocked this combat except by creatures with flying and creatures in a
//! pile with the chosen label." Creatures that weren't in a pile (they entered later) can't
//! block those attackers; attackers that join the combat later aren't affected (rulings).

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{ObjectId, PlayerId};

const SIDES: &str = "card:Raging River:left and right piles";
const LABELS: &str = "card:Raging River:labels";
const TEXT: &str = "Whenever one or more creatures you control attack, each defending player divides all creatures without flying they control into a \"left\" pile and a \"right\" pile. Then, for each attacking creature you control, choose \"left\" or \"right.\" That creature can't be blocked this combat except by creatures with flying and creatures in a pile with the chosen label.";

inventory::submit! { ManualAbility {
    card: "Raging River",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Whenever one or more creatures you control attack, draw a card.")
            .iter()
            .map(|a| map_effect(a, TEXT, |_| Effect::Custom(SIDES.into())))
            .collect()
    },
    reason: "defenders split into left and right piles; each attacker can be blocked only from its chosen side: only this card",
} }

fn combat_key(g: &Game) -> i64 {
    i64::from(g.turn.number) * 100 + i64::from(g.turn.combat_phases)
}

fn ask_side(g: &mut Game, p: PlayerId, o: ObjectId, src: Option<ObjectId>) -> i64 {
    let prompt = format!("{}: left or right?", g.describe(o));
    g.ask_option(p, src, &prompt, vec!["Left".into(), "Right".into()]) as i64
}

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    /// Records (combat, creature, side): the piles, then the attackers' labels.
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != SIDES {
            return false;
        }
        let key = combat_key(g);
        g.cards.retain(SIDES, |r| r[0] == key);
        g.cards.retain(LABELS, |r| r[0] == key);
        let you = ctx.controller;
        let Some(cs) = g.combat.clone() else {
            return true;
        };
        for p in g.apnap() {
            if !cs.defending_players.contains(&p) {
                continue;
            }
            let ground: Vec<ObjectId> = g
                .permanents_controlled_by(p)
                .into_iter()
                .filter(|o| {
                    g.obj(*o).is_creature()
                        && !g.obj(*o).has_keyword(crate::keywords::KeywordKind::Flying)
                })
                .collect();
            for c in ground {
                let side = ask_side(g, p, c, ctx.source);
                g.cards.push(SIDES, vec![key, i64::from(c.0), side]);
            }
            g.emit(crate::events::Event::Custom {
                name: crate::piles::GROUPED_INTO_PILES.into(),
                player: Some(p),
                obj: ctx.stack_obj.or(ctx.source),
                amount: 0,
            });
        }
        let attackers: Vec<ObjectId> = cs
            .attackers
            .iter()
            .map(|a| a.id)
            .filter(|a| g.is_live(*a) && g.obj(*a).controller == you)
            .collect();
        for a in attackers {
            let side = ask_side(g, you, a, ctx.source);
            g.cards.push(LABELS, vec![key, i64::from(a.0), side]);
        }
        true
    }
    fn block_allowed(&self, g: &Game, blocker: ObjectId, attacker: ObjectId) -> bool {
        let key = combat_key(g);
        let Some(label) = g
            .cards
            .get(LABELS)
            .iter()
            .find(|r| r[0] == key && r[1] == i64::from(attacker.0))
            .map(|r| r[2])
        else {
            return true;
        };
        g.obj(blocker).has_keyword(crate::keywords::KeywordKind::Flying)
            || g.cards
                .get(SIDES)
                .iter()
                .any(|r| r[0] == key && r[1] == i64::from(blocker.0) && r[2] == label)
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
