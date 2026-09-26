//! CR 702.88 Rebound.
//!
//! "Rebound" means "If this spell was cast from your hand, instead of putting it into your
//! graveyard as it resolves, exile it and, at the beginning of your next upkeep, you may
//! cast this card from exile without paying its mana cost." (CR 702.88a). A static ability
//! that functions while the spell is on the stack: a replacement effect of where the spell
//! goes as it resolves ([`KeywordRules::resolved_destination`]) that also creates the
//! delayed triggered ability ([`KeywordRules::after_spell_resolved`]). Casting the card
//! with it follows the rules for alternative costs (CR 702.88b); several instances are
//! redundant (CR 702.88c).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::CastOption;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;
use smol_str::SmolStr;

/// `Effect::Custom` of the delayed triggered ability: "you may cast this card from exile
/// without paying its mana cost" (the card is `vars::IT`).
const CAST: &str = "rebound:you may cast this card from exile";

pub struct Rebound;

/// Whether rebound applies to the spell `spell` as it resolves: it was cast from its
/// controller's hand (not a copy, not cast from another zone or another player's hand,
/// and not a spell whose control changed).
fn rebounds(g: &Game, spell: ObjectId) -> bool {
    let o = g.obj(spell);
    o.stack.as_deref().is_some_and(|si| {
        si.cast.was_cast && si.cast.from == Some(ZoneKind::Hand) && o.controller == o.owner
    }) && o.kind == ObjKind::Card
}

impl KeywordRules for Rebound {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Rebound]
    }

    fn resolved_destination(
        &self,
        g: &Game,
        spell: ObjectId,
        _kw: &Keyword,
    ) -> Option<(Zone, LibraryPosition)> {
        rebounds(g, spell).then_some((Zone::Exile, LibraryPosition::Top))
    }

    fn after_spell_resolved(&self, g: &mut Game, spell: ObjectId, _kw: &Keyword, new: ObjectId) {
        if !rebounds(g, spell) || g.obj(new).zone != Zone::Exile {
            return;
        }
        let p = g.obj(spell).controller;
        let mut ctx = Ctx::new(Some(spell), p);
        ctx.set_var(vars::IT, vec![Entity::Object(new)]);
        g.log(|g| format!("{} is exiled (rebound)", g.describe(new)));
        g.exec(
            &Effect::DelayedTrigger {
                trigger: TriggerCond::BeginningOf {
                    step: TriggerStep::Upkeep,
                    whose: PlayerRel::You,
                },
                body: Box::new(Body::effect(Effect::Custom(SmolStr::new(CAST)))),
                once: true,
            },
            &mut ctx,
        );
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != CAST {
            return false;
        }
        let p = ctx.controller;
        let Some(card) = ctx.var_objects(vars::IT).first().copied() else {
            return true;
        };
        // The card, if it's still in exile (a new object otherwise, CR 400.7).
        if !g.is_live(card) || g.obj(card).zone != Zone::Exile {
            return true;
        }
        let prompt = format!(
            "Cast {} from exile without paying its mana cost?",
            g.describe(card)
        );
        if !g.ask_yes_no(p, Some(card), &prompt, true) {
            return true;
        }
        // CR 702.88b: an alternative cost; timing restrictions based on its card type are
        // ignored as it's cast during the ability's resolution (CR 608.2g).
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::Free;
        opt.alt_cost = Some(Cost::free());
        opt.any_time = true;
        let _ = g.cast_with_option(p, card, opt);
        true
    }
}

inventory::submit! { KeywordRegistration(&Rebound) }
