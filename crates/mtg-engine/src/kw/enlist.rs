//! CR 702.154 Enlist: "As this creature attacks, you may tap up to one untapped creature
//! you control that you didn't choose to attack with and that either has haste or has been
//! under your control continuously since this turn began. When you do, this creature gets
//! +X/+0 until end of turn, where X is the tapped creature's power." (CR 702.154a).
//!
//! * The static ability is an optional cost to attack (CR 508.1g, 702.154b), paid as
//!   attackers are declared ([`KeywordRules::pay_attack_costs`]); the triggered ability is
//!   linked to it (CR 607.2h): it triggers only when that cost is paid, and X is the power
//!   of the creature tapped for it as the ability resolves (its last known information if
//!   it has left the battlefield).
//! * A creature "enlists" the creature tapped to pay its enlist cost (CR 702.154c); it
//!   can't enlist itself (it's attacking), and a creature can be tapped for only one
//!   enlist ability. Each enlist is recorded as an [`ENLISTED`] event (for "whenever ~
//!   enlists a creature" and "if it enlisted a creature this combat").
//! * Several instances of enlist on one creature function independently: each is paid
//!   separately and its own triggered ability triggers only for its own cost
//!   (CR 702.154d).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::{Game, PendingTrigger};
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::turn::Step;
use crate::types::*;

/// `Event::Custom`: `obj` (an attacking creature) enlisted a creature; `amount` is the
/// enlisted creature's object id.
pub const ENLISTED: &str = "enlisted";
/// `TriggerCond::Custom` of the enlist triggered ability: it triggers only as its cost is
/// paid (see [`Enlist::pay_attack_costs`]), never from other events.
pub const PAID: &str = "enlist:its enlist cost was paid";
/// `TriggerCond::Custom`: "whenever ~ enlists a creature".
pub const ENLISTS: &str = "enlist:~ enlists a creature";
/// `Condition::Custom`: "if it enlisted a creature this combat" (the attacking creature
/// of the triggering event).
pub const ENLISTED_THIS_COMBAT: &str = "enlist:it enlisted a creature this combat";
/// The creature tapped for the enlist cost, in the triggered ability's context.
const TAPPED: Var = vars::USER + 1540;

pub struct Enlist;

/// The creatures `p` could tap for an enlist ability as the creatures `attacking` attack.
fn candidates(g: &Game, p: PlayerId, attacking: &[ObjectId]) -> Vec<ObjectId> {
    g.permanents()
        .filter(|o| {
            o.controller == p
                && o.is_creature()
                && !o.tapped
                && !attacking.contains(&o.id)
                && (!o.summoning_sick || o.has_keyword(KeywordKind::Haste))
        })
        .map(|o| o.id)
        .collect()
}

/// Whether `attacker` enlisted a creature this combat (since the last beginning of combat
/// step began).
pub fn enlisted_this_combat(g: &Game, attacker: ObjectId) -> bool {
    let start = g
        .turn_events
        .iter()
        .rposition(|e| {
            matches!(
                e,
                Event::StepBegan {
                    step: Step::BeginningOfCombat,
                    ..
                }
            )
        })
        .unwrap_or(0);
    g.turn_events[start..].iter().any(|e| {
        matches!(e, Event::Custom { name, obj: Some(o), .. }
            if name == ENLISTED && *o == attacker)
    })
}

impl KeywordRules for Enlist {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Enlist]
    }

    /// The linked triggered ability: "When you do, this creature gets +X/+0 until end of
    /// turn, where X is the tapped creature's power."
    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::Custom(PAID.into()),
                Body::effect(Effect::Modify {
                    what: Sel::This,
                    mods: vec![Modification::ModifyPT(
                        Value::PowerOf(Box::new(Sel::Var(TAPPED))),
                        Value::c(0),
                    )],
                    duration: Duration::EndOfTurn,
                }),
            )),
            KeywordKind::Enlist.name(),
        )])
    }

    fn pay_attack_costs(&self, g: &mut Game, ap: PlayerId, declared: &[(ObjectId, Entity)]) {
        let attacking: Vec<ObjectId> = declared.iter().map(|(a, _)| *a).collect();
        for &a in &attacking {
            if !g.is_live(a) || g.obj(a).controller != ap {
                continue;
            }
            g.recompute();
            // Each instance separately, with its own linked triggered ability.
            let chars = g.obj(a).chars.clone();
            let instances: Vec<u64> = chars
                .abilities
                .iter()
                .filter(|ab| {
                    matches!(&ab.kind, AbilityKind::Keyword(k) if k.kind == KeywordKind::Enlist)
                })
                .map(|ab| ab.uid)
                .collect();
            let derived = crate::keyword_impls::derived_by_keyword(&chars);
            for kw_uid in instances {
                let Some(ability) = derived
                    .iter()
                    .find(|(k, d)| {
                        *k == kw_uid
                            && matches!(&d.kind, AbilityKind::Triggered(t)
                                if matches!(&t.trigger, TriggerCond::Custom(n) if n == PAID))
                    })
                    .map(|(_, d)| d.clone())
                else {
                    continue;
                };
                let options = candidates(g, ap, &attacking);
                if options.is_empty() {
                    continue;
                }
                let name = g.obj(a).chars.name.clone();
                let chosen = g.ask_objects(
                    ap,
                    Some(a),
                    &format!("Enlist: tap up to one creature for {name}"),
                    options,
                    0,
                    1,
                );
                let Some(&tapped) = chosen.first() else {
                    continue;
                };
                if !g.tap(tapped) {
                    continue;
                }
                g.log(|g| format!("{name} enlists {}", g.obj(tapped).chars.name));
                g.emit(Event::Custom {
                    name: ENLISTED.into(),
                    player: Some(ap),
                    obj: Some(a),
                    amount: tapped.0 as i32,
                });
                // CR 603.12, 607.2h: "When you do" — the linked triggered ability.
                let mut saved = Ctx::new(Some(a), ap);
                saved.ability_uid = ability.uid;
                saved.vars.insert(TAPPED, vec![Entity::Object(tapped)]);
                g.trigger_order += 1;
                let order = g.trigger_order;
                g.pending_triggers.push(PendingTrigger {
                    source: a,
                    controller: ap,
                    ability,
                    event: EventInfo {
                        object: Some(a),
                        other: Some(tapped),
                        player: Some(ap),
                        ..Default::default()
                    },
                    source_lki: None,
                    saved: Some(saved),
                    body: None,
                    order,
                });
            }
        }
    }

    fn custom_trigger(
        &self,
        _g: &Game,
        name: &str,
        src: ObjectId,
        _ctl: PlayerId,
        ev: &Event,
    ) -> Option<Vec<EventInfo>> {
        match name {
            PAID => Some(vec![]),
            ENLISTS => {
                let Event::Custom {
                    name: n,
                    obj: Some(a),
                    player,
                    amount,
                } = ev
                else {
                    return Some(vec![]);
                };
                if n != ENLISTED || *a != src {
                    return Some(vec![]);
                }
                Some(vec![EventInfo {
                    object: Some(ObjectId(*amount as u32)),
                    other: Some(*a),
                    player: *player,
                    ..Default::default()
                }])
            }
            _ => None,
        }
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != ENLISTED_THIS_COMBAT {
            return None;
        }
        let it = ctx.event.as_ref().and_then(|e| e.object);
        Some(it.is_some_and(|a| enlisted_this_combat(g, a)))
    }
}

inventory::submit! { KeywordRegistration(&Enlist) }
