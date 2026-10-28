//! Where an instant or sorcery spell goes as it resolves, changed by an ability an effect
//! gave it (a replacement effect of its being put into its owner's graveyard, CR 608.2n,
//! 614.1a):
//!
//! * [`TO_HAND`]: "The next time you cast an instant or sorcery spell from your hand this
//!   turn, put that card into your hand instead of into your graveyard as it resolves."
//!   (Soulfire Grand Master);
//! * [`EXILE_THEN_HAND`]: "exile that card instead of putting it into your graveyard as it
//!   resolves. If you do, return it to your hand at the beginning of the next end step."
//!   (Feather, the Redeemed): a delayed triggered ability returns the exiled card
//!   (CR 603.7).
//!
//! A spell that's countered, or never resolves, isn't affected, nor is one its controller
//! doesn't own (it would be put into another player's graveyard).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::*;
use crate::types::*;
use smol_str::SmolStr;

/// "Put that card into your hand instead of into your graveyard as it resolves."
pub const TO_HAND: &str = "put this spell into its owner's hand instead of into a graveyard as it resolves";
/// "Exile that card instead of putting it into your graveyard as it resolves. If you do,
/// return it to your hand at the beginning of the next end step."
pub const EXILE_THEN_HAND: &str =
    "exile this spell instead of putting it into a graveyard as it resolves; return it to its owner's hand at the beginning of the next end step";

/// The marker ability `name` given to a spell.
pub fn marker(name: &str) -> Modification {
    let mut s = StaticAbility::new(StaticEffect::Custom(SmolStr::new(name)));
    s.zone = FunctionZone::Stack;
    Modification::AddAbility(AbilityDef::new(AbilityKind::Static(s), name))
}

fn has_marker(g: &Game, spell: ObjectId, name: &str) -> bool {
    g.obj(spell).chars.abilities.iter().any(|a| {
        matches!(&a.kind, AbilityKind::Static(s) if matches!(&s.effect, StaticEffect::Custom(n) if n == name))
    })
}

pub struct SpellDestinationMarkers;

impl KeywordRules for SpellDestinationMarkers {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn global_resolved_destination(
        &self,
        g: &Game,
        spell: ObjectId,
    ) -> Option<(Zone, LibraryPosition)> {
        // "Instead of putting it into your graveyard": a spell its controller doesn't own
        // would go to another player's graveyard, so these don't apply to it.
        if g.obj(spell).owner != g.obj(spell).controller {
            return None;
        }
        if has_marker(g, spell, TO_HAND) {
            return Some((Zone::Hand(g.obj(spell).owner), LibraryPosition::Top));
        }
        has_marker(g, spell, EXILE_THEN_HAND).then_some((Zone::Exile, LibraryPosition::Top))
    }

    fn global_after_spell_resolved(&self, g: &mut Game, spell: ObjectId, new: ObjectId) {
        if !has_marker(g, spell, EXILE_THEN_HAND)
            || g.obj(new).zone != Zone::Exile
            || g.obj(spell).owner != g.obj(spell).controller
        {
            return;
        }
        let controller = g.obj(spell).controller;
        let mut ctx = Ctx::new(Some(spell), controller);
        ctx.set_var(vars::IT, vec![Entity::Object(new)]);
        let e = Effect::DelayedTrigger {
            trigger: TriggerCond::BeginningOf {
                step: TriggerStep::End,
                whose: PlayerRel::Any,
            },
            body: Box::new(Body::effect(Effect::Move {
                what: Sel::All(Filter::and(vec![
                    Filter::In(Box::new(Sel::Var(vars::IT))),
                    Filter::InZone(ZoneKind::Exile),
                ])),
                to: Destination::zone(ZoneKind::Hand),
            })),
            once: true,
        };
        g.exec(&e, &mut ctx);
    }
}

inventory::submit! { KeywordRegistration(&SpellDestinationMarkers) }
