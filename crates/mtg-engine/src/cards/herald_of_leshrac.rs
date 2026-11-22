//! Herald of Leshrac: "Cumulative upkeep—Gain control of a land you don't control."
//! Cumulative upkeep (CR 702.24a) whose cost is gaining control of a different land you
//! don't control for each age counter; with too few such lands it can't be paid and the
//! Herald is sacrificed. The lands are chosen as the ability resolves (rulings). Its
//! other abilities compile.

use super::ManualAbility;
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{CardType, Entity, ObjectId};

const PAY: &str = "card:Herald of Leshrac:gain control of a land you don't control for each age counter";
const TEXT: &str = "Cumulative upkeep—Gain control of a land you don't control.";
const LANDS: Var = vars::USER + 4460;

inventory::submit! { ManualAbility {
    card: "Herald of Leshrac",
    face: 0,
    text: TEXT,
    build: |_| {
        let age = crate::kw::cumulative_upkeep::AGE;
        let mut t = TriggeredAbility::new(
            TriggerCond::BeginningOf {
                step: TriggerStep::Upkeep,
                whose: PlayerRel::You,
            },
            Body::effect(Effect::Seq(vec![
                Effect::AddCounters {
                    what: Sel::This,
                    kind: age.into(),
                    n: Value::Const(1),
                },
                Effect::Custom(PAY.into()),
            ])),
        );
        t.intervening_if = Some(Condition::Exists(Filter::Source));
        vec![AbilityDef::new(AbilityKind::Triggered(t), TEXT)]
    },
    reason: "cumulative upkeep paid by gaining control of lands you don't control: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != PAY {
            return false;
        }
        let Some(src) = ctx.source else {
            return true;
        };
        let you = ctx.controller;
        let n = g.obj(src).counter(crate::kw::cumulative_upkeep::AGE);
        let lands: Vec<ObjectId> = g
            .permanents()
            .filter(|o| o.is(CardType::Land) && o.controller != you)
            .map(|o| o.id)
            .collect();
        let pay = n as usize <= lands.len()
            && g.ask_yes_no(
                you,
                Some(src),
                &format!("Pay cumulative upkeep: gain control of {n} land(s) you don't control?"),
                true,
            );
        if !pay {
            g.exec(&Effect::SacrificeObjects { what: Sel::This }, ctx);
            return true;
        }
        let chosen = g.ask_objects(you, Some(src), "Choose lands to gain control of", lands, n, n);
        ctx.set_var(LANDS, chosen.into_iter().map(Entity::Object).collect());
        g.exec(
            &Effect::GainControl {
                what: Sel::Var(LANDS),
                who: PlayerRef::You,
                duration: Duration::Permanent,
            },
            ctx,
        );
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
