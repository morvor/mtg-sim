//! Gale, Waterdeep Prodigy: "Whenever you cast an instant or sorcery spell from your hand,
//! you may cast up to one target card of the other type from your graveyard. If a spell
//! cast from your graveyard this way would be put into your graveyard, exile it instead."
//! "The other type": a sorcery card for an instant spell, an instant card for a sorcery.

use super::{parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{CardType, ObjectId};

const OTHER_TYPE: &str = "card:Gale, Waterdeep Prodigy:a card of the other type";
const TEXT: &str = "Whenever you cast an instant or sorcery spell from your hand, you may cast up to one target card of the other type from your graveyard. If a spell cast from your graveyard this way would be put into your graveyard, exile it instead.";

inventory::submit! { ManualAbility {
    card: "Gale, Waterdeep Prodigy",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Whenever you cast an instant or sorcery spell from your hand, you may cast target instant card from your graveyard. If that spell would be put into your graveyard, exile it instead.")
            .iter()
            .map(|a| {
                let mut kind = a.kind.clone();
                if let AbilityKind::Triggered(t) = &mut kind {
                    if let Some(spec) = t.body.targets.first_mut() {
                        spec.what = TargetKind::Object(Filter::And(vec![
                            Filter::Custom(OTHER_TYPE.into()),
                            Filter::Card,
                            Filter::InZone(ZoneKind::Graveyard),
                            Filter::OwnedBy(PlayerRel::You),
                        ]));
                        spec.min = Value::Const(0);
                        spec.text = "up to one target card of the other type from your graveyard".into();
                    }
                }
                AbilityDef::new(kind, TEXT)
            })
            .collect()
    },
    reason: "\"a card of the other type\" (instant or sorcery, relative to the spell that triggered it): only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != OTHER_TYPE {
            return None;
        }
        let Some(spell) = ctx.event.as_ref().and_then(|e| e.spell.or(e.object)) else {
            return Some(false);
        };
        let s = &g.obj(spell).chars;
        let c = &g.obj(id).chars;
        Some(match (s.is(CardType::Instant), s.is(CardType::Sorcery)) {
            (true, false) => c.is(CardType::Sorcery),
            (false, true) => c.is(CardType::Instant),
            _ => false,
        })
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
