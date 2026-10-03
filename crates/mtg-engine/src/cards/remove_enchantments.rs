//! Remove Enchantments: "Return to your hand all enchantments you both own and control,
//! all Auras you own attached to permanents you control, and all Auras you own attached
//! to attacking creatures your opponents control. Then destroy all other enchantments you
//! control, all other Auras attached to permanents you control, and all other Auras
//! attached to attacking creatures your opponents control."

use super::{spell, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{CardType, Entity, ObjectId};

const RESOLVE: &str = "card:Remove Enchantments:return and destroy enchantments";
const TEXT: &str = "Return to your hand all enchantments you both own and control, all Auras you own attached to permanents you control, and all Auras you own attached to attacking creatures your opponents control. Then destroy all other enchantments you control, all other Auras attached to permanents you control, and all other Auras attached to attacking creatures your opponents control.";

inventory::submit! { ManualAbility {
    card: "Remove Enchantments",
    face: 0,
    text: TEXT,
    build: |_| vec![spell(vec![], Effect::Custom(RESOLVE.into()), TEXT)],
    reason: "returns or destroys enchantments by owner/controller/attachment combinations: unique",
} }

struct Rules;

/// The enchantments the instruction covers ("you control", "Auras attached to permanents
/// you control", "Auras attached to attacking creatures your opponents control").
fn covered(g: &Game, ctx: &Ctx) -> Vec<ObjectId> {
    let you = ctx.controller;
    g.permanents()
        .filter(|o| o.is(CardType::Enchantment))
        .filter(|o| {
            if o.controller == you {
                return true;
            }
            if !o.chars.has_subtype("Aura") {
                return false;
            }
            match o.attached_to {
                Some(Entity::Object(h)) => {
                    let host = g.obj(h);
                    host.controller == you
                        || (g.is_attacking(h) && g.are_opponents(you, host.controller))
                }
                _ => false,
            }
        })
        .map(|o| o.id)
        .collect()
}

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != RESOLVE {
            return false;
        }
        let you = ctx.controller;
        let all = covered(g, ctx);
        // The ones you own of these are returned: enchantments you both own and control,
        // and Auras you own attached as described.
        let (mine, others): (Vec<ObjectId>, Vec<ObjectId>) =
            all.into_iter().partition(|o| g.obj(*o).owner == you);
        let var = vars::USER + 40;
        ctx.set_var(var, mine.into_iter().map(Entity::Object).collect());
        g.exec(
            &Effect::Move {
                what: Sel::Var(var),
                to: Destination::zone(ZoneKind::Hand),
            },
            ctx,
        );
        ctx.set_var(var, others.into_iter().map(Entity::Object).collect());
        g.exec(
            &Effect::Destroy {
                what: Sel::Var(var),
                no_regen: false,
            },
            ctx,
        );
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
