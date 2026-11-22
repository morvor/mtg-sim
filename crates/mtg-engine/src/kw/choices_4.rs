//! Choices IV (`oracle/patterns/choices_4.rs`): filters reading a choice made by the
//! instruction's own ability.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::ObjectId;

/// `Filter::Custom`: an object whose power is of the quality (odd or even) chosen by the
/// linked ability (CR 208, 607.2d). An object with no power has none of either quality;
/// an undefined choice matches nothing (CR 607.5a).
pub const POWER_OF_CHOSEN_QUALITY: &str = "choices 4: power of the chosen quality";

/// `Effect::Custom`: "collect evidence X" with X chosen as the instruction is performed
/// (Incinerator of the Guilty): the player chooses X (no more than the total mana value of
/// the cards in their graveyard), then collects evidence X (CR 701.59a); X stays the
/// value of X for the rest of the resolution and for the reflexive ability that follows
/// ("When you do, ... deals X damage", CR 603.12). Whether evidence was collected is the
/// instruction's result.
pub const COLLECT_EVIDENCE_X: &str = "choices 4: collect evidence X";

/// `Effect::Custom`: "unlock a locked door of a Room you control" (Ghostly Dancers): the
/// player chooses a Room they control with a locked door and a locked door of it, and
/// unlocks it (CR 709.5f, 709.5j). Done only if there was one.
pub const UNLOCK_A_DOOR: &str = "choices 4: unlock a locked door of a Room you control";

pub struct Choices4;

impl KeywordRules for Choices4 {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name == UNLOCK_A_DOOR {
            unlock_a_door(g, ctx);
            return true;
        }
        if name != COLLECT_EVIDENCE_X {
            return false;
        }
        let p = ctx.controller;
        let total: i64 = g
            .player(p)
            .graveyard
            .clone()
            .iter()
            .map(|c| g.mana_value_of(*c) as i64)
            .sum();
        let x = g
            .ask_number(p, ctx.source, "Choose X (collect evidence X)", 0, total)
            .clamp(0, total);
        let done = crate::kwa::evidence_forage::collect_evidence(g, p, x as u32, ctx.source);
        ctx.x = x as i32;
        ctx.x_defined = true;
        ctx.prev_happened = done;
        true
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != POWER_OF_CHOSEN_QUALITY {
            return None;
        }
        let Some(p) = g.obj(id).chars.power else {
            return Some(false);
        };
        let q = g.linked_choice(ctx).and_then(|ch| ch.text.clone());
        Some(match q.as_deref() {
            Some("odd") => p.rem_euclid(2) == 1,
            Some("even") => p.rem_euclid(2) == 0,
            _ => false,
        })
    }
}

inventory::submit! { KeywordRegistration(&Choices4) }

fn unlock_a_door(g: &mut Game, ctx: &mut Ctx) {
    let p = ctx.controller;
    if g.dirty {
        g.recompute();
    }
    let rooms: Vec<ObjectId> = g
        .permanents()
        .filter(|o| o.controller == p)
        .map(|o| o.id)
        .filter(|id| !crate::rooms::locked_halves(g, *id).is_empty())
        .collect();
    ctx.prev_happened = false;
    if rooms.is_empty() {
        return;
    }
    let room = g
        .ask_objects(
            p,
            ctx.source,
            "Choose a Room to unlock a door of",
            rooms.clone(),
            1,
            1,
        )
        .first()
        .copied()
        .filter(|r| rooms.contains(r))
        .unwrap_or(rooms[0]);
    let locked = crate::rooms::locked_halves(g, room);
    let half = if locked.len() > 1 {
        let names: Vec<String> = crate::rooms::room_card(g, room)
            .map(|c| c.faces.iter().map(|f| f.chars.name.to_string()).collect())
            .unwrap_or_else(|| vec!["Left door".into(), "Right door".into()]);
        let i = g.ask_option(
            p,
            ctx.source,
            "Unlock which door?",
            locked
                .iter()
                .map(|h| names.get(*h).cloned().unwrap_or_default())
                .collect(),
        );
        locked.get(i).copied().unwrap_or(locked[0])
    } else {
        locked[0]
    };
    ctx.prev_happened = crate::rooms::unlock(g, room, half, p);
}
