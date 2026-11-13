//! CR 702.38 Amplify: "Amplify N" means "As this object enters, reveal any number of cards
//! from your hand that share a creature type with it. This permanent enters with N +1/+1
//! counters on it for each card revealed this way. You can't reveal this card or any
//! other cards that are entering the battlefield at the same time as this card."
//! (CR 702.38a). Each instance works separately (CR 702.38b).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;
use smol_str::SmolStr;
use std::collections::BTreeSet;

/// "As this enters, reveal ... amplify N" (the effect's name is followed by N).
const AMPLIFY: &str = "amplify:";

pub struct Amplify;

impl KeywordRules for Amplify {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Amplify]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let n = kw.n.unwrap_or(0);
        let s = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(Filter::Source),
            action: ReplacementAction::AsEnters(Box::new(Effect::Custom(SmolStr::new(
                format!("{AMPLIFY}{n}"),
            )))),
            self_replacement: false,
            optional: false,
        }));
        Some(vec![AbilityDef::new(
            AbilityKind::Static(s),
            format!("Amplify {n}"),
        )])
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some(n) = name.strip_prefix(AMPLIFY) else {
            return false;
        };
        let n: u32 = n.parse().unwrap_or(0);
        let Some(this) = ctx.source else {
            return true;
        };
        let p = ctx.controller;
        let shares = Filter::SharesCreatureType(Box::new(Sel::This));
        let cands: Vec<Entity> = g
            .player(p)
            .hand
            .iter()
            .copied()
            .filter(|c| *c != this && !g.entering.contains(c) && g.matches(*c, &shares, ctx))
            .map(Entity::Object)
            .collect();
        if cands.is_empty() {
            return true;
        }
        let revealed = match g.ask(
            p,
            Decision::ChooseEntities {
                source: Some(this),
                prompt: "Reveal any number of cards that share a creature type with it (amplify)"
                    .into(),
                candidates: cands.clone(),
                min: 0,
                max: cands.len() as u32,
            },
        ) {
            Answer::Entities(v) => {
                let mut seen = BTreeSet::new();
                if v.iter().all(|e| cands.contains(e) && seen.insert(*e)) {
                    v
                } else {
                    cands.clone()
                }
            }
            // Revealing more cards only adds counters.
            _ => cands.clone(),
        };
        for e in &revealed {
            if let Some(o) = e.object() {
                g.emit(crate::events::Event::Custom {
                    name: "revealed".into(),
                    player: Some(p),
                    obj: Some(o),
                    amount: 0,
                });
            }
        }
        let total = n * revealed.len() as u32;
        if total > 0 {
            if let Some(em) = ctx.entering.as_mut() {
                em.counters.push((SmolStr::new(counters::PLUS1), total));
            }
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Amplify) }
