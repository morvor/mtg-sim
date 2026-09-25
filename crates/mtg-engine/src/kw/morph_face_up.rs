//! Turning a face-down permanent face up for its morph, megamorph, or disguise cost: a
//! special action (CR 116.2b, 702.37e, 702.168d). If that cost contains {X}, the player
//! chooses X immediately before paying it (CR 107.3d).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::Illegal;
use crate::decision::{Action, Answer, Decision, SpecialAction};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::types::*;

pub struct MorphFaceUp;

/// Whether it has megamorph, and the cost to turn `id` face up, if it's a face-down permanent whose
/// face-up characteristics have morph, megamorph, or disguise.
fn face_up_cost(g: &Game, id: ObjectId) -> Option<(bool, Cost)> {
    let o = g.obj(id);
    if !o.face_down || o.zone != Zone::Battlefield || !g.is_live(id) {
        return None;
    }
    let card = o.card.as_ref()?;
    card.front()
        .chars
        .abilities
        .iter()
        .find_map(|a| match &a.kind {
            AbilityKind::Keyword(k)
                if matches!(k.kind, KeywordKind::Morph | KeywordKind::Disguise) =>
            {
                let megamorph = k
                    .text
                    .as_deref()
                    .is_some_and(|t| t.to_lowercase().starts_with("megamorph"));
                k.cost.clone().map(|c| (megamorph, c))
            }
            _ => None,
        })
}

impl KeywordRules for MorphFaceUp {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    /// CR 702.37e: any time a player has priority, they may turn a face-down permanent
    /// they control face up by paying its morph cost.
    fn special_actions(&self, g: &Game, p: PlayerId) -> Vec<Action> {
        if g.turn.priority != Some(p) {
            return vec![];
        }
        // Only if its cost could be paid (with X = 0).
        g.permanents()
            .filter(|o| o.controller == p)
            .filter(|o| {
                face_up_cost(g, o.id).is_some_and(|(_, mut cost)| {
                    if let Some(m) = cost.mana.clone().filter(|m| m.has_x()) {
                        cost.mana = Some(m.with_x(0));
                    }
                    g.can_pay_cost(p, &cost, Some(o.id), &Ctx::new(Some(o.id), p))
                })
            })
            .map(|o| Action::Special(SpecialAction::TurnFaceUp { obj: o.id }))
            .collect()
    }

    fn perform_special_action(
        &self,
        g: &mut Game,
        p: PlayerId,
        sa: &SpecialAction,
    ) -> Option<Result<(), Illegal>> {
        let SpecialAction::TurnFaceUp { obj } = sa else {
            return None;
        };
        let obj = *obj;
        let Some((megamorph, mut cost)) = face_up_cost(g, obj) else {
            return Some(Err(Illegal("nothing to turn face up".into())));
        };
        if g.obj(obj).controller != p {
            return Some(Err(Illegal("not your permanent".into())));
        }
        // CR 107.3d: X is chosen immediately before the cost is paid.
        let mut chosen_x = None;
        if let Some(m) = cost.mana.clone().filter(|m| m.has_x()) {
            let max = g.max_mana_available(p) as i64;
            let x = match g.ask(p, Decision::ChooseX { source: obj, max }) {
                Answer::Number(n) if (0..=max).contains(&n) => n,
                _ => 0,
            };
            cost.mana = Some(m.with_x(x as u32));
            chosen_x = Some(x as i32);
        }
        let ctx = Ctx::new(Some(obj), p);
        if !g.pay_cost(p, &cost, Some(obj), &ctx) {
            return Some(Err(Illegal("can't pay the cost to turn it face up".into())));
        }
        // CR 702.37f: other abilities of the permanent that refer to X use the value
        // chosen as the special action was taken (see `etb_trigger_cast_info`).
        if let Some(x) = chosen_x {
            g.objects[obj.0 as usize]
                .cast
                .get_or_insert_with(Default::default)
                .x = Some(x);
        }
        crate::facedown::turn_face_up(g, obj, true);
        // CR 702.37b: megamorph puts a +1/+1 counter on it as it's turned face up.
        if megamorph {
            g.add_counters(Entity::Object(obj), counters::PLUS1, 1, Some(obj));
        }
        Some(Ok(()))
    }
}

inventory::submit! { KeywordRegistration(&MorphFaceUp) }
