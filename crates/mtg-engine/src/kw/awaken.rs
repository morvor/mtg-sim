//! CR 702.113 Awaken: "Awaken N—[cost]" means "You may pay [cost] rather than pay this
//! spell's mana cost as you cast this spell" and "If this spell's awaken cost was paid,
//! put N +1/+1 counters on target land you control. That land becomes a 0/0 Elemental
//! creature with haste. It's still a land." (CR 702.113a).
//!
//! * The first is an alternative cost (CR 601.2b, 601.2f–h): a way of casting the card
//!   ([`CastMethod::Keyword`] `Awaken`), recorded as [`AWAKEN`] in the spell's paid costs.
//! * The second is a spell ability the keyword stands for, added after the spell's other
//!   spell abilities (so it's followed after them, with its own target slot). Its target
//!   is chosen only if the awaken cost was paid; otherwise the spell is cast as though it
//!   didn't have that target (CR 702.113b): the target has that condition.
//! * The land keeps its other types, subtypes, and abilities, and doesn't get a color.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::CastOption;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;

/// The name recorded in `CastInfo::paid` when a spell is cast for its awaken cost.
pub const AWAKEN: &str = "awaken";

/// "Put N +1/+1 counters on target land you control. That land becomes a 0/0 Elemental
/// creature with haste. It's still a land." — only if the awaken cost was paid.
fn awaken_ability(n: i32) -> Ability {
    let paid = Condition::CostPaid(AWAKEN.into());
    let mut target = TargetSpec::object(
        Filter::and(vec![
            Filter::Type(CardType::Land),
            Filter::ControlledBy(PlayerRel::You),
        ]),
        "target land you control",
    );
    target.condition = Some(paid.clone());
    let land = Sel::Target(0);
    let effect = Effect::If {
        cond: paid,
        then: Box::new(Effect::Seq(vec![
            Effect::AddCounters {
                what: land.clone(),
                kind: counters::PLUS1.into(),
                n: Value::c(n),
            },
            Effect::Modify {
                what: land,
                mods: vec![
                    Modification::AddTypes(vec![CardType::Creature]),
                    Modification::AddSubtypes(vec![Subtype::new("Elemental")]),
                    Modification::SetPT(Some(Value::c(0)), Some(Value::c(0))),
                    Modification::AddKeyword(Keyword::new(KeywordKind::Haste)),
                ],
                duration: Duration::Permanent,
            },
        ])),
        otherwise: Box::new(Effect::Noop),
    };
    AbilityDef::new(
        AbilityKind::Spell(SpellAbility {
            body: Body::simple(vec![target], effect),
        }),
        KeywordKind::Awaken.name(),
    )
}

pub struct Awaken;

impl KeywordRules for Awaken {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Awaken]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        Some(vec![awaken_ability(kw.n.unwrap_or(0))])
    }

    fn cast_options(&self, g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<CastOption> {
        let Some(cost) = kw.cost.clone() else {
            return vec![];
        };
        let o = g.obj(card);
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::Keyword(KeywordKind::Awaken);
        if o.zone != Zone::Hand(p) {
            let chars = g.option_characteristics(card, &opt);
            if !g.permitted_cards(p).contains(&card) || !g.permission_allows(p, card, &chars, false)
            {
                return vec![];
            }
        }
        // The awaken ability needs a land its caster controls to target (CR 601.2c).
        if !g.permanents().any(|o| o.controller == p && o.chars.is_land()) {
            return vec![];
        }
        opt.alt_cost = Some(super::modified_keyword_cost(
            g,
            p,
            KeywordKind::Awaken,
            &cost,
        ));
        opt.tag = Some(AWAKEN);
        vec![opt]
    }
}

inventory::submit! { KeywordRegistration(&Awaken) }
