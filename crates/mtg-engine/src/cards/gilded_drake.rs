//! Gilded Drake: "When ~ enters, exchange control of ~ and up to one target creature an
//! opponent controls. If you don't or can't make an exchange, sacrifice ~. This ability
//! still resolves if its target becomes illegal." The last sentence is an exception to
//! CR 608.2b: with its target illegal, the exchange can't be made and the Drake is
//! sacrificed (rulings).

use super::{map_effect, parse, ManualAbility};
use crate::ability::{Body, Effect};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::ObjectId;

const STILL_RESOLVES: &str = "card:Gilded Drake:still resolves if its target becomes illegal";
const TEXT: &str = "When ~ enters, exchange control of ~ and up to one target creature an opponent controls. If you don't or can't make an exchange, sacrifice ~. This ability still resolves if its target becomes illegal.";

inventory::submit! { ManualAbility {
    card: "Gilded Drake",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "When ~ enters, exchange control of ~ and up to one target creature an opponent controls. If you don't or can't make an exchange, sacrifice ~.")
            .iter()
            .map(|a| {
                map_effect(a, TEXT, |e| {
                    Effect::Seq(vec![Effect::Custom(STILL_RESOLVES.into()), e])
                })
            })
            .collect()
    },
    reason: "\"this ability still resolves if its target becomes illegal\" (an exception to CR 608.2b): only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, _g: &mut Game, name: &str, _ctx: &mut Ctx) -> bool {
        // The marker does nothing as the ability resolves.
        name == STILL_RESOLVES
    }
    fn resolves_despite_illegal_targets(&self, _g: &Game, _stack_obj: ObjectId, body: &Body) -> bool {
        matches!(&body.effect, Effect::Seq(v)
            if matches!(v.first(), Some(Effect::Custom(n)) if n == STILL_RESOLVES))
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
