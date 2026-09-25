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

/// The morph, megamorph, or disguise ability among `abilities`: (kind, is megamorph,
/// cost).
fn face_up_keyword(abilities: &[Ability]) -> Option<(KeywordKind, bool, Cost)> {
    abilities.iter().find_map(|a| match &a.kind {
        AbilityKind::Keyword(k) if matches!(k.kind, KeywordKind::Morph | KeywordKind::Disguise) => {
            let megamorph = k
                .text
                .as_deref()
                .is_some_and(|t| t.to_lowercase().starts_with("megamorph"));
            k.cost.clone().map(|c| (k.kind, megamorph, c))
        }
        _ => None,
    })
}

/// Whether any continuous effect or static ability in the game could change an object's
/// abilities (copy, text-, type-, or ability-changing effects, CR 613.1). Without one, a
/// face-down permanent would have its printed abilities face up.
fn effects_could_change_abilities(g: &Game) -> bool {
    let risky = |m: &Modification| {
        matches!(
            m.layer(),
            Layer::L1aCopy | Layer::L3Text | Layer::L4Type | Layer::L6Ability
        )
    };
    g.effects
        .iter()
        .any(|e| e.layer1.is_some() || e.mods.iter().any(risky))
        || g.live_objects().into_iter().any(|id| {
            g.obj(id).chars.abilities.iter().any(|a| {
                matches!(&a.kind, AbilityKind::Static(s)
                    if matches!(&s.effect, StaticEffect::Continuous { mods, .. } if mods.iter().any(risky)))
            })
        })
}

/// Whether it has megamorph, and the cost to turn `id` face up, if it's a face-down
/// permanent that would have morph, megamorph, or disguise if it were face up (CR 702.37e:
/// if it wouldn't have a morph cost face up, e.g. because of an effect that would apply to
/// it, it can't be turned face up this way).
fn face_up_cost(g: &Game, id: ObjectId) -> Option<(bool, Cost)> {
    let o = g.obj(id);
    if !o.face_down || o.zone != Zone::Battlefield || !g.is_live(id) {
        return None;
    }
    let card = o.card.as_ref()?;
    // Printed without such an ability: nothing to look at.
    let printed = face_up_keyword(&card.front().chars.abilities)?;
    let (kind, megamorph, cost) = if effects_could_change_abilities(g) {
        // The characteristics it would have face up, with the effects that would apply.
        let mut h = g.clone();
        h.objects[id.0 as usize].face_down = false;
        h.recompute();
        face_up_keyword(&h.obj(id).chars.abilities)?
    } else {
        printed
    };
    // "All morph costs cost {2} more" (a megamorph cost is a morph cost, CR 702.37b).
    let cost = if kind == KeywordKind::Morph {
        super::modified_keyword_cost(g, o.controller, KeywordKind::Morph, &cost)
    } else {
        cost
    };
    Some((megamorph, cost))
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
