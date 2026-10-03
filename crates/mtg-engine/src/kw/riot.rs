//! CR 702.136 Riot: a static ability. "Riot" means "You may have this permanent enter
//! with an additional +1/+1 counter on it. If you don't, it gains haste." (CR 702.136a).
//! It's a replacement effect that modifies how the permanent enters (CR 614.1c, 614.12):
//! the choice is made as it enters, and the haste lasts indefinitely, even if it loses
//! riot or another player gains control of it. Each instance works separately
//! (CR 702.136b): each is its own replacement effect, and each asks.
//!
//! A permanent that can't have a +1/+1 counter put on it gains haste.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;
use smol_str::SmolStr;

/// `Effect::Custom`: riot's choice, made as the permanent enters.
const RIOT: &str = "riot:counter or haste";

pub struct Riot;

impl KeywordRules for Riot {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Riot]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let s = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(Filter::Source),
            action: ReplacementAction::AsEnters(Box::new(Effect::Custom(SmolStr::new(RIOT)))),
            self_replacement: false,
            optional: false,
        }));
        Some(vec![AbilityDef::new(
            AbilityKind::Static(s),
            KeywordKind::Riot.name(),
        )])
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != RIOT {
            return false;
        }
        let Some(this) = ctx.source else {
            return true;
        };
        let p = ctx.controller;
        let counter = !crate::counter_rules::counters_prevented(g, this, counters::PLUS1)
            && g.ask_yes_no(
                p,
                Some(this),
                "Riot: enter with a +1/+1 counter? (Otherwise it gains haste.)",
                true,
            );
        let name = g.obj(this).chars.name.clone();
        if let Some(em) = ctx.entering.as_mut() {
            if counter {
                em.counters.push((SmolStr::new(counters::PLUS1), 1));
            } else {
                em.on_entry.push(Effect::Modify {
                    what: Sel::This,
                    mods: vec![Modification::AddKeyword(Keyword::new(KeywordKind::Haste))],
                    duration: Duration::Permanent,
                });
            }
        }
        g.log(|_| {
            format!(
                "{name}: riot ({})",
                if counter { "+1/+1 counter" } else { "haste" }
            )
        });
        true
    }
}

inventory::submit! { KeywordRegistration(&Riot) }
