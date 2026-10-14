//! Duplicant: "As long as a card exiled with ~ is a creature card, ~ has the power,
//! toughness, and creature types of the last creature card exiled with it. It's still a
//! Shapeshifter." (CR 607.2a: the cards its imprint ability exiled.)
//!
//! Its base power and toughness are set (layer 7b, CR 613.4b), so counters and other
//! effects still modify them; the creature types replace its others in layer 4. Both are
//! constantly updated from the exiled card's current characteristics (rulings).

use super::ManualAbility;
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::{Characteristics, Zone};
use crate::types::{is_creature_type, CardType, ObjectId};

const HAS_CREATURE: &str = "card:Duplicant:a card exiled with it is a creature card";
const TYPES: &str = "card:Duplicant:creature types of the last creature card exiled with it";
const POWER: &str = "card:Duplicant:power of the last creature card exiled with it";
const TOUGHNESS: &str = "card:Duplicant:toughness of the last creature card exiled with it";
const TEXT: &str = "As long as a card exiled with ~ is a creature card, ~ has the power, toughness, and creature types of the last creature card exiled with it. It's still a Shapeshifter.";

inventory::submit! { ManualAbility {
    card: "Duplicant",
    face: 0,
    text: TEXT,
    build: |_| {
        let mut s = StaticAbility::new(StaticEffect::Continuous {
            affected: Filter::Source,
            mods: vec![
                Modification::Custom { name: TYPES.into(), layer: Layer::L4Type },
                Modification::SetPT(
                    Some(Value::Custom(POWER.into())),
                    Some(Value::Custom(TOUGHNESS.into())),
                ),
            ],
        });
        s.condition = Some(Condition::Custom(HAS_CREATURE.into()));
        vec![AbilityDef::new(AbilityKind::Static(s), TEXT)]
    },
    reason: "power, toughness and creature types of the last creature card exiled with it: unique last-exiled tracking",
} }

/// The last creature card exiled with `src` that's still in exile.
fn last_creature_card(g: &Game, src: ObjectId) -> Option<ObjectId> {
    g.obj(src)
        .linked
        .values()
        .flatten()
        .copied()
        .filter(|o| {
            let ob = g.obj(*o);
            ob.next.is_none() && ob.zone == Zone::Exile && ob.chars.card_types.contains(CardType::Creature)
        })
        .last()
}

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        (name == HAS_CREATURE).then(|| ctx.source.and_then(|s| last_creature_card(g, s)).is_some())
    }
    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        let power = match name {
            POWER => true,
            TOUGHNESS => false,
            _ => return None,
        };
        let card = ctx.source.and_then(|s| last_creature_card(g, s))?;
        let o = g.obj(card);
        Some(if power { o.power() } else { o.toughness() } as i64)
    }
    fn custom_modification(
        &self,
        g: &Game,
        name: &str,
        chars: &mut Characteristics,
        ctx: &Ctx,
        _target: ObjectId,
    ) -> bool {
        if name != TYPES {
            return false;
        }
        let Some(card) = ctx.source.and_then(|s| last_creature_card(g, s)) else {
            return true;
        };
        let from = &g.obj(card).chars;
        chars.subtypes.retain(|s| !is_creature_type(s));
        for s in from.subtypes.iter().filter(|s| is_creature_type(s)) {
            if !chars.subtypes.contains(s) {
                chars.subtypes.push(s.clone());
            }
        }
        if !chars.subtypes.iter().any(|s| s == "Shapeshifter") {
            chars.subtypes.push("Shapeshifter".into());
        }
        chars.all_creature_types = from.all_creature_types;
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
