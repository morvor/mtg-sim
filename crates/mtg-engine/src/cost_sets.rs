//! Costs over sets of objects chosen as the cost is paid (CR 118.3, 601.2h, 602.2b):
//!
//! - [`CostPart::AnyNumberOf`]: "Exile one or more other artifacts you control with total
//!   mana value X" (Fabrication Foundry): as many objects as the player chooses, at least
//!   one, which together have the relationships the filter requires (here a total mana
//!   value equal to the X announced for the activation, CR 107.3a). The cost can't be paid
//!   (and so the ability can't be activated) unless such a set exists.
//! - [`CostPart::AndPayItsManaCost`]: "Exile a creature card from your graveyard and pay its
//!   mana cost" (Back from the Brink): one card is chosen and exiled, then its mana cost is
//!   paid as that card last existed: X is 0 (CR 107.3h) and nothing that changes the cost
//!   of casting a spell applies (the card isn't cast; Back from the Brink's rulings). A card
//!   whose mana cost couldn't be paid can't be chosen.

use crate::ability::*;
use crate::casting::{Illegal, PaidCost};
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::*;

/// The permanents an "untap a tapped [permanent]" cost untapped, for "that land" in the
/// ability's effect ("{T}, Untap a tapped land an opponent controls: Add one mana of any
/// type that land could produce.", Benthic Explorers).
pub const COST_UNTAPPED: Var = vars::USER + 30_872;

/// Records [`COST_UNTAPPED`] for the ability's effect.
pub fn record_untapped(paid: &PaidCost, vars: &mut std::collections::BTreeMap<Var, Vec<Entity>>) {
    if !paid.untapped.is_empty() {
        vars.insert(
            COST_UNTAPPED,
            paid.untapped.iter().map(|o| Entity::Object(*o)).collect(),
        );
    }
}

/// The variable holding the card whose mana cost [`CostPart::AndPayItsManaCost`] pays.
const ITS_CARD: Var = vars::USER + 30_871;

impl Game {
    /// The objects an exile or sacrifice part of `p`'s cost could use, with the part's
    /// filter. `None` for other parts.
    fn set_cost_candidates<'a>(
        &self,
        p: PlayerId,
        part: &'a CostPart,
        src: Option<ObjectId>,
        ctx: &Ctx,
    ) -> Option<(Vec<ObjectId>, &'a Filter, bool)> {
        match part {
            CostPart::Exile { filter, zone, .. } => {
                let cands = self
                    .exile_cost_cards(p, *zone, filter, ctx)
                    .into_iter()
                    .filter(|c| {
                        Some(*c) != src && crate::draw_rules::usable_for_cost(self, *c, filter, ctx)
                    })
                    .collect();
                Some((cands, filter, true))
            }
            CostPart::Sacrifice { filter, .. } => {
                let cands = self
                    .objects_matching(filter, ctx)
                    .into_iter()
                    .filter(|o| {
                        self.obj(*o).controller == p
                            && self.may_sacrifice_for_cost(*o, ctx)
                            && Some(*o) != self.mana_reserve
                    })
                    .collect();
                Some((cands, filter, false))
            }
            _ => None,
        }
    }

    /// Whether [`CostPart::AnyNumberOf`] can be paid: some set of at least `min` of the
    /// candidates has the filter's relationships.
    pub(crate) fn any_number_payable(
        &self,
        p: PlayerId,
        part: &CostPart,
        min: u32,
        src: Option<ObjectId>,
        ctx: &Ctx,
    ) -> bool {
        let Some((cands, filter, _)) = self.set_cost_candidates(p, part, src, ctx) else {
            return false;
        };
        (min as usize..=cands.len())
            .any(|n| crate::target_groups::find_together(self, filter, &cands, n, ctx).is_some())
    }

    /// Pays [`CostPart::AnyNumberOf`]: the player chooses the objects; a choice without the
    /// relationships is replaced by a set that has them (the smallest there is).
    pub(crate) fn pay_any_number(
        &mut self,
        p: PlayerId,
        part: &CostPart,
        min: u32,
        src: Option<ObjectId>,
        ctx: &Ctx,
        paid: &mut PaidCost,
    ) -> Result<(), Illegal> {
        let Some((cands, filter, exile)) = self.set_cost_candidates(p, part, src, ctx) else {
            return Err(Illegal("not a cost over a set".into()));
        };
        let filter = filter.clone();
        let prompt = if exile {
            "Choose cards to exile (cost)"
        } else {
            "Choose permanents to sacrifice (cost)"
        };
        let picked = self.ask_objects(p, src, prompt, cands.clone(), min, cands.len() as u32);
        let ok = |g: &Game, v: &[ObjectId]| {
            v.len() >= min as usize
                && v.iter().all(|o| cands.contains(o))
                && crate::target_groups::together_ok(g, &filter, v, ctx)
        };
        let pick = if ok(self, &picked) {
            picked
        } else {
            (min as usize..=cands.len())
                .find_map(|n| crate::target_groups::find_together(self, &filter, &cands, n, ctx))
                .ok_or_else(|| Illegal("no set of objects can pay the cost".into()))?
        };
        paid.objects.extend(pick.iter().copied());
        if exile {
            for c in pick {
                paid.exiled.push(c);
                self.exile_object(c, src);
            }
        } else {
            // Sacrificed together (CR 603.10a).
            paid.sacrificed.extend(pick.iter().copied());
            let what: Vec<(ObjectId, PlayerId)> = pick.iter().map(|o| (*o, p)).collect();
            self.sacrifice_simultaneously(&what);
        }
        Ok(())
    }

    /// The candidates of [`CostPart::AndPayItsManaCost`] whose mana cost `p` could pay.
    fn its_mana_cost_candidates(
        &self,
        p: PlayerId,
        part: &CostPart,
        src: Option<ObjectId>,
        ctx: &Ctx,
    ) -> Vec<ObjectId> {
        let Some((cands, _, true)) = self.set_cost_candidates(p, part, src, ctx) else {
            return vec![];
        };
        cands
            .into_iter()
            .filter(|c| {
                let mut c2 = ctx.clone();
                c2.vars.insert(ITS_CARD, vec![Entity::Object(*c)]);
                crate::mana_abilities::can_pay_mana_cost_of(self, p, &Sel::Var(ITS_CARD), src, &c2)
            })
            .collect()
    }

    /// Whether [`CostPart::AndPayItsManaCost`] can be paid.
    pub(crate) fn and_pay_its_mana_cost_payable(
        &self,
        p: PlayerId,
        part: &CostPart,
        src: Option<ObjectId>,
        ctx: &Ctx,
    ) -> bool {
        !self.its_mana_cost_candidates(p, part, src, ctx).is_empty()
    }

    /// Pays [`CostPart::AndPayItsManaCost`]: chooses and exiles the card, then pays its
    /// mana cost as it last existed.
    pub(crate) fn pay_and_its_mana_cost(
        &mut self,
        p: PlayerId,
        part: &CostPart,
        src: Option<ObjectId>,
        ctx: &Ctx,
        paid: &mut PaidCost,
    ) -> Result<(), Illegal> {
        let cands = self.its_mana_cost_candidates(p, part, src, ctx);
        if cands.is_empty() {
            return Err(Illegal("no card whose mana cost can be paid".into()));
        }
        let card = self
            .ask_objects(p, src, "Choose a card to exile (cost)", cands.clone(), 1, 1)
            .into_iter()
            .find(|c| cands.contains(c))
            .unwrap_or(cands[0]);
        paid.objects.push(card);
        paid.exiled.push(card);
        self.exile_object(card, src);
        // Its mana cost as it last existed in the graveyard (the old object keeps it).
        let mut c2 = ctx.clone();
        c2.vars.insert(ITS_CARD, vec![Entity::Object(card)]);
        if !crate::mana_abilities::pay_mana_cost_of(self, p, &Sel::Var(ITS_CARD), src, &c2) {
            return Err(Illegal("can't pay its mana cost".into()));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// "choose a creature type and behold two creatures of that type" (a cost)
// ---------------------------------------------------------------------------

/// `Effect::Custom` prefix (followed by the number): "choose a creature type and behold N
/// creatures of that type" (Celestial Reunion's additional cost). The type is chosen among
/// those the player could then behold N creatures of (a cost must be payable in full,
/// CR 118.3; only creature cards in hand and creatures they control, not kindred cards,
/// its ruling), recorded as the spell's chosen type (CR 601.2b), and then N creatures of
/// it are beheld (CR 701.4a).
pub const CHOOSE_TYPE_AND_BEHOLD: &str = "cost sets:choose a creature type and behold ";

fn behold_count(name: &str) -> Option<u32> {
    name.strip_prefix(CHOOSE_TYPE_AND_BEHOLD)?.parse().ok()
}

fn of_type(t: &str) -> Filter {
    Filter::and(vec![
        Filter::Type(CardType::Creature),
        Filter::Subtype(smol_str::SmolStr::new(t)),
    ])
}

/// The creature types `p` could behold `n` creatures of.
fn beholdable_types(g: &Game, p: PlayerId, n: u32, ctx: &Ctx) -> Vec<String> {
    subtype_lists()
        .creature
        .iter()
        .filter(|t| crate::behold::can_behold(g, p, &of_type(t), n, ctx))
        .cloned()
        .collect()
}

pub struct ChooseTypeAndBehold;

impl crate::kw::KeywordRules for ChooseTypeAndBehold {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
    }

    fn custom_effect_possible(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        let n = behold_count(name)?;
        Some(!beholdable_types(g, ctx.controller, n, ctx).is_empty())
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some(n) = behold_count(name) else {
            return false;
        };
        let p = ctx.controller;
        let types = beholdable_types(g, p, n, ctx);
        let (Some(src), false) = (ctx.source, types.is_empty()) else {
            ctx.prev_happened = false;
            return true;
        };
        let i = g.ask_option(p, Some(src), "Choose a creature type", types.clone());
        let t = smol_str::SmolStr::new(&types[i.min(types.len() - 1)]);
        let link = ctx.link;
        let o = &mut g.objects[src.0 as usize];
        o.choices.creature_type = Some(t.clone());
        o.choices.list.push(t.clone());
        for l in [0, link] {
            o.linked_choices.entry(l).or_default().creature_type = Some(t.clone());
        }
        g.dirty = true;
        crate::behold::behold(g, p, &of_type(&t), n, ctx);
        true
    }
}

inventory::submit! { crate::kw::KeywordRegistration(&ChooseTypeAndBehold) }

// ---------------------------------------------------------------------------
// Target groups sized by a repeatable additional cost
// ---------------------------------------------------------------------------

/// `Value::Custom` prefix (followed by the cost's name): how many times the spell's
/// repeatable additional cost was announced (CR 601.2b), for the size of a target group
/// chosen after it (CR 601.2c: "For each additional {1}{R} you paid, destroy another target
/// artifact"). Unlike `spell_cost_grammar::paid_times`, which a cost reduction reads as the
/// most it could be while the spell is only being considered, this is 0 until the spell is
/// on the stack: a spell needs no targets for payments it hasn't announced.
pub const TIMES_ANNOUNCED: &str = "cost sets:times announced:";

/// The value [`TIMES_ANNOUNCED`] of the repeatable cost `name`.
pub fn times_announced(name: &str) -> Value {
    Value::Custom(format!("{TIMES_ANNOUNCED}{name}").into())
}

pub struct TimesAnnounced;

impl crate::kw::KeywordRules for TimesAnnounced {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        let n = name.strip_prefix(TIMES_ANNOUNCED)?;
        let Some(src) = ctx.source else {
            return Some(0);
        };
        let times = g.obj(src).stack.as_deref().map_or(0, |si| {
            si.cast.paid.iter().filter(|p| p.as_str() == n).count()
        });
        Some(times as i64)
    }
}

inventory::submit! { crate::kw::KeywordRegistration(&TimesAnnounced) }

/// The spell's own additional cost `c` without its repeatable parts ("you may pay {1}{G}
/// any number of times") while `card` is only being considered for casting (not on the
/// stack): the number of times is announced as it's cast (CR 601.2b), and paying it zero
/// times is the least it can cost, so whether the spell could be cast doesn't depend on
/// paying them.
pub fn unannounced_repeats_left_out(g: &Game, card: ObjectId, c: &Cost) -> Cost {
    if matches!(g.obj(card).zone, crate::object::Zone::Stack) {
        return c.clone();
    }
    let prefix = crate::kw::spell_cost_grammar::paid_times("");
    let mut out = c.clone();
    out.parts.retain(|p| {
        !matches!(p, CostPart::Repeated { times: Value::Custom(n), .. } if n.starts_with(prefix.as_str()))
    });
    out
}
