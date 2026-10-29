//! Piracy: "Until end of turn, you may tap lands you don't control for mana. Spend this
//! mana only to cast spells."
//!
//! For the rest of the turn its controller may activate the mana abilities of lands other
//! players control (CR 605.3a), and that mana can be spent only to cast spells (CR 106.6).
//! (No land has a mana ability any player may activate, so mana a player taps another
//! player's land for is always mana this effect let them make.)

use super::{spell, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::mana::ManaRestriction;
use crate::object::Zone;
use crate::types::{ObjectId, PlayerId};

const PIRACY: &str = "card:Piracy:you may tap lands you don't control for mana this turn";
const TEXT: &str = "Until end of turn, you may tap lands you don't control for mana. Spend this mana only to cast spells.";

inventory::submit! { ManualAbility {
    card: "Piracy",
    face: 0,
    text: TEXT,
    build: |_ctx| vec![spell(vec![], Effect::Custom(PIRACY.into()), TEXT)],
    reason: "tapping other players' lands for mana: only this card",
} }

/// Whether `p` may tap `src` for mana thanks to the effect.
fn applies(g: &Game, p: PlayerId, src: ObjectId) -> bool {
    let o = g.obj(src);
    o.zone == Zone::Battlefield
        && o.controller != p
        && o.chars.is(crate::types::CardType::Land)
        && g
            .cards
            .get(PIRACY)
            .iter()
            .any(|r| r[0] == i64::from(p.0) && r[1] == i64::from(g.turn.number))
}

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != PIRACY {
            return false;
        }
        let row = vec![i64::from(ctx.controller.0), i64::from(g.turn.number)];
        g.cards.push(PIRACY, row);
        true
    }
    fn may_activate_others_mana_ability(
        &self,
        g: &Game,
        p: PlayerId,
        src: ObjectId,
        a: &Ability,
    ) -> bool {
        // "Tap lands for mana": its mana abilities with {T} in their costs.
        matches!(&a.kind, AbilityKind::Activated(act) if act.cost.has_tap()) && applies(g, p, src)
    }
    fn added_mana_restriction(
        &self,
        g: &Game,
        p: PlayerId,
        src: ObjectId,
    ) -> Option<ManaRestriction> {
        applies(g, p, src).then_some(ManaRestriction::SpellsOnly)
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
