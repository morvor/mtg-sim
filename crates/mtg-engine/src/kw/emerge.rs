//! CR 702.119 Emerge.
//!
//! * "Emerge [cost]" means "You may cast this spell by paying [cost] and sacrificing a
//!   creature rather than paying its mana cost" and "If you chose to pay this spell's
//!   emerge cost, its total cost is reduced by an amount of generic mana equal to the
//!   sacrificed creature's mana value." (CR 702.119a). It's an alternative cost
//!   (CR 601.2b, 601.2f–h): the spell's mana value doesn't change.
//! * "Emerge from [quality] [cost]" sacrifices a [quality] permanent instead (CR 702.119b);
//!   the quality is the keyword's filter.
//! * The permanent to sacrifice is chosen as the player chooses to pay the emerge cost
//!   (CR 601.2b, [`KeywordRules::announce`]), and sacrificed as the total cost is paid
//!   (CR 702.119c, 601.2h): it's still on the battlefield while the total cost is
//!   determined and mana abilities are activated, so it can be tapped for mana first.
//! * Only generic mana is reduced; the colored part of the emerge cost remains.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::{CastOption, Illegal};
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;

/// The name recorded in `CastInfo::paid` when a spell is cast for its emerge cost.
pub const EMERGE: &str = "emerge";

/// `Condition::Custom`: a spell with emerge that the ability's controller is casting is on
/// top of the stack ("When you sacrifice ~ while casting a spell with emerge"), whether or
/// not it's being cast for its emerge cost.
pub const CASTING_A_SPELL_WITH_EMERGE: &str = "emerge:you're casting a spell with emerge";

/// The variable of the spell's saved context holding the permanent chosen to be
/// sacrificed for its emerge cost.
const CHOSEN: Var = vars::USER + 119;

fn is_emerge(method: &CastMethod) -> bool {
    *method == CastMethod::Keyword(KeywordKind::Emerge)
}

/// What the emerge ability says to sacrifice: a creature, or a [quality] permanent.
fn quality(kw: &Keyword) -> Filter {
    kw.filter.clone().unwrap_or_else(Filter::creature)
}

/// Permanents `p` could sacrifice to cast `card` for this emerge cost.
fn candidates(g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<ObjectId> {
    let q = quality(kw);
    let ctx = Ctx::new(Some(card), p);
    g.permanents()
        .filter(|o| o.controller == p && o.id != card)
        .map(|o| o.id)
        .filter(|o| g.matches(*o, &q, &ctx) && !g.cant_be_sacrificed(*o))
        .collect()
}

/// The permanent chosen to be sacrificed for the emerge cost of the spell `spell`.
fn chosen(g: &Game, spell: ObjectId) -> Option<ObjectId> {
    g.saved_ctx
        .get(&spell)
        .and_then(|c| c.vars.get(&CHOSEN))
        .and_then(|v| v.first())
        .and_then(|e| e.object())
}

pub struct Emerge;

impl KeywordRules for Emerge {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Emerge]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != CASTING_A_SPELL_WITH_EMERGE {
            return None;
        }
        Some(
            g.special.casting > 0
                && g.stack.last().is_some_and(|s| {
                    let o = g.obj(*s);
                    o.is_spell()
                        && o.controller == ctx.controller
                        && o.chars.has_keyword(KeywordKind::Emerge)
                }),
        )
    }

    fn cast_options(&self, g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<CastOption> {
        let Some(cost) = kw.cost.clone() else {
            return vec![];
        };
        if candidates(g, p, card, kw).is_empty() {
            return vec![];
        }
        let o = g.obj(card);
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::Keyword(KeywordKind::Emerge);
        if o.zone != Zone::Hand(p) {
            let chars = g.option_characteristics(card, &opt);
            if !g.permitted_cards(p).contains(&card) || !g.permission_allows(p, card, &chars, false)
            {
                return vec![];
            }
        }
        opt.alt_cost = Some(super::modified_keyword_cost(
            g,
            p,
            KeywordKind::Emerge,
            &cost,
        ));
        opt.tag = Some(EMERGE);
        vec![opt]
    }

    /// CR 702.119c, 601.2b: the permanent to sacrifice is chosen now; sacrificing it is
    /// part of the total cost.
    fn announce(
        &self,
        g: &mut Game,
        p: PlayerId,
        spell: ObjectId,
        kw: &Keyword,
        method: &CastMethod,
        extra: &mut Cost,
    ) -> Result<(), Illegal> {
        if !is_emerge(method) {
            return Ok(());
        }
        let cands = candidates(g, p, spell, kw);
        if cands.is_empty() {
            return Err(Illegal("nothing to sacrifice for emerge".into()));
        }
        let entities: Vec<Entity> = cands.iter().map(|c| Entity::Object(*c)).collect();
        let pick = match g.ask(
            p,
            Decision::ChooseEntities {
                source: Some(spell),
                prompt: "Choose a permanent to sacrifice (emerge)".into(),
                candidates: entities.clone(),
                min: 1,
                max: 1,
            },
        ) {
            Answer::Entities(v) if v.len() == 1 && entities.contains(&v[0]) => v[0].object(),
            // By default, the one that reduces the cost most.
            _ => cands.iter().copied().max_by_key(|c| g.mana_value_of(*c)),
        };
        let Some(pick) = pick else {
            return Err(Illegal("nothing to sacrifice for emerge".into()));
        };
        g.saved_ctx
            .entry(spell)
            .or_insert_with(|| Ctx::new(Some(spell), p))
            .vars
            .insert(CHOSEN, vec![Entity::Object(pick)]);
        crate::casting::add_cost(
            extra,
            &Cost::free().with(CostPart::Sacrifice {
                filter: Filter::Objects(vec![pick]),
                count: Value::c(1),
            }),
        );
        g.log(|g| format!("{p} will sacrifice {} (emerge)", g.describe(pick)));
        Ok(())
    }

    /// CR 702.119a: once the total cost is determined (CR 601.2f), it's reduced by generic
    /// mana equal to the chosen permanent's mana value. (Generic reductions don't depend
    /// on the order they're applied in, and every increase is already included.)
    fn pay_mana_otherwise(
        &self,
        g: &mut Game,
        _p: PlayerId,
        spell: ObjectId,
        _kw: &Keyword,
        cost: &mut Cost,
    ) -> Result<(), Illegal> {
        let emerged = g
            .obj(spell)
            .stack
            .as_deref()
            .is_some_and(|s| is_emerge(&s.cast.method));
        if !emerged {
            return Ok(());
        }
        if let (Some(c), Some(m)) = (chosen(g, spell), cost.mana.as_mut()) {
            m.reduce_generic(g.mana_value_of(c));
        }
        Ok(())
    }

    /// For the check whether it could be cast for its emerge cost: the best reduction.
    fn payable_otherwise(
        &self,
        g: &Game,
        p: PlayerId,
        card: ObjectId,
        kw: &Keyword,
        method: &CastMethod,
        cost: &mut Cost,
    ) {
        if !is_emerge(method) {
            return;
        }
        let best = candidates(g, p, card, kw)
            .into_iter()
            .map(|c| g.mana_value_of(c))
            .max()
            .unwrap_or(0);
        if let Some(m) = cost.mana.as_mut() {
            m.reduce_generic(best);
        }
    }
}

inventory::submit! { KeywordRegistration(&Emerge) }
