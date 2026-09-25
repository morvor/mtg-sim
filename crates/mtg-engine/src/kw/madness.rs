//! CR 702.35 Madness: "If a player would discard this card, that player discards it, but
//! exiles it instead of putting it into their graveyard" (a static ability that functions
//! while the card is in a hand) and "When this card is exiled this way, its owner may cast
//! it by paying [cost] rather than paying its mana cost. If that player doesn't, they put
//! this card into their graveyard." (CR 702.35a). Casting it follows the rules for
//! alternative costs (CR 702.35b) and ignores timing restrictions based on its card type.
//! If it isn't cast and goes to the graveyard, effects referencing the discarded card can
//! find it there (CR 702.35c, 400.7k).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::CastOption;
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::*;

/// The custom effect of the madness triggered ability.
pub const CAST_MADNESS: &str = "madness_cast";

pub struct Madness;

/// The card's madness cost. A madness ability granted with "The madness cost is equal to
/// its mana cost" has no cost of its own.
fn madness_cost(g: &Game, card: ObjectId) -> Option<Cost> {
    let o = g.obj(card);
    o.chars
        .keywords()
        .find(|k| k.kind == KeywordKind::Madness)
        .map(|k| match &k.cost {
            Some(c) => c.clone(),
            None => Cost::mana(o.chars.mana_cost.clone().unwrap_or_default()),
        })
}

impl KeywordRules for Madness {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Madness]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let text = KeywordKind::Madness.name();
        let mut exile = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::Discard(PlayerFilter::Any, Filter::Source),
            action: ReplacementAction::MoveInstead(Destination::zone(ZoneKind::Exile)),
            self_replacement: false,
            optional: false,
        }));
        exile.zone = FunctionZone::Hand;
        // "When this card is exiled this way": the discard event of this card, which the
        // replacement effect above put into exile (it always applies to a discard of this
        // card, even along with other effects that exile it, as with Rest in Peace).
        let mut cast = TriggeredAbility::new(
            TriggerCond::Discards {
                who: PlayerRel::Any,
                filter: Filter::Source,
            },
            Body::effect(Effect::Custom(CAST_MADNESS.into())),
        );
        cast.zone = FunctionZone::Exile;
        Some(vec![
            AbilityDef::new(AbilityKind::Static(exile), text),
            AbilityDef::new(AbilityKind::Triggered(cast), text),
        ])
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != CAST_MADNESS {
            return false;
        }
        cast_madness(g, ctx);
        true
    }
}

/// The madness triggered ability resolving: the card's owner may cast it from exile by
/// paying its madness cost rather than its mana cost; if they don't, they put it into
/// their graveyard (CR 702.35a).
pub fn cast_madness(g: &mut Game, ctx: &mut Ctx) {
    let Some(card) = ctx.source else {
        return;
    };
    // The ability can't find the card if it has left exile (CR 400.7).
    if !g.is_live(card) || g.obj(card).zone != Zone::Exile {
        return;
    }
    let owner = g.obj(card).owner;
    if let Some(cost) = madness_cost(g, card) {
        let name = g.obj(card).chars.name.clone();
        if g.ask_yes_no(owner, Some(card), &format!("Cast {name} for its madness cost?"), true) {
            let mut opt = CastOption::normal(FaceState::Front);
            opt.method = CastMethod::Keyword(KeywordKind::Madness);
            opt.alt_cost = Some(cost);
            // Timing permissions based on card type don't apply (CR 702.35 rulings).
            opt.any_time = true;
            opt.tag = Some("madness");
            if g.cast_with_option(owner, card, opt).is_ok() {
                return;
            }
        }
    }
    if g.is_live(card) && g.obj(card).zone == Zone::Exile {
        g.move_object_ev(MoveEv {
            obj: card,
            to: Zone::Graveyard(owner),
            pos: LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(owner),
            etb: EtbInfo::default(),
            source: Some(card),
        });
    }
}

/// CR 702.35c, 400.7k: after a madness triggered ability resolved, if the exiled card
/// wasn't cast and was moved to a public zone, effects referencing the discarded card
/// (the object it was in exile) can find the object it became.
pub fn found_after_madness(g: &Game, discarded: ObjectId) -> Option<ObjectId> {
    let o = g.try_obj(discarded)?;
    if o.zone != Zone::Exile || !o.chars.has_keyword(KeywordKind::Madness) {
        return None;
    }
    // It was discarded into exile.
    let prev = g.try_obj(o.prev?)?;
    if !matches!(prev.zone, Zone::Hand(_)) {
        return None;
    }
    let next = o.next?;
    let public = !matches!(
        g.obj(next).zone,
        Zone::Hand(_) | Zone::Library(_) | Zone::Outside(_) | Zone::Nowhere | Zone::Stack
    );
    (public && g.is_live(next)).then_some(next)
}

inventory::submit! { KeywordRegistration(&Madness) }
