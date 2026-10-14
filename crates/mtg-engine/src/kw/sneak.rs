//! CR 702.190 Sneak.
//!
//! * "Sneak [cost]" means "Any time you could cast an instant during your declare
//!   blockers step, you may cast this spell by paying [cost] and returning an unblocked
//!   creature you control to its owner's hand rather than paying this spell's mana cost."
//!   (CR 702.190a). It's an alternative cost (CR 601.2b, 601.2f–h). The unblocked
//!   attacking creature to return is chosen as the player chooses to pay the sneak cost
//!   ([`KeywordRules::announce`]) and returned as the total cost is paid.
//! * A permanent spell whose sneak cost was paid enters the battlefield tapped and
//!   attacking the same player, planeswalker, or battle as the returned creature was
//!   (CR 702.190b, 506.3a; [`KeywordRules::permanent_spell_etb`]). It was never declared
//!   as an attacker.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::{CastOption, Illegal};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::turn::Step;
use crate::types::*;

/// The name recorded in `CastInfo::paid` when a spell is cast for its sneak cost.
pub const SNEAK: &str = "sneak";

/// The variable of the spell's saved context holding what the returned creature was
/// attacking.
const ATTACKED: Var = vars::USER + 190;

fn is_sneak(method: &CastMethod) -> bool {
    *method == CastMethod::Keyword(KeywordKind::Sneak)
}

/// Unblocked attacking creatures `p` controls (CR 509.1h).
fn unblocked_attackers(g: &Game, p: PlayerId) -> Vec<ObjectId> {
    let Some(cb) = g.combat.as_ref() else {
        return vec![];
    };
    g.permanents()
        .filter(|o| o.controller == p && o.is_creature() && cb.is_unblocked(o.id))
        .map(|o| o.id)
        .collect()
}

/// Whether `p` could cast a spell for its sneak cost now: during their declare blockers
/// step (any time they could cast an instant then).
fn sneak_window(g: &Game, p: PlayerId) -> bool {
    g.turn.active == p && g.turn.step == Step::DeclareBlockers
}

/// Whether the spell was cast for its sneak cost.
pub fn sneaked(g: &Game, spell: ObjectId) -> bool {
    g.obj(spell)
        .stack
        .as_deref()
        .is_some_and(|s| s.cast.paid.iter().any(|x| x == SNEAK))
}

pub struct Sneak;

impl KeywordRules for Sneak {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Sneak]
    }

    fn cast_options(&self, g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<CastOption> {
        let Some(cost) = kw.cost.clone() else {
            return vec![];
        };
        if !sneak_window(g, p) || unblocked_attackers(g, p).is_empty() {
            return vec![];
        }
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::Keyword(KeywordKind::Sneak);
        // From its owner's hand, or a zone an effect lets it be cast from.
        if g.obj(card).zone != Zone::Hand(p) {
            let chars = g.option_characteristics(card, &opt);
            if !g.permitted_cards(p).contains(&card) || !g.permission_allows(p, card, &chars, false)
            {
                return vec![];
            }
        }
        opt.alt_cost = Some(super::modified_keyword_cost(
            g,
            p,
            KeywordKind::Sneak,
            &cost,
        ));
        // "Any time you could cast an instant during your declare blockers step."
        opt.flash = true;
        opt.tag = Some(SNEAK);
        vec![opt]
    }

    /// The unblocked attacking creature to return is chosen now (CR 601.2b); returning it
    /// is part of the total cost.
    fn announce(
        &self,
        g: &mut Game,
        p: PlayerId,
        spell: ObjectId,
        _kw: &Keyword,
        method: &CastMethod,
        extra: &mut Cost,
    ) -> Result<(), Illegal> {
        if !is_sneak(method) {
            return Ok(());
        }
        let cands = unblocked_attackers(g, p);
        let pick = match cands.len() {
            0 => return Err(Illegal("no unblocked attacking creature to return".into())),
            1 => cands[0],
            _ => {
                let es: Vec<Entity> = cands.iter().map(|c| Entity::Object(*c)).collect();
                g.ask_entities(
                    p,
                    Some(spell),
                    "Choose an unblocked attacking creature to return (sneak)",
                    es,
                    1,
                    1,
                )
                .into_iter()
                .filter_map(|e| e.object())
                .find(|o| cands.contains(o))
                .unwrap_or(cands[0])
            }
        };
        let attacked = g.combat.as_ref().and_then(|cb| cb.attack_target(pick));
        let saved = g
            .saved_ctx
            .entry(spell)
            .or_insert_with(|| Ctx::new(Some(spell), p));
        saved.vars.insert(ATTACKED, attacked.into_iter().collect());
        crate::casting::add_cost(
            extra,
            &Cost::free().with(CostPart::ReturnToHand {
                filter: Filter::Objects(vec![pick]),
                count: Value::c(1),
            }),
        );
        g.log(|g| format!("{p} will return {} (sneak)", g.describe(pick)));
        Ok(())
    }

    /// CR 702.190b: it enters tapped and attacking what the returned creature attacked.
    fn permanent_spell_etb(
        &self,
        g: &mut Game,
        spell: ObjectId,
        etb: &mut crate::replacement::EtbInfo,
    ) {
        if !sneaked(g, spell) || g.combat.is_none() {
            return;
        }
        let target = g
            .saved_ctx
            .get(&spell)
            .and_then(|c| c.vars.get(&ATTACKED))
            .and_then(|v| v.first().copied());
        if let Some(target) = target {
            etb.tapped = true;
            etb.attacking = Some(target);
        }
    }
}

inventory::submit! { KeywordRegistration(&Sneak) }
