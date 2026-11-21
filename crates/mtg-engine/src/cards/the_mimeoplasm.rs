//! The Mimeoplasm: "As ~ enters, you may exile two creature cards from graveyards. If you
//! do, it enters as a copy of one of those cards with a number of additional +1/+1
//! counters on it equal to the power of the other card." Exactly two (from any
//! graveyards); you choose which is copied; counters the copied card enters with are kept
//! too (rulings).

use super::ManualAbility;
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::{counters, ObjectId};

const MIMIC: &str = "card:The Mimeoplasm:exile two creature cards and enter as a copy of one";
const TEXT: &str = "As ~ enters, you may exile two creature cards from graveyards. If you do, it enters as a copy of one of those cards with a number of additional +1/+1 counters on it equal to the power of the other card.";

inventory::submit! { ManualAbility {
    card: "The Mimeoplasm",
    face: 0,
    text: TEXT,
    build: |_| {
        vec![AbilityDef::new(
            AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
                event: ReplacementEvent::EntersBattlefield(Filter::Source),
                action: ReplacementAction::AsEnters(Box::new(Effect::Custom(MIMIC.into()))),
                self_replacement: false,
                optional: false,
            }))),
            TEXT,
        )]
    },
    reason: "enters as a copy of one exiled card with counters equal to the other's power: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != MIMIC {
            return false;
        }
        let you = ctx.controller;
        let cands: Vec<ObjectId> = g
            .players
            .iter()
            .flat_map(|p| p.graveyard.iter().copied())
            .filter(|c| g.obj(*c).chars.is_creature())
            .collect();
        if cands.len() < 2
            || !g.ask_yes_no(you, ctx.source, "Exile two creature cards from graveyards?", true)
        {
            return true;
        }
        let two = g.ask_objects(you, ctx.source, "Choose two creature cards", cands, 2, 2);
        if two.len() != 2 {
            return true;
        }
        let moves = two
            .iter()
            .map(|c| MoveEv {
                obj: *c,
                to: Zone::Exile,
                pos: LibraryPosition::Top,
                cause: MoveCause::Exile,
                by: Some(you),
                etb: EtbInfo::default(),
                source: ctx.source,
            })
            .collect();
        let exiled: Vec<ObjectId> = g
            .move_objects(moves)
            .into_iter()
            .flatten()
            .filter(|c| g.obj(*c).zone == Zone::Exile)
            .collect();
        if exiled.len() != 2 {
            return true;
        }
        if g.dirty {
            g.recompute();
        }
        let copy = g
            .ask_objects(you, ctx.source, "Choose the card to copy", exiled.clone(), 1, 1)
            .first()
            .copied()
            .unwrap_or(exiled[0]);
        let other = exiled.into_iter().find(|c| *c != copy).unwrap_or(copy);
        let power = g.obj(other).power().max(0) as u32;
        if let Some(em) = ctx.entering.as_mut() {
            em.copy_of = Some(copy);
            if power > 0 {
                em.counters.push((counters::PLUS1.into(), power));
            }
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
