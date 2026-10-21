//! Johan: "At the beginning of combat on your turn, you may have ~ gain "~ can't attack"
//! until end of combat. If you do, attacking doesn't cause creatures you control to tap
//! this combat if ~ is untapped."

use super::ManualAbility;
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;
use crate::types::ObjectId;

const NO_TAP: &str = "card:Johan:attacking doesn't cause creatures you control to tap this combat";
const TEXT: &str = "At the beginning of combat on your turn, you may have ~ gain \"~ can't attack\" until end of combat. If you do, attacking doesn't cause creatures you control to tap this combat if ~ is untapped.";

inventory::submit! { ManualAbility {
    card: "Johan",
    face: 0,
    text: TEXT,
    build: |_| {
        let cant_attack = Effect::AddRestriction {
            restriction: Restriction::CantAttack(Filter::In(Box::new(Sel::This))),
            duration: Duration::EndOfCombat,
        };
        let body = Body::effect(Effect::May {
            who: PlayerRef::You,
            effect: Box::new(Effect::Seq(vec![cant_attack, Effect::Custom(NO_TAP.into())])),
        });
        vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::BeginningOf {
                    step: TriggerStep::BeginningOfCombat,
                    whose: PlayerRel::You,
                },
                body,
            )),
            TEXT,
        )]
    },
    reason: "creatures you control attack without tapping this combat while it's untapped: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    /// Records (turn, combat, Johan).
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != NO_TAP {
            return false;
        }
        let Some(src) = ctx.source else {
            return true;
        };
        let turn = i64::from(g.turn.number);
        let combat = i64::from(g.turn.combat_phases);
        g.cards.retain(NO_TAP, |r| r[0] == turn && r[1] == combat);
        g.cards.push(NO_TAP, vec![turn, combat, i64::from(src.0)]);
        true
    }
    fn attacks_without_tapping(&self, g: &Game, attacker: ObjectId) -> bool {
        let turn = i64::from(g.turn.number);
        let combat = i64::from(g.turn.combat_phases);
        let ctl = g.obj(attacker).controller;
        g.cards.get(NO_TAP).iter().any(|r| {
            let johan = ObjectId(r[2] as u32);
            r[0] == turn
                && r[1] == combat
                && g.is_live(johan)
                && g.obj(johan).zone == Zone::Battlefield
                && !g.obj(johan).tapped
                && g.obj(johan).controller == ctl
        })
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
