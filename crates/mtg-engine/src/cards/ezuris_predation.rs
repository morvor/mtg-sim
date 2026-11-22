//! Ezuri's Predation: "For each creature your opponents control, create a 4/4 green
//! Phyrexian Beast creature token. Each of those tokens fights a different one of those
//! creatures." You choose which Beast fights which creature; extra tokens (from a
//! replacement effect) fight nothing; the fights happen at the same time (rulings).

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;
use crate::types::{CardType, Entity, ObjectId};

const FIGHT: &str = "card:Ezuri's Predation:each token fights a different one of those creatures";
const PREY: Var = vars::USER + 4410;
const TEXT: &str = "For each creature your opponents control, create a 4/4 green Phyrexian Beast creature token. Each of those tokens fights a different one of those creatures.";

fn opposing_creatures() -> Filter {
    Filter::And(vec![
        Filter::Type(CardType::Creature),
        Filter::ControlledBy(PlayerRel::Opponent),
    ])
}

inventory::submit! { ManualAbility {
    card: "Ezuri's Predation",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "For each creature your opponents control, create a 4/4 green Phyrexian Beast creature token.")
            .iter()
            .map(|a| {
                map_effect(a, TEXT, |e| {
                    Effect::Seq(vec![
                        Effect::Store {
                            var: PREY,
                            sel: Sel::All(opposing_creatures()),
                        },
                        e,
                        Effect::Custom(FIGHT.into()),
                    ])
                })
            })
            .collect()
    },
    reason: "each created token fights a different one of the creatures counted: only this card",
} }

struct Rules;

fn on_battlefield_creature(g: &Game, id: ObjectId) -> bool {
    g.is_live(id) && g.obj(id).zone == Zone::Battlefield && g.obj(id).is_creature()
}

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != FIGHT {
            return false;
        }
        let prey: Vec<ObjectId> = ctx
            .vars
            .get(&PREY)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(Entity::object)
            .collect();
        let mut tokens: Vec<ObjectId> = ctx
            .vars
            .get(&vars::CREATED)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(Entity::object)
            .collect();
        // You choose which Beast fights which creature.
        let mut pairs = Vec::new();
        for c in prey {
            if tokens.is_empty() {
                break;
            }
            let pick = g.ask_objects(
                ctx.controller,
                ctx.source,
                &format!("Choose a Beast to fight {}", g.describe(c)),
                tokens.clone(),
                1,
                1,
            );
            let t = pick.first().copied().unwrap_or(tokens[0]);
            tokens.retain(|x| *x != t);
            pairs.push((t, c));
        }
        // CR 701.14a-b: each deals damage equal to its power to the other, all at once;
        // a fight with a creature that isn't on the battlefield deals no damage.
        let mut events = Vec::new();
        for (t, c) in pairs {
            if !on_battlefield_creature(g, t) || !on_battlefield_creature(g, c) {
                continue;
            }
            let (pt, pc) = (g.obj(t).power().max(0) as u32, g.obj(c).power().max(0) as u32);
            if pt > 0 {
                events.push((t, Entity::Object(c), pt));
            }
            if pc > 0 {
                events.push((c, Entity::Object(t), pc));
            }
        }
        if !events.is_empty() {
            g.deal_damage_batch(events, false);
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
