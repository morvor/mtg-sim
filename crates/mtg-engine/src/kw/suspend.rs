//! CR 702.62 Suspend. "Suspend N—[cost]" means "If you could begin to cast this card by
//! putting it onto the stack from your hand, you may pay [cost] and exile it with N time
//! counters on it. This action doesn't use the stack" (a special action, CR 116.2f), "At
//! the beginning of your upkeep, if this card is suspended, remove a time counter from
//! it," and "When the last time counter is removed from this card, if it's exiled, you
//! may play it without paying its mana cost if able. If you don't, it remains exiled. If
//! you cast a creature spell this way, it gains haste until you lose control of the spell
//! or the permanent it becomes."

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::{CastOption, Illegal};
use crate::decision::{Action, SpecialAction};
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::{Affected, ContinuousEffect, Game};
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::*;

pub struct Suspend;

/// The custom effect name of the last ability ("you may play it without paying its mana
/// cost").
pub const CAST_SUSPENDED: &str = "suspend_cast";

fn suspend_keyword(g: &Game, card: ObjectId) -> Option<Keyword> {
    g.obj(card)
        .chars
        .keywords()
        .find(|k| k.kind == KeywordKind::Suspend)
        .cloned()
}

/// "If you could begin to cast this card by putting it onto the stack from your hand",
/// considering effects that would prohibit casting it (CR 702.62a, 702.62c).
fn could_begin_to_cast(g: &Game, p: PlayerId, card: ObjectId) -> bool {
    let chars = g.face_characteristics(card, FaceState::Front);
    g.obj(card).zone == Zone::Hand(p)
        && g.timing_allows_cast(p, card, &chars, &CastOption::normal(FaceState::Front))
        && !g.cast_prohibited(p, card, &chars)
}

fn time_counters(sel: Sel) -> Value {
    Value::CountersOn(Box::new(sel), Some(counters::TIME.into()))
}

impl KeywordRules for Suspend {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Suspend]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        // CR 702.62b: suspended = in exile, with suspend, with a time counter on it.
        let suspended = Condition::And(vec![
            Condition::SelMatches(Sel::This, Filter::InZone(ZoneKind::Exile)),
            Condition::Compare(time_counters(Sel::This), Cmp::Gt, Value::c(0)),
        ]);
        let mut upkeep = TriggeredAbility::new(
            TriggerCond::BeginningOf {
                step: TriggerStep::Upkeep,
                whose: PlayerRel::You,
            },
            Body::effect(Effect::RemoveCounters {
                what: Sel::This,
                kind: Some(counters::TIME.into()),
                n: Value::c(1),
            }),
        );
        upkeep.intervening_if = Some(suspended);
        upkeep.zone = FunctionZone::Exile;
        let mut last = TriggeredAbility::new(
            TriggerCond::Where {
                trigger: Box::new(TriggerCond::CountersRemoved {
                    filter: Filter::Source,
                    kind: Some(counters::TIME.into()),
                }),
                cond: Condition::Compare(time_counters(Sel::This), Cmp::Eq, Value::c(0)),
            },
            Body::effect(Effect::Custom(CAST_SUSPENDED.into())),
        );
        last.intervening_if = Some(Condition::SelMatches(
            Sel::This,
            Filter::InZone(ZoneKind::Exile),
        ));
        last.zone = FunctionZone::Exile;
        Some(vec![
            AbilityDef::new(AbilityKind::Triggered(upkeep), "Suspend"),
            AbilityDef::new(AbilityKind::Triggered(last), "Suspend"),
        ])
    }

    fn special_actions(&self, g: &Game, p: PlayerId) -> Vec<Action> {
        g.player(p)
            .hand
            .iter()
            .copied()
            .filter(|c| g.has_priority(p) && could_begin_to_cast(g, p, *c))
            .filter(|c| {
                suspend_keyword(g, *c).is_some_and(|k| {
                    let cost = k.cost.unwrap_or_default();
                    g.can_pay_cost(p, &cost, Some(*c), &Ctx::new(Some(*c), p))
                })
            })
            .map(|card| Action::Special(SpecialAction::Suspend { card }))
            .collect()
    }

    fn perform_special_action(
        &self,
        g: &mut Game,
        p: PlayerId,
        sa: &SpecialAction,
    ) -> Option<Result<(), Illegal>> {
        let SpecialAction::Suspend { card } = sa else {
            return None;
        };
        let card = *card;
        let bad = |s: &str| Some(Err(Illegal(s.into())));
        if !g.is_live(card) || !g.has_priority(p) || !could_begin_to_cast(g, p, card) {
            return bad("can't suspend that card now");
        }
        let Some(kw) = suspend_keyword(g, card) else {
            return bad("no suspend");
        };
        let cost = kw.cost.clone().unwrap_or_default();
        if !crate::special_actions::pay(g, p, &cost, Some(card), &Ctx::new(Some(card), p)) {
            return bad("can't pay the suspend cost");
        }
        let new = g.move_object_ev(MoveEv {
            obj: card,
            to: Zone::Exile,
            pos: LibraryPosition::Top,
            cause: MoveCause::Exile,
            by: Some(p),
            etb: EtbInfo::default(),
            source: None,
        });
        if let Some(new) = new {
            let n = kw.n.unwrap_or(0).max(0) as u32;
            if n > 0 {
                g.add_counters(Entity::Object(new), counters::TIME, n, None);
            }
        }
        Some(Ok(()))
    }
}

/// The last ability of suspend: its controller may play the exiled card without paying
/// its mana cost; a creature spell cast this way gains haste (CR 702.62a).
pub fn cast_suspended(g: &mut Game, ctx: &mut Ctx) {
    let Some(card) = ctx.source else {
        return;
    };
    if !g.is_live(card) || g.obj(card).zone != Zone::Exile {
        return;
    }
    let p = g.obj(card).owner;
    if !g.ask_yes_no(p, Some(card), "Cast it without paying its mana cost?", true) {
        return;
    }
    let Ok(spell) = crate::casting::cast_during_resolution(g, p, card, CastMethod::Free) else {
        return;
    };
    if g.obj(spell).chars.is(CardType::Creature) {
        // The effect follows the spell to the permanent it becomes (CR 611.3d).
        let id = g.new_effect_id();
        let ts = g.new_timestamp();
        let turn = g.turn.number;
        g.effects.push(ContinuousEffect {
            id,
            source: Some(spell),
            controller: p,
            timestamp: ts,
            duration: Duration::Permanent,
            affected: Affected::Objects(vec![spell]),
            mods: vec![Modification::AddKeyword(Keyword::new(KeywordKind::Haste))],
            layer1: None,
            created_turn: turn,
        });
        g.carried_effects.push(id);
        g.dirty = true;
    }
}

inventory::submit! { KeywordRegistration(&Suspend) }
