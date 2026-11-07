//! Dimensional Breach: "Exile all permanents. For as long as any of those cards remain
//! exiled, at the beginning of each player's upkeep, that player returns one of the exiled
//! cards they own to the battlefield." Each player chooses which of their cards returns; a
//! card that can't legally be returned can't be chosen (rulings).

use super::{spell, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::{Event, MoveCause};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::{EventInfo, Zone};
use crate::replacement::{EtbInfo, MoveEv};
use crate::turn::Step;
use crate::types::{ObjectId, PlayerId};

const EXILE_ALL: &str = "card:Dimensional Breach:exile all permanents and remember them";
const UPKEEP: &str = "card:Dimensional Breach:an upkeep while any of them remain exiled";
const RETURN_ONE: &str = "card:Dimensional Breach:that player returns one of their cards";
const TEXT: &str = "Exile all permanents. For as long as any of those cards remain exiled, at the beginning of each player's upkeep, that player returns one of the exiled cards they own to the battlefield.";

inventory::submit! { ManualAbility {
    card: "Dimensional Breach",
    face: 0,
    text: TEXT,
    build: |_| {
        vec![spell(
            vec![],
            Effect::Custom(EXILE_ALL.into()),
            TEXT,
        )]
    },
    reason: "each player returns one of the exiled cards each upkeep for as long as any remain: only this card",
} }

struct Rules;

/// The cards (still the same objects in exile) the Breach `src` exiled.
fn remaining(g: &Game, src: ObjectId) -> Vec<ObjectId> {
    g.cards
        .get(EXILE_ALL)
        .iter()
        .filter(|r| r[0] == i64::from(src.0))
        .map(|r| ObjectId(r[1] as u32))
        .filter(|c| g.is_live(*c) && g.obj(*c).zone == Zone::Exile)
        .collect()
}

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some(src) = ctx.source else {
            return name == EXILE_ALL || name == RETURN_ONE;
        };
        if name == EXILE_ALL {
            let perms: Vec<ObjectId> = g.permanents().map(|o| o.id).collect();
            let moves = perms
                .into_iter()
                .map(|o| MoveEv {
                    obj: o,
                    to: Zone::Exile,
                    pos: LibraryPosition::Top,
                    cause: MoveCause::Exile,
                    by: Some(ctx.controller),
                    etb: EtbInfo::default(),
                    source: Some(src),
                })
                .collect();
            let exiled: Vec<ObjectId> = g.move_objects(moves).into_iter().flatten().collect();
            for c in exiled {
                if g.obj(c).zone == Zone::Exile {
                    g.cards.push(EXILE_ALL, vec![i64::from(src.0), i64::from(c.0)]);
                }
            }
            super::delayed_for_rest_of_game(
                g,
                ctx,
                TriggerCond::Custom(UPKEEP.into()),
                Body::effect(Effect::Custom(RETURN_ONE.into())),
            );
            return true;
        }
        if name != RETURN_ONE {
            return false;
        }
        let Some(p) = ctx.event.as_ref().and_then(|e| e.player) else {
            return true;
        };
        let mine: Vec<ObjectId> = remaining(g, src)
            .into_iter()
            .filter(|c| g.obj(*c).owner == p)
            .collect();
        let ok: Vec<ObjectId> = mine
            .into_iter()
            .filter(|c| crate::attach::could_enter_unspecified(g, *c, p))
            .collect();
        if let Some(c) = g
            .ask_objects(p, Some(src), "Return one of your exiled cards", ok, 1, 1)
            .first()
            .copied()
        {
            g.move_object_ev(MoveEv {
                obj: c,
                to: Zone::Battlefield,
                pos: LibraryPosition::Top,
                cause: MoveCause::Return,
                by: Some(p),
                etb: EtbInfo {
                    controller: Some(p),
                    ..Default::default()
                },
                source: Some(src),
            });
        }
        true
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
        Some(match ev {
            Event::StepBegan {
                step: Step::Upkeep,
                active,
            } if !remaining(g, src).is_empty() => vec![EventInfo {
                player: Some(*active),
                ..Default::default()
            }],
            _ => vec![],
        })
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
