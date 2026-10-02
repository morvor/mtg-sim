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
//! One that comes with "If you cast a spell this way, you may cast it as though it had
//! flash" (`CostChange::AlternativeCostWithFlash`, Primal Prayers) lets the spell be cast
//! as though it had flash when cast for it (CR 601.3c).
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
use crate::decision::{Answer, Decision};
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
    /// "If you cast a spell this way, you may cast it as though it had flash" (CR 601.3c:
    /// the player may begin to cast it as though it had flash).
    pub flash: bool,
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
    let (CostTarget::Spells(f), change) = (&cm.applies_to, &cm.change) else {
        return;
    };
    let (cost, flash) = match change {
        CostChange::AlternativeCost(c) => (c, false),
        CostChange::AlternativeCostWithFlash(c) => (c, true),
        _ => return,
    };
    out.push(OfferedAltCost {
        source,
        controller,
        who: cm.who,
        spells: f.clone(),
        cost: cost.clone(),
        flash,
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
        let mut seen: Vec<(Cost, bool)> = Vec::new();
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
            let cost = cost_for_spell(&offer.cost, mv);
            // The same cost offered by several objects any number of times is one way of
            // casting it.
            if offer.once.is_none() {
                if seen
                    .iter()
                    .any(|(c, f)| *f == offer.flash && format!("{c:?}") == format!("{cost:?}"))
                {
                    continue;
                }
                seen.push((cost.clone(), offer.flash));
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
            // CR 601.3c: cast this way, it may be cast as though it had flash.
            o.flash |= offer.flash;
            o.alt_source = Some(AltCostSource {
                source: offer.source,
                once: offer.once.clone(),
            });
            out.push(o);
        }
    }
}

/// An offered alternative cost for a spell with mana value `mv` (X being 0, CR 107.3b),
/// amounts relative to the spell given: "life equal to its mana value", and "{X}, where X
/// is that spell's mana value" (`CostPart::Repeated` of {1} that many times: that much
/// generic mana).
pub fn cost_for_spell(cost: &Cost, mv: u32) -> Cost {
    let mut c = crate::permissions::spell_relative_cost(cost, mv);
    let mut parts = Vec::new();
    for part in std::mem::take(&mut c.parts) {
        match part {
            CostPart::Repeated { cost, times: Value::ManaValueOf(s) }
                if matches!(*s, Sel::This) && cost.parts.is_empty() && cost.mana.is_some() =>
            {
                let total = c.mana.get_or_insert_with(crate::mana::ManaCost::default);
                if let Some(m) = &cost.mana {
                    for _ in 0..mv {
                        total.add(m);
                    }
                }
            }
            other => parts.push(other),
        }
    }
    c.parts = parts;
    c
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
    // Named with the object offering it (two As Foretolds are told apart).
    let from = g
        .try_obj(src.source)
        .map_or_else(String::new, |_| format!(" ({})", g.describe(src.source)));
    match method {
        CastMethod::Keyword(k) => format!("{k:?}, {what}{from}"),
        _ => format!("{what}{from}"),
    }
}

/// `Filter::Custom` prefix, followed by the name of an optional additional cost the
/// object whose ability the filter is in offers: "a spell for which that cost was paid"
/// ("Those spells cost {G} less to cast if you paid life this way.").
pub const PAID_OFFERED_COST: &str = "offered optional cost paid:";

/// `Value::Custom` prefix, followed by the name of an optional additional cost the object
/// whose ability the value is in offers: how many times it was paid for the spell (or the
/// permanent entering from it) the value is about — the amount of mana paid for "you may
/// pay any amount of mana" ("that creature enters with that many additional +1/+1
/// counters on it").
pub const PAID_OFFERED_AMOUNT: &str = "offered optional cost amount paid:";

/// The name recorded in a spell's `CastInfo::paid` for the optional additional cost `name`
/// the object `src` offers: each object's is its own (Defiler of Vigor: "You may only pay
/// the additional cost once per permanent spell", once for each Defiler).
pub fn paid_name(name: &str, src: ObjectId) -> SmolStr {
    SmolStr::new(format!("{name}@{}", src.0))
}

/// Announces the optional additional costs and choices between additional costs other
/// objects offer for `spell` (with the characteristics `chars`) as `p` casts it
/// (CR 601.2b, 118.8): "As an additional cost to cast green permanent spells, you may pay 2
/// life." Each object's cost is offered once; what's chosen is added to `extra` and
/// recorded in `paid` under [`paid_name`]. A cost that can't be paid can't be chosen.
pub fn announce(
    g: &mut Game,
    p: PlayerId,
    spell: ObjectId,
    chars: &Characteristics,
    extra: &mut Cost,
    paid: &mut Vec<SmolStr>,
) {
    let offers: Vec<(ObjectId, CostChange)> = g
        .statics
        .cost_modifiers
        .iter()
        .filter(|(src, ctl, cm)| {
            let CostTarget::Spells(f) = &cm.applies_to else {
                return false;
            };
            if !matches!(
                cm.change,
                CostChange::OptionalAdditionalCost { .. } | CostChange::AdditionalCostChoice(_)
            ) {
                return false;
            }
            let ctx = Ctx::new(Some(*src), *ctl);
            g.player_rel_matches(cm.who, p, &ctx)
                && crate::spell_costs::spells_change_applies(g, spell, f, &cm.change, &ctx)
        })
        .map(|(src, _, cm)| (*src, cm.change.clone()))
        .collect();
    for (src, change) in offers {
        let from = g.obj(src).chars.name.clone();
        match change {
            // "You may pay any amount of mana" ({X}): the player announces how much
            // (CR 601.2b), recorded once for each mana.
            CostChange::OptionalAdditionalCost { name, cost }
                if cost.parts.is_empty() && cost.mana.as_ref().is_some_and(|m| m.has_x()) =>
            {
                let n = match g.ask(
                    p,
                    Decision::OptionalCost {
                        source: spell,
                        name: format!("{name} ({from})"),
                        repeatable: true,
                    },
                ) {
                    Answer::Number(n) if n > 0 => n as u32,
                    Answer::Bool(true) => 1,
                    _ => 0,
                }
                .min(g.max_mana_available(p));
                if n > 0 {
                    crate::casting::add_cost(extra, &Cost::mana(crate::mana::ManaCost::generic(n)));
                    for _ in 0..n {
                        paid.push(paid_name(&name, src));
                    }
                }
            }
            CostChange::OptionalAdditionalCost { name, cost } => {
                if g.can_pay_cost_optimistic(p, &cost, Some(spell), chars)
                    && matches!(
                        g.ask(
                            p,
                            Decision::OptionalCost {
                                source: spell,
                                name: format!("{name} ({from})"),
                                repeatable: false,
                            }
                        ),
                        Answer::Bool(true)
                    )
                {
                    crate::casting::add_cost(extra, &cost);
                    paid.push(paid_name(&name, src));
                }
            }
            CostChange::AdditionalCostChoice(options) => {
                let payable: Vec<(SmolStr, Cost)> = options
                    .iter()
                    .filter(|(_, c)| g.can_pay_cost_optimistic(p, c, Some(spell), chars))
                    .cloned()
                    .collect();
                let pool = if payable.is_empty() { options } else { payable };
                let i = g.ask_option(
                    p,
                    Some(spell),
                    &format!("Choose an additional cost to pay ({from})"),
                    pool.iter().map(|(n, _)| n.to_string()).collect(),
                );
                if let Some((name, cost)) = pool.into_iter().nth(i) {
                    crate::casting::add_cost(extra, &cost);
                    paid.push(paid_name(&name, src));
                }
            }
            _ => {}
        }
    }
}

struct OfferedCosts;

impl KeywordRules for OfferedCosts {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    /// [`PAID_OFFERED_COST`]: the spell's controller paid the optional additional cost of
    /// that name the filter's source offers (`ctx.source`).
    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        let cost = name.strip_prefix(PAID_OFFERED_COST)?;
        let Some(src) = ctx.source else {
            return Some(false);
        };
        let want = paid_name(cost, src);
        // A spell on the stack, or one entering the battlefield as it resolves.
        let o = g.obj(id);
        let cast = o.stack.as_ref().map(|si| &si.cast).or(o.cast.as_deref());
        Some(cast.is_some_and(|c| c.was_cast && c.paid.iter().any(|x| *x == want)))
    }

    /// [`PAID_OFFERED_AMOUNT`]: how many times the spell `ctx.cast` describes (the one a
    /// replacement effect modifies as it enters) had that cost of `ctx.source`'s paid.
    fn custom_value(&self, _g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        let cost = name.strip_prefix(PAID_OFFERED_AMOUNT)?;
        let (Some(src), Some(cast)) = (ctx.source, ctx.cast.as_ref()) else {
            return Some(0);
        };
        let want = paid_name(cost, src);
        Some(cast.paid.iter().filter(|x| **x == want).count() as i64)
    }
}

inventory::submit! { KeywordRegistration(&OfferedCosts) }
