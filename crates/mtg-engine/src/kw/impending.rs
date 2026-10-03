//! CR 702.176 Impending: "Impending N—[cost]" represents four abilities (CR 702.176a):
//!
//! 1. "You may choose to pay [cost] rather than pay this spell's mana cost" — an
//!    alternative cost (CR 601.2b, 601.2f–h), a way to cast the card
//!    ([`KeywordRules::cast_options`]) recorded as [`IMPENDING`] in `CastInfo::paid`;
//! 2. "If you chose to pay this permanent's impending cost, it enters with N time counters
//!    on it" — a replacement effect as it enters from the stack;
//! 3. "As long as this permanent's impending cost was paid and it has a time counter on
//!    it, it's not a creature" — a static ability (layer 4);
//! 4. "At the beginning of your end step, if this permanent's impending cost was paid and
//!    it has a time counter on it, remove a time counter from it."
//!
//! The permanent sees how the spell it was cast as was paid for (CR 400.7d); an object that
//! enters as a copy of it wasn't cast for its impending cost, so it doesn't get counters
//! and is a creature.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::CastOption;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;

/// The name recorded in `CastInfo::paid` when a spell is cast for its impending cost.
pub const IMPENDING: &str = "impending";

fn paid() -> Condition {
    Condition::CostPaid(IMPENDING.into())
}

fn has_time_counter() -> Condition {
    Condition::Compare(
        Value::CountersOn(Box::new(Sel::This), Some(counters::TIME.into())),
        Cmp::Gt,
        Value::c(0),
    )
}

pub struct Impending;

impl KeywordRules for Impending {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Impending]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let text = KeywordKind::Impending.name();
        let n = kw.n.unwrap_or(0).max(0);
        // "If you chose to pay this permanent's impending cost, it enters with N time
        // counters on it."
        let enters = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(Filter::Source),
            action: ReplacementAction::AsEnters(Box::new(Effect::If {
                cond: paid(),
                then: Box::new(Effect::EnterWithCounters {
                    kind: counters::TIME.into(),
                    n: Value::c(n),
                }),
                otherwise: Box::new(Effect::Noop),
            })),
            self_replacement: false,
            optional: false,
        }));
        // "As long as this permanent's impending cost was paid and it has a time counter on
        // it, it's not a creature."
        let mut not_creature = StaticAbility::new(StaticEffect::Continuous {
            affected: Filter::Source,
            mods: vec![Modification::RemoveTypes(vec![CardType::Creature])],
        });
        not_creature.condition = Some(Condition::And(vec![paid(), has_time_counter()]));
        // "At the beginning of your end step, if this permanent's impending cost was paid
        // and it has a time counter on it, remove a time counter from it."
        let mut end_step = TriggeredAbility::new(
            TriggerCond::BeginningOf {
                step: TriggerStep::End,
                whose: PlayerRel::You,
            },
            Body::effect(Effect::RemoveCounters {
                what: Sel::This,
                kind: Some(counters::TIME.into()),
                n: Value::c(1),
            }),
        );
        end_step.intervening_if = Some(Condition::And(vec![paid(), has_time_counter()]));
        Some(vec![
            AbilityDef::new(AbilityKind::Static(enters), text),
            AbilityDef::new(AbilityKind::Static(not_creature), text),
            AbilityDef::new(AbilityKind::Triggered(end_step), text),
        ])
    }

    /// "You may choose to pay [cost] rather than pay this spell's mana cost": from any zone
    /// the card could be cast from.
    fn cast_options(&self, g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<CastOption> {
        let Some(cost) = kw.cost.clone() else {
            return vec![];
        };
        let o = g.obj(card);
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::Keyword(KeywordKind::Impending);
        if o.zone != Zone::Hand(p) {
            let chars = g.option_characteristics(card, &opt);
            if !g.permitted_cards(p).contains(&card) || !g.permission_allows(p, card, &chars, false)
            {
                return vec![];
            }
        }
        opt.alt_cost = Some(cost);
        opt.tag = Some(IMPENDING);
        vec![opt]
    }
}

inventory::submit! { KeywordRegistration(&Impending) }
