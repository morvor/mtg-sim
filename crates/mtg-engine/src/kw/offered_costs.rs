//! Costs other objects offer for the spells a player casts (CR 118.8, 118.9, 601.2b,
//! 601.2f): alternative costs and optional additional costs that a static ability gives a
//! whole class of spells rather than one spell its own.
//!
//! **Alternative costs** ("You may pay {W}{U}{B}{R}{G} rather than pay the mana cost for
//! spells you cast." — Fist of Suns; "Once each turn, you may pay {0} rather than pay the
//! mana cost for a spell you cast from exile." — Warped Space; "You may cast Dragon spells
//! without paying their mana costs." — Dracogenesis) are `CostChange::AlternativeCost`s
//! for `CostTarget::Spells`, collected as [`OfferedAltCost`]s with the static abilities
//! (`ActiveStatics::offered_alt_costs`). Every way a player could cast a card that has no
//! alternative cost yet — normally, as one half or face of it, with a keyword that isn't an
//! alternative cost such as jump-start (CR 702.133a; Radical Idea ruling), or with an
//! effect's permission that doesn't require a cost of its own — is also offered for each
//! such cost that applies to the spell it would become, judged by the characteristics it
//! would have (CR 601.3e) ([`extend_cast_options`]). The cost replaces the mana cost: no
//! other alternative cost can be applied to the spell (CR 118.9a, 601.2b), so ways that
//! already have one (flashback, evoke, a permission's "pay life equal to its mana value
//! rather than pay its mana cost", ...) aren't offered with it; additional costs are paid
//! and cost increases and reductions apply to it (CR 118.9d, 601.2f); X is 0 unless the
//! cost has X itself (CR 107.3); the spell's mana cost and mana value don't change
//! (CR 118.9c). It doesn't change when the spell can be cast, nor give a permission to cast
//! it from anywhere (Runeforge Champion, Tlincalli Hunter rulings). "Life equal to its mana
//! value" is the mana value of the spell the card would become (X being 0). Without paying
//! its mana cost is `CastMethod::Free`; another such cost is
//! `CastMethod::Alternative(`[`OFFERED_ALT_COST`]`)` (or the keyword's method it's
//! combined with), and the player chooses among the costs offered as they cast the spell.
//!
//! A once-each-turn alternative cost ("Once each turn, ...", "Once during each of your
//! turns, ...") is a static ability with the condition
//! `once_unused(`[`ALT_COST_SLOT`]`)` (`kw/once_each_turn_cast.rs`): casting a spell for
//! it uses it up for the turn ([`record_use`]). Each object's is its own (As Foretold
//! ruling: "If you control multiple As Foretolds, you may cast one spell for each of them
//! paying {0}").

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::CastOption;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::*;
use crate::types::*;
use smol_str::SmolStr;

/// The `CastMethod::Alternative` id of a spell cast normally (or as one of its halves or
/// faces) for an alternative cost another object offers.
pub const OFFERED_ALT_COST: u64 = 0x0FFE_4ED0_0118_0009;

/// The once-each-turn slot of an object's once-each-turn alternative cost.
pub const ALT_COST_SLOT: &str = "alternative cost";

/// An alternative cost an object's static ability (or an effect a player has) offers for
/// the spells a player casts.
#[derive(Clone, Debug)]
pub struct OfferedAltCost {
    /// The object whose ability offers it.
    pub source: ObjectId,
    pub controller: PlayerId,
    /// Whose spells, relative to `controller`.
    pub who: PlayerRel,
    /// Which spells, relative to `source` and `controller`.
    pub spells: Filter,
    pub cost: Cost,
    /// The once-each-turn use it is, if it's one.
    pub once: Option<SmolStr>,
}

/// The object whose alternative cost a way of casting a spell uses, and the once-each-turn
/// use that is, if it's one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AltCostSource {
    pub source: ObjectId,
    pub once: Option<SmolStr>,
}

/// Collects the alternative cost a functioning cost modifier of `source` offers, if it
/// offers one; `condition` is the static ability's (it holds: its once-each-turn use
/// hasn't been used).
pub fn collect(
    out: &mut Vec<OfferedAltCost>,
    source: ObjectId,
    controller: PlayerId,
    cm: &CostModifier,
    condition: Option<&Condition>,
) {
    let (CostTarget::Spells(f), CostChange::AlternativeCost(cost)) = (&cm.applies_to, &cm.change)
    else {
        return;
    };
    out.push(OfferedAltCost {
        source,
        controller,
        who: cm.who,
        spells: f.clone(),
        cost: cost.clone(),
        once: condition.and_then(crate::kw::once_each_turn_cast::condition_slot),
    });
}

/// Whether a cost is "without paying its mana cost": nothing at all.
fn is_nothing(c: &Cost) -> bool {
    c.mana.is_none() && c.parts.is_empty()
}

/// Adds to `out` (the ways `p` may cast `card`) each of those ways that has no alternative
/// cost yet, for each alternative cost other objects offer for the spell it would become
/// (CR 118.9, 118.9a, 601.2b, 601.3e).
pub fn extend_cast_options(g: &Game, p: PlayerId, card: ObjectId, out: &mut Vec<CastOption>) {
    let offers: Vec<&OfferedAltCost> = g
        .statics
        .offered_alt_costs
        .iter()
        .filter(|o| g.player_rel_matches(o.who, p, &Ctx::new(Some(o.source), o.controller)))
        .collect();
    if offers.is_empty() {
        return;
    }
    let base: Vec<CastOption> = out
        .iter()
        .filter(|o| o.alt_cost.is_none() && !matches!(o.method, CastMethod::FaceDown(_)))
        .cloned()
        .collect();
    for opt in base {
        let chars = g.option_characteristics(card, &opt);
        // X is 0 for amounts relative to the spell (CR 107.3b).
        let mv = chars
            .mana_cost
            .as_ref()
            .map_or(0, |m| m.mana_value_with_x(0));
        let mut seen: Vec<Cost> = Vec::new();
        for offer in &offers {
            let ctx = Ctx::new(Some(offer.source), offer.controller);
            let change = CostChange::AlternativeCost(offer.cost.clone());
            if !crate::spell_costs::spells_change_applies_as(
                g,
                card,
                &chars,
                &offer.spells,
                &change,
                &ctx,
            ) {
                continue;
            }
            let cost = crate::permissions::spell_relative_cost(&offer.cost, mv);
            // The same cost offered by several objects any number of times is one way of
            // casting it.
            if offer.once.is_none() {
                if seen.iter().any(|c| format!("{c:?}") == format!("{cost:?}")) {
                    continue;
                }
                seen.push(cost.clone());
            }
            let mut o = opt.clone();
            if matches!(o.method, CastMethod::Normal | CastMethod::Half(_)) {
                o.method = if is_nothing(&cost) {
                    CastMethod::Free
                } else {
                    CastMethod::Alternative(OFFERED_ALT_COST)
                };
            }
            o.alt_cost = Some(cost);
            o.alt_source = Some(AltCostSource {
                source: offer.source,
                once: offer.once.clone(),
            });
            out.push(o);
        }
    }
}

/// Records that a spell was cast for the alternative cost `src` offers: a once-each-turn
/// one is used up for the turn.
pub fn record_use(g: &mut Game, src: Option<&AltCostSource>) {
    if let Some(AltCostSource {
        source,
        once: Some(slot),
    }) = src
    {
        g.history
            .once_permissions_used
            .push((*source, slot.clone()));
        g.dirty = true;
    }
}

/// How a way of casting a spell for the alternative cost `cost` that `src` offers is
/// named to the player choosing how to cast it.
pub fn label(g: &Game, method: &CastMethod, cost: &Cost, src: &AltCostSource) -> String {
    let what = if is_nothing(cost) {
        "without paying its mana cost".to_string()
    } else {
        format!("pay {}", crate::casting::cost_label(cost))
    };
    let from = g
        .try_obj(src.source)
        .map_or_else(String::new, |o| format!(" ({})", o.chars.name));
    match method {
        CastMethod::Keyword(k) => format!("{k:?}, {what}{from}"),
        _ => format!("{what}{from}"),
    }
}

struct OfferedCosts;

impl KeywordRules for OfferedCosts {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
}

inventory::submit! { KeywordRegistration(&OfferedCosts) }
