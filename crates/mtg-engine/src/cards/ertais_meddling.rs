//! Ertai's Meddling: "Target spell's controller exiles it with X delay counters on it." and
//! "At the beginning of each of that player's upkeeps, if that card is exiled, remove a
//! delay counter from it. If the card has no delay counters on it, the player puts it onto
//! the stack as a copy of the original spell." The card is put onto the stack (not cast)
//! with the original spell's choices (targets, modes, X, costs paid); a copy of a spell
//! ceases to exist when exiled (rulings).

use super::{spell, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::{Event, MoveCause};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::{EventInfo, ObjKind, Zone};
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::{Entity, ObjectId, PlayerId};

const EXILE: &str = "card:Ertai's Meddling:exile the spell with X delay counters";
const SCHEDULE: &str = "card:Ertai's Meddling:the delay";
const UPKEEP: &str = "card:Ertai's Meddling:that player's upkeep while it's exiled";
const TICK: &str = "card:Ertai's Meddling:remove a delay counter; at none, put it onto the stack";
const DELAY: &str = "delay";
const EXILE_TEXT: &str = "Target spell's controller exiles it with X delay counters on it.";
const DELAY_TEXT: &str = "At the beginning of each of that player's upkeeps, if that card is exiled, remove a delay counter from it. If the card has no delay counters on it, the player puts it onto the stack as a copy of the original spell.";

inventory::submit! { ManualAbility {
    card: "Ertai's Meddling",
    face: 0,
    text: EXILE_TEXT,
    build: |_| {
        vec![spell(
            vec![TargetSpec::one(TargetKind::Spell(Filter::Spell), "target spell")],
            Effect::Custom(EXILE.into()),
            EXILE_TEXT,
        )]
    },
    reason: "exiles a spell with delay counters to be put back onto the stack later: only this card",
} }

inventory::submit! { ManualAbility {
    card: "Ertai's Meddling",
    face: 0,
    text: DELAY_TEXT,
    build: |_| vec![spell(vec![], Effect::Custom(SCHEDULE.into()), DELAY_TEXT)],
    reason: "puts the delayed card back onto the stack as a copy of the original spell: only this card",
} }

struct Rules;

/// Rows (Meddling, exiled card, player, original spell).
fn rows_of(g: &Game, src: ObjectId) -> Vec<Vec<i64>> {
    g.cards
        .get(EXILE)
        .iter()
        .filter(|r| r[0] == i64::from(src.0))
        .cloned()
        .collect()
}

fn exiled(g: &Game, c: ObjectId) -> bool {
    g.is_live(c) && g.obj(c).zone == Zone::Exile
}

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        match name {
            EXILE => {
                let (Some(src), Some(Entity::Object(spell))) =
                    (ctx.source, ctx.targets.first().and_then(|v| v.first()).copied())
                else {
                    return true;
                };
                let p = g.obj(spell).controller;
                let copy = g.obj(spell).kind == ObjKind::SpellCopy;
                let Some(card) = g.move_object_ev(MoveEv {
                    obj: spell,
                    to: Zone::Exile,
                    pos: LibraryPosition::Top,
                    cause: MoveCause::Exile,
                    by: Some(p),
                    etb: EtbInfo::default(),
                    source: Some(src),
                }) else {
                    return true;
                };
                if copy || !exiled(g, card) {
                    return true;
                }
                let x = ctx.x.max(0) as u32;
                if x > 0 {
                    g.put_counters(
                        Entity::Object(card),
                        DELAY,
                        x,
                        crate::event_causes::CounterPut {
                            source: Some(src),
                            by: Some(p),
                            origin: crate::events::CounterOrigin::Effect,
                        },
                    );
                }
                g.cards.push(
                    EXILE,
                    vec![i64::from(src.0), i64::from(card.0), i64::from(p.0), i64::from(spell.0)],
                );
                true
            }
            SCHEDULE => {
                super::delayed_for_rest_of_game(
                    g,
                    ctx,
                    TriggerCond::Custom(UPKEEP.into()),
                    Body::effect(Effect::Custom(TICK.into())),
                );
                true
            }
            TICK => {
                let (Some(src), Some(card)) = (ctx.source, ctx.event.as_ref().and_then(|e| e.object))
                else {
                    return true;
                };
                let Some(row) = rows_of(g, src).into_iter().find(|r| r[1] == i64::from(card.0)) else {
                    return true;
                };
                if !exiled(g, card) {
                    return true;
                }
                let p = PlayerId(row[2] as u8);
                g.remove_counters_by(Entity::Object(card), DELAY, 1, Some(p));
                if g.obj(card).counter(DELAY) == 0 {
                    g.cards.retain(EXILE, |r| *r != row);
                    put_back_on_stack(g, card, ObjectId(row[3] as u32), p);
                }
                true
            }
            _ => false,
        }
    }
    fn custom_trigger(
        &self,
        g: &Game,
        name: &str,
        src: ObjectId,
        _ctl: PlayerId,
        ev: &Event,
    ) -> Option<Vec<EventInfo>> {
        if name != UPKEEP {
            return None;
        }
        let Event::StepBegan {
            step: crate::turn::Step::Upkeep,
            active,
        } = ev
        else {
            return Some(vec![]);
        };
        Some(
            rows_of(g, src)
                .into_iter()
                .filter(|r| r[2] == i64::from(active.0) && exiled(g, ObjectId(r[1] as u32)))
                .map(|r| EventInfo {
                    object: Some(ObjectId(r[1] as u32)),
                    player: Some(*active),
                    ..Default::default()
                })
                .collect(),
        )
    }
}

/// The card is put onto the stack by `p` as a copy of the original spell `orig` (its last
/// known information on the stack): the same choices, not cast.
fn put_back_on_stack(g: &mut Game, card: ObjectId, orig: ObjectId, p: PlayerId) {
    let Some(mut info) = g.obj(orig).stack.clone() else {
        return;
    };
    info.cast.was_cast = false;
    if let Some(list) = g.zone_list_mut(Zone::Exile) {
        list.retain(|x| *x != card);
    }
    let face = g.obj(orig).face;
    let id = g.create_incarnation(card, Zone::Stack);
    {
        let o = &mut g.objects[id.0 as usize];
        o.zone = Zone::Stack;
        o.controller = p;
        o.base_controller = p;
        o.face = face;
        if let Some(d) = o.card.clone() {
            o.base = d.characteristics(o.face);
        }
        o.stack = Some(info);
    }
    g.stack.push(id);
    g.emit(Event::ZoneChange {
        old: card,
        new: id,
        from: Zone::Exile,
        to: Zone::Stack,
        cause: MoveCause::Effect,
        by: Some(p),
        lookback: None,
    });
    g.dirty = true;
}

inventory::submit! { KeywordRegistration(&Rules) }
