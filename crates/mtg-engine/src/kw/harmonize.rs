//! CR 702.180 Harmonize.
//!
//! * "Harmonize [cost]" means "You may cast this card from your graveyard by paying
//!   [cost] and tapping up to one untapped creature you control rather than paying this
//!   spell's mana cost," "If you cast this spell using its harmonize ability, its total
//!   cost is reduced by an amount of generic mana equal to the tapped creature's power,"
//!   and "If the harmonize cost was paid, exile this card instead of putting it anywhere
//!   else any time it would leave the stack." (CR 702.180a). It's an alternative cost
//!   (CR 601.2b, 601.2f–h): timing restrictions still apply, and the spell's mana value
//!   doesn't change.
//! * The creature to tap is chosen as the player chooses to pay the harmonize cost
//!   (CR 601.2b, [`KeywordRules::announce`]) and tapped as the total cost is paid
//!   (CR 702.180b, 601.2h): it's still untapped while the total cost is determined, which
//!   is reduced by its power then. Only generic mana is reduced.
//! * A harmonize ability granted with "Its harmonize cost is equal to its mana cost" has
//!   no cost of its own ([`Keyword::cost`] is `None`): the card's mana cost is paid.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::{CastOption, Illegal};
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;

/// The name recorded in `CastInfo::paid` when a spell is cast using its harmonize
/// ability.
pub const HARMONIZE: &str = "harmonize";

/// The variable of the spell's saved context holding the creature chosen to be tapped.
const CHOSEN: Var = vars::USER + 180;

fn is_harmonize(method: &CastMethod) -> bool {
    *method == CastMethod::Keyword(KeywordKind::Harmonize)
}

/// Whether the spell was cast using its harmonize ability.
pub fn cast_with_harmonize(g: &Game, spell: ObjectId) -> bool {
    g.obj(spell)
        .stack
        .as_deref()
        .is_some_and(|s| is_harmonize(&s.cast.method))
}

/// Untapped creatures `p` controls that could be tapped for the harmonize cost of `card`.
fn candidates(g: &Game, p: PlayerId, card: ObjectId) -> Vec<ObjectId> {
    g.permanents()
        .filter(|o| o.controller == p && o.id != card && o.is_creature() && !o.tapped)
        .map(|o| o.id)
        .collect()
}

/// The creature chosen to be tapped for the harmonize cost of the spell `spell`.
fn chosen(g: &Game, spell: ObjectId) -> Option<ObjectId> {
    g.saved_ctx
        .get(&spell)
        .and_then(|c| c.vars.get(&CHOSEN))
        .and_then(|v| v.first())
        .and_then(|e| e.object())
}

pub struct Harmonize;

impl KeywordRules for Harmonize {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Harmonize]
    }

    /// "If the harmonize cost was paid, exile this card instead of putting it anywhere
    /// else any time it would leave the stack."
    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let mut s = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::ZoneChange {
                filter: Filter::Source,
                from: Some(ZoneKind::Stack),
                to: None,
            },
            action: ReplacementAction::MoveInstead(Destination::zone(ZoneKind::Exile)),
            self_replacement: false,
            optional: false,
        }));
        s.zone = FunctionZone::Stack;
        s.condition = Some(Condition::CostPaid(HARMONIZE.into()));
        Some(vec![AbilityDef::new(
            AbilityKind::Static(s),
            KeywordKind::Harmonize.name(),
        )])
    }

    /// "You may cast this card from your graveyard by paying [cost] ... rather than paying
    /// this spell's mana cost."
    fn cast_options(&self, g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<CastOption> {
        if !crate::as_though::in_graveyard_for(g, p, card) {
            return vec![];
        }
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::Keyword(KeywordKind::Harmonize);
        let cost = match &kw.cost {
            Some(c) => c.clone(),
            None => {
                // A card with no mana cost has an unpayable harmonize cost (CR 118.6).
                let chars = g.option_characteristics(card, &opt);
                Cost::mana(
                    chars
                        .mana_cost
                        .clone()
                        .unwrap_or_else(crate::cost_rules::unpayable),
                )
            }
        };
        opt.alt_cost = Some(super::modified_keyword_cost(
            g,
            p,
            KeywordKind::Harmonize,
            &cost,
        ));
        opt.tag = Some(HARMONIZE);
        vec![opt]
    }

    /// CR 702.180b, 601.2b: up to one untapped creature to tap is chosen now; tapping it
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
        if !is_harmonize(method) {
            return Ok(());
        }
        let cands = candidates(g, p, spell);
        if cands.is_empty() {
            return Ok(());
        }
        let entities: Vec<Entity> = cands.iter().map(|c| Entity::Object(*c)).collect();
        let pick = match g.ask(
            p,
            Decision::ChooseEntities {
                source: Some(spell),
                prompt: "Choose up to one untapped creature to tap (harmonize)".into(),
                candidates: entities.clone(),
                min: 0,
                max: 1,
            },
        ) {
            Answer::Entities(v) if v.is_empty() => None,
            Answer::Entities(v) if v.len() == 1 && entities.contains(&v[0]) => v[0].object(),
            // By default, the one that reduces the cost most.
            _ => cands.iter().copied().max_by_key(|c| g.obj(*c).power()),
        };
        let Some(pick) = pick else {
            return Ok(());
        };
        g.saved_ctx
            .entry(spell)
            .or_insert_with(|| Ctx::new(Some(spell), p))
            .vars
            .insert(CHOSEN, vec![Entity::Object(pick)]);
        crate::casting::add_cost(
            extra,
            &Cost::free().with(CostPart::TapUntapped {
                filter: Filter::Objects(vec![pick]),
                count: Value::c(1),
            }),
        );
        g.log(|g| format!("{p} will tap {} (harmonize)", g.describe(pick)));
        Ok(())
    }

    /// CR 702.180a: once the total cost is determined (CR 601.2f), it's reduced by
    /// generic mana equal to the chosen creature's power.
    fn pay_mana_otherwise(
        &self,
        g: &mut Game,
        _p: PlayerId,
        spell: ObjectId,
        _kw: &Keyword,
        cost: &mut Cost,
    ) -> Result<(), Illegal> {
        if !cast_with_harmonize(g, spell) {
            return Ok(());
        }
        if let (Some(c), Some(m)) = (chosen(g, spell), cost.mana.as_mut()) {
            m.reduce_generic(g.obj(c).power().max(0) as u32);
        }
        Ok(())
    }

    /// For the check whether it could be cast using harmonize: the best reduction.
    fn payable_otherwise(
        &self,
        g: &Game,
        p: PlayerId,
        card: ObjectId,
        _kw: &Keyword,
        method: &CastMethod,
        cost: &mut Cost,
    ) {
        if !is_harmonize(method) {
            return;
        }
        let best = candidates(g, p, card)
            .into_iter()
            .map(|c| g.obj(c).power().max(0) as u32)
            .max()
            .unwrap_or(0);
        if let Some(m) = cost.mana.as_mut() {
            m.reduce_generic(best);
        }
    }

    /// "Exile it instead of putting it anywhere else" applies wherever the card would go,
    /// after any other replacement effect: its controller has nothing to choose.
    fn resolved_destination_replaces(&self) -> bool {
        false
    }

    fn resolved_destination(
        &self,
        g: &Game,
        spell: ObjectId,
        _kw: &Keyword,
    ) -> Option<(Zone, LibraryPosition)> {
        cast_with_harmonize(g, spell).then_some((Zone::Exile, LibraryPosition::Top))
    }

    fn countered_destination(
        &self,
        g: &Game,
        spell: ObjectId,
        _kw: &Keyword,
    ) -> Option<(Zone, LibraryPosition)> {
        cast_with_harmonize(g, spell).then_some((Zone::Exile, LibraryPosition::Top))
    }
}

inventory::submit! { KeywordRegistration(&Harmonize) }
