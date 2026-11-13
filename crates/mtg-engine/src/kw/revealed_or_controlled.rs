//! "As an additional cost to cast this spell, you may reveal a Dragon card from your
//! hand." with "If you revealed a Dragon card or controlled a Dragon as you cast this
//! spell, ..." (Dragons of Tarkir). Revealing is an optional additional cost recorded as
//! "reveal" in the spell's `CastInfo::paid` (CR 601.2b); whether its controller controlled
//! a permanent of that kind as they cast it is recorded as [`CONTROLLED`] as it becomes
//! cast (CR 601.2i). The condition holds if either happened: revealing a card and
//! controlling one gives no additional benefit, nor can more than one card be revealed.
//! A copy of the spell (CR 707.10) has the bonus only if a card was revealed.

use crate::ability::*;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use smol_str::SmolStr;

/// The name of the optional additional cost "you may reveal a [quality] card from your
/// hand" in `CastInfo::paid`.
pub const REVEAL: &str = "reveal";
/// Recorded in `CastInfo::paid` when the caster controlled a permanent with the quality
/// of the card that could be revealed as they cast the spell.
pub const CONTROLLED: &str = "reveal: controlled one as it was cast";

/// The quality of the card the spell's optional reveal cost reveals ("a Dragon card from
/// your hand" → Dragon), if it has one.
fn revealed_quality(chars: &crate::object::Characteristics) -> Option<Filter> {
    chars.abilities.iter().find_map(|a| {
        let AbilityKind::Static(s) = &a.kind else {
            return None;
        };
        let StaticEffect::CostModifier(CostModifier {
            applies_to: CostTarget::ThisSpell,
            change: CostChange::OptionalAdditionalCost { name, cost },
            ..
        }) = &s.effect
        else {
            return None;
        };
        if name.as_str() != REVEAL {
            return None;
        }
        cost.parts.iter().find_map(|p| match p {
            CostPart::RevealFromHand { filter, .. } => Some(permanent_quality(filter)),
            _ => None,
        })
    })
}

/// The parts of "a [quality] card in your hand" that describe the quality.
fn permanent_quality(f: &Filter) -> Filter {
    match f {
        Filter::And(v) => Filter::and(
            v.iter()
                .filter(|x| {
                    !matches!(
                        x,
                        Filter::Card | Filter::InZone(_) | Filter::OwnedBy(_)
                    )
                })
                .cloned()
                .collect(),
        ),
        other => other.clone(),
    }
}

struct RevealedOrControlled;

impl super::KeywordRules for RevealedOrControlled {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn on_event(&self, g: &mut Game, ev: &Event) {
        // A copy copies the reveal (an additional cost paid, CR 707.10), but it wasn't
        // cast: whether a Dragon was controlled as the original was cast doesn't matter.
        if let Event::SpellCopied { spell, .. } = ev {
            if let Some(si) = g.objects[spell.0 as usize].stack.as_mut() {
                si.cast.paid.retain(|p| p.as_str() != CONTROLLED);
            }
            return;
        }
        let Event::SpellCast { spell, player, .. } = ev else {
            return;
        };
        let (spell, p) = (*spell, *player);
        if !g.is_live(spell) {
            return;
        }
        let Some(q) = revealed_quality(&g.obj(spell).chars) else {
            return;
        };
        let ctx = Ctx::new(Some(spell), p);
        let controls = g
            .battlefield
            .iter()
            .any(|id| g.obj(*id).controller == p && g.matches(*id, &q, &ctx));
        if !controls {
            return;
        }
        if let Some(si) = g.objects[spell.0 as usize].stack.as_mut() {
            si.cast.paid.push(SmolStr::new(CONTROLLED));
        }
    }
}

inventory::submit! { super::KeywordRegistration(&RevealedOrControlled) }
