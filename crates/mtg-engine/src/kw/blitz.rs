//! CR 702.152 Blitz: "Blitz [cost]" means "You may cast this card by paying [cost] rather
//! than its mana cost," "If this spell's blitz cost was paid, sacrifice the permanent this
//! spell becomes at the beginning of the next end step," and "As long as this permanent's
//! blitz cost was paid, it has haste and 'When this permanent is put into a graveyard from
//! the battlefield, draw a card.'" (CR 702.152a). The blitz cost is an alternative cost
//! (CR 601.2b, 601.2f–h): the spell is cast only when it otherwise could be, and its mana
//! value is still that of its mana cost.
//!
//! The delayed triggered ability is created as the spell resolves and the permanent
//! enters (CR 608.3g); it sacrifices only that permanent, if it's still on the
//! battlefield. A copy of a permanent whose blitz cost was paid wasn't cast for its blitz
//! cost: it has neither haste nor the draw ability, and isn't sacrificed.
//!
//! Each instance of blitz records its own payment (CR 702.152b): the name recorded in
//! `CastInfo::paid` identifies the instance by its cost, so only the abilities of the
//! instance that was used apply.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::CastOption;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

pub struct Blitz;

/// The name recorded in `CastInfo::paid` when a spell is cast for this instance's blitz
/// cost (CR 702.152b: each instance refers only to its own payment).
pub fn paid_tag(kw: &Keyword) -> &'static str {
    let cost = match &kw.cost {
        Some(c) => {
            let mana = c.mana.as_ref().map(|m| m.to_string()).unwrap_or_default();
            if c.parts.is_empty() {
                mana
            } else {
                format!("{mana} {:?}", c.parts)
            }
        }
        None => String::new(),
    };
    intern(format!("blitz {cost}"))
}

/// A `&'static str` for a tag (`CastOption::tag`); there are few distinct blitz costs.
fn intern(s: String) -> &'static str {
    static TAGS: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();
    let mut tags = TAGS.get_or_init(Default::default).lock().unwrap();
    if let Some(t) = tags.get(s.as_str()) {
        return t;
    }
    let t: &'static str = Box::leak(s.into_boxed_str());
    tags.insert(t);
    t
}

impl KeywordRules for Blitz {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Blitz]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let text = KeywordKind::Blitz.name();
        let paid = Condition::CostPaid(paid_tag(kw).into());
        // "If this spell's blitz cost was paid, sacrifice the permanent this spell becomes
        // at the beginning of the next end step": functions on the stack (CR 608.3g).
        let mut sac = StaticAbility::new(StaticEffect::DelayedTriggerAsEnters {
            condition: Some(paid.clone()),
            trigger: TriggerCond::BeginningOf {
                step: TriggerStep::End,
                whose: PlayerRel::Any,
            },
            body: Body::effect(Effect::SacrificeObjects {
                what: Sel::Var(vars::IT),
            }),
        });
        sac.zone = FunctionZone::Stack;
        // "As long as this permanent's blitz cost was paid, it has haste and 'When this
        // permanent is put into a graveyard from the battlefield, draw a card.'": the
        // permanent's own static ability (layer 6), so a copy of it (which wasn't cast for
        // its blitz cost) has neither.
        let draw = AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::ZoneChange {
                    filter: Filter::Source,
                    from: Some(ZoneKind::Battlefield),
                    to: Some(ZoneKind::Graveyard),
                },
                Body::effect(Effect::Draw {
                    who: PlayerRef::You,
                    n: Value::c(1),
                }),
            )),
            "When this permanent is put into a graveyard from the battlefield, draw a card.",
        );
        let mut grant = StaticAbility::new(StaticEffect::Continuous {
            affected: Filter::Source,
            mods: vec![
                Modification::AddKeyword(Keyword::new(KeywordKind::Haste)),
                Modification::AddAbility(draw),
            ],
        });
        grant.condition = Some(paid);
        Some(vec![
            AbilityDef::new(AbilityKind::Static(sac), text),
            AbilityDef::new(AbilityKind::Static(grant), text),
        ])
    }

    /// CR 702.152a–b: an alternative cost; each instance of blitz is its own way to cast
    /// the spell, and only one of them may be used.
    fn cast_options(&self, g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<CastOption> {
        let o = g.obj(card);
        let Some(cost) = kw.cost.clone() else {
            return vec![];
        };
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::Keyword(KeywordKind::Blitz);
        // Only from a zone the card could be cast from (its owner's hand, or a zone an
        // effect lets them cast it from).
        if o.zone != Zone::Hand(p) {
            let chars = g.option_characteristics(card, &opt);
            if !g.permitted_cards(p).contains(&card) || !g.permission_allows(p, card, &chars, false)
            {
                return vec![];
            }
        }
        // "Blitz costs you pay cost {1} less ..." (Henzie "Toolbox" Torre).
        opt.alt_cost = Some(super::modified_keyword_cost(
            g,
            p,
            KeywordKind::Blitz,
            &cost,
        ));
        opt.tag = Some(paid_tag(kw));
        vec![opt]
    }
}

inventory::submit! { KeywordRegistration(&Blitz) }
