//! Snowblind: "Enchanted creature gets -X/-Y. If that creature is attacking, X is the number
//! of snow lands defending player controls. Otherwise, X is the number of snow lands its
//! controller controls. Y is equal to X or to enchanted creature's toughness minus 1,
//! whichever is smaller."

use super::{parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{CardType, Entity, PlayerId, Supertype};

const MINUS_X: &str = "card:Snowblind:-X";
const MINUS_Y: &str = "card:Snowblind:-Y";
const TEXT: &str = "Enchanted creature gets -X/-Y. If that creature is attacking, X is the number of snow lands defending player controls. Otherwise, X is the number of snow lands its controller controls. Y is equal to X or to enchanted creature's toughness minus 1, whichever is smaller.";

inventory::submit! { ManualAbility {
    card: "Snowblind",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Enchanted creature gets -1/-1.")
            .iter()
            .map(|a| {
                let mut kind = a.kind.clone();
                if let AbilityKind::Static(s) = &mut kind {
                    if let StaticEffect::Continuous { mods, .. } = &mut s.effect {
                        *mods = vec![Modification::ModifyPT(
                            Value::Custom(MINUS_X.into()),
                            Value::Custom(MINUS_Y.into()),
                        )];
                    }
                }
                AbilityDef::new(kind, TEXT)
            })
            .collect()
    },
    reason: "-X/-Y with X from the defending player's or its controller's snow lands and Y keeping toughness at least 1: only this card",
} }

struct Rules;

fn snow_lands(g: &Game, p: PlayerId) -> i64 {
    g.permanents()
        .filter(|o| o.controller == p && o.is(CardType::Land) && o.chars.supertypes.contains(Supertype::Snow))
        .count() as i64
}

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name != MINUS_X && name != MINUS_Y {
            return None;
        }
        // The creature the effect is being applied to.
        let c = ctx
            .vars
            .get(&vars::AFFECTED)
            .and_then(|v| v.first())
            .and_then(|e| e.object())
            .or_else(|| match ctx.source.and_then(|s| g.obj(s).attached_to) {
                Some(Entity::Object(o)) => Some(o),
                _ => None,
            })?;
        let who = g
            .combat
            .as_ref()
            .and_then(|cs| cs.defending_player_of(g, c))
            .unwrap_or(g.obj(c).controller);
        let x = snow_lands(g, who);
        Some(if name == MINUS_X {
            -x
        } else {
            -x.min(i64::from(g.obj(c).toughness()) - 1).max(0)
        })
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
