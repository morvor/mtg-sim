//! Master of the Wild Hunt: "{T}: Tap all untapped Wolf creatures you control. Each Wolf
//! tapped this way deals damage equal to its power to target creature. That creature deals
//! damage equal to its power divided as its controller chooses among any number of those
//! Wolves." The Wolves are tapped by the effect; the target's controller divides its damage
//! as the ability resolves, among the Wolves tapped this way only (rulings).

use super::ManualAbility;
use crate::ability::*;
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;
use crate::types::{CardType, Entity, ObjectId};

const HUNT: &str = "card:Master of the Wild Hunt:the Wolves and the target deal damage to each other";
const TEXT: &str = "{T}: Tap all untapped Wolf creatures you control. Each Wolf tapped this way deals damage equal to its power to target creature. That creature deals damage equal to its power divided as its controller chooses among any number of those Wolves.";

inventory::submit! { ManualAbility {
    card: "Master of the Wild Hunt",
    face: 0,
    text: TEXT,
    build: |_| {
        let body = Body::simple(
            vec![TargetSpec::object(Filter::Type(CardType::Creature), "target creature")],
            Effect::Custom(HUNT.into()),
        );
        vec![AbilityDef::new(
            AbilityKind::Activated(ActivatedAbility::new(Cost::tap(), body)),
            TEXT,
        )]
    },
    reason: "Wolves tapped by the effect damage the target, which divides its damage back among them: only this card",
} }

struct Rules;

fn on_battlefield(g: &Game, o: ObjectId) -> bool {
    g.is_live(o) && g.obj(o).zone == Zone::Battlefield
}

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != HUNT {
            return false;
        }
        let Some(Entity::Object(target)) = ctx.targets.first().and_then(|v| v.first()).copied()
        else {
            return true;
        };
        let you = ctx.controller;
        let wolves: Vec<ObjectId> = g
            .permanents_controlled_by(you)
            .into_iter()
            .filter(|o| {
                let w = g.obj(*o);
                w.is_creature() && w.chars.has_subtype("Wolf") && !w.tapped
            })
            .collect();
        let tapped: Vec<ObjectId> = wolves.into_iter().filter(|w| g.tap(*w)).collect();
        if g.dirty {
            g.recompute();
        }
        // Each Wolf deals damage equal to its power to the target, all at once.
        let hits: Vec<(ObjectId, Entity, u32)> = tapped
            .iter()
            .filter(|w| on_battlefield(g, **w))
            .map(|w| (*w, Entity::Object(target), g.obj(*w).power().max(0) as u32))
            .filter(|(_, _, n)| *n > 0)
            .collect();
        if !hits.is_empty() && on_battlefield(g, target) {
            g.deal_damage_batch(hits, false);
        }
        if g.dirty {
            g.recompute();
        }
        // Then the target divides damage equal to its power among those Wolves.
        let recipients: Vec<ObjectId> =
            tapped.into_iter().filter(|w| on_battlefield(g, *w)).collect();
        if recipients.is_empty() || !on_battlefield(g, target) {
            return true;
        }
        let power = g.obj(target).power().max(0) as u32;
        if power == 0 {
            return true;
        }
        let chooser = g.obj(target).controller;
        let n = recipients.len();
        let split: Vec<u32> = match g.ask(
            chooser,
            Decision::Divide {
                source: target,
                total: power,
                recipients: recipients.iter().map(|w| Entity::Object(*w)).collect(),
                min_each: 0,
            },
        ) {
            Answer::Numbers(v)
                if v.len() == n && v.iter().all(|x| *x >= 0) && v.iter().sum::<i64>() == power as i64 =>
            {
                v.into_iter().map(|x| x as u32).collect()
            }
            _ => {
                let mut d = vec![0; n];
                d[0] = power;
                d
            }
        };
        let back: Vec<(ObjectId, Entity, u32)> = recipients
            .into_iter()
            .zip(split)
            .filter(|(_, k)| *k > 0)
            .map(|(w, k)| (target, Entity::Object(w), k))
            .collect();
        if !back.is_empty() {
            g.deal_damage_batch(back, false);
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
