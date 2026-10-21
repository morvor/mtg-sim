//! Retether: "Return each Aura card from your graveyard to the battlefield. Only creatures
//! can be enchanted this way." Each Aura that could enchant a creature on the battlefield
//! returns attached to a creature you choose; the others stay. Whether an Aura can be
//! attached is checked before any of them return, and they return at once (rulings).

use super::{spell, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::{CardType, Entity, ObjectId};

const RETURN_AURAS: &str = "card:Retether:return each Aura card to a creature";
const TEXT: &str = "Return each Aura card from your graveyard to the battlefield. Only creatures can be enchanted this way.";

inventory::submit! { ManualAbility {
    card: "Retether",
    face: 0,
    text: TEXT,
    build: |_| vec![spell(vec![], Effect::Custom(RETURN_AURAS.into()), TEXT)],
    reason: "returns Auras only onto creatures; an Aura that can't enchant one stays (CR 303.4f-g): only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != RETURN_AURAS {
            return false;
        }
        let you = ctx.controller;
        let auras: Vec<ObjectId> = g
            .player(you)
            .graveyard
            .iter()
            .copied()
            .filter(|c| g.obj(*c).chars.has_subtype("Aura"))
            .collect();
        let creatures: Vec<ObjectId> = g
            .permanents()
            .filter(|o| o.is(CardType::Creature))
            .map(|o| o.id)
            .collect();
        let mut moves = Vec::new();
        for a in auras {
            let ok: Vec<ObjectId> = creatures
                .iter()
                .copied()
                .filter(|c| crate::attach::can_attach(g, a, Entity::Object(*c)))
                .collect();
            let prompt = format!("Choose a creature for {} to enchant", g.describe(a));
            let Some(c) = g.ask_objects(you, ctx.source, &prompt, ok, 1, 1).first().copied() else {
                continue;
            };
            moves.push(MoveEv {
                obj: a,
                to: Zone::Battlefield,
                pos: LibraryPosition::Top,
                cause: MoveCause::Return,
                by: Some(you),
                etb: EtbInfo {
                    controller: Some(you),
                    attach_to: Some(Entity::Object(c)),
                    attach_specified: true,
                    ..Default::default()
                },
                source: ctx.source,
            });
        }
        if !moves.is_empty() {
            g.move_objects(moves);
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
