//! CR 702.72 Champion: "Champion an [object]" means "When this permanent enters, sacrifice
//! it unless you exile another [object] you control" and "When this permanent leaves the
//! battlefield, return the exiled card to the battlefield under its owner's control". The
//! two abilities are linked (CR 607.2k, 702.72b): the second returns only the card the
//! first exiled.
//!
//! A permanent is "championed" by another permanent if the latter exiles the former as the
//! direct result of a champion ability (CR 702.72c): the champion ability reports it with
//! an [`Event::Custom`] named [`CHAMPIONED_EVENT`], which "When a Faerie is championed
//! with ~" ([`TriggerCond::Custom`] [`CHAMPIONED`]) triggers on.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::EventInfo;
use crate::types::*;

/// `Event::Custom` name: a permanent was championed; `obj` is the exiled card.
pub const CHAMPIONED_EVENT: &str = "championed";
/// `Effect::Custom`: reports the cards the champion ability just exiled ("it") as
/// championed.
const REPORT: &str = "champion:report championed";
/// `TriggerCond::Custom`: "When [something] is championed with ~": a permanent was exiled
/// by this permanent's champion ability. The trigger object is the exiled card, and the
/// trigger's last known information is the permanent as it was championed.
pub const CHAMPIONED: &str = "champion:championed with this";

pub struct Champion;

impl KeywordRules for Champion {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Champion]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let what = kw.filter.clone().unwrap_or(Filter::Permanent);
        const V: Var = vars::USER + 70;
        let enters = TriggeredAbility::new(
            TriggerCond::EntersBattlefield(Filter::Source),
            Body::effect(Effect::Seq(vec![
                Effect::Exile {
                    what: Sel::Choose {
                        chooser: PlayerRef::You,
                        filter: Filter::And(vec![
                            what,
                            Filter::Permanent,
                            Filter::Other,
                            Filter::ControlledBy(PlayerRel::You),
                        ]),
                        count: Value::c(1),
                        up_to: true,
                        store: None,
                    },
                    face_down: false,
                    link: true,
                },
                Effect::Custom(REPORT.into()),
                Effect::If {
                    cond: Condition::Not(Box::new(Condition::PrevAffectedAny)),
                    then: Box::new(Effect::SacrificeObjects { what: Sel::This }),
                    otherwise: Box::new(Effect::Noop),
                },
            ])),
        );
        // "The exiled card": only while it's still exiled (a card that left exile is a new
        // object, CR 400.7).
        let leaves = TriggeredAbility::new(
            TriggerCond::LeavesBattlefield(Filter::Source),
            Body::effect(Effect::ForEach {
                sel: Sel::All(Filter::and(vec![
                    Filter::In(Box::new(Sel::Linked)),
                    Filter::InZone(ZoneKind::Exile),
                ])),
                var: V,
                effect: Box::new(Effect::Move {
                    what: Sel::Var(V),
                    to: {
                        let mut d = Destination::battlefield();
                        d.controller = Some(PlayerRef::OwnerOf(Box::new(Sel::Var(V))));
                        d
                    },
                }),
            }),
        );
        Some(vec![
            AbilityDef::new(AbilityKind::Triggered(enters), "Champion"),
            AbilityDef::new(AbilityKind::Triggered(leaves), "Champion"),
        ])
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != REPORT {
            return false;
        }
        // Only a card exiled by the ability of a permanent still on the battlefield is
        // linked to it (and so "championed with" it).
        for o in ctx.var_objects(vars::IT) {
            g.emit(Event::Custom {
                name: CHAMPIONED_EVENT.into(),
                player: Some(ctx.controller),
                obj: Some(o),
                amount: 0,
            });
        }
        true
    }

    fn custom_trigger(
        &self,
        g: &Game,
        name: &str,
        src: ObjectId,
        ctl: PlayerId,
        ev: &Event,
    ) -> Option<Vec<EventInfo>> {
        if name != CHAMPIONED {
            return None;
        }
        let Event::Custom {
            name: n,
            obj: Some(o),
            ..
        } = ev
        else {
            return Some(vec![]);
        };
        if n != CHAMPIONED_EVENT || !g.obj(src).linked.values().any(|v| v.contains(o)) {
            return Some(vec![]);
        }
        Some(vec![EventInfo {
            object: Some(*o),
            lki: g.obj(*o).prev,
            player: Some(ctl),
            ..Default::default()
        }])
    }
}

inventory::submit! { KeywordRegistration(&Champion) }
