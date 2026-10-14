//! Sagas (CR 714). Chapter abilities are triggered abilities with the trigger
//! `TriggerCond::Custom("chapter:N")`, which trigger when lore counters are put on the
//! Saga bringing the count from below N to at least N (CR 714.2b, 714.2c).
//!
//! * A Saga enters with a lore counter (CR 714.3a); one with read ahead enters with the
//!   number of lore counters its controller chooses, and its chapter abilities trigger
//!   the turn it entered only for exactly that many counters (CR 714.3b, 702.155).
//! * Its controller puts a lore counter on it as their precombat main phase begins
//!   (CR 714.3c, `turn.rs`), and sacrifices it once it has as many lore counters as its
//!   final chapter number and no chapter ability of it is on the stack (CR 714.4,
//!   `sba.rs`).

use crate::ability::*;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::GameObject;
use crate::types::*;

pub fn chapter_numbers(o: &GameObject) -> Vec<u32> {
    let mut v = Vec::new();
    for a in &o.chars.abilities {
        if let AbilityKind::Triggered(t) = &a.kind {
            if let TriggerCond::Custom(name) = &t.trigger {
                if let Some(n) = name.strip_prefix("chapter:") {
                    for part in n.split(',') {
                        if let Ok(k) = part.trim().parse::<u32>() {
                            v.push(k);
                        }
                    }
                }
            }
        }
    }
    v
}

pub fn has_chapters(o: &GameObject) -> bool {
    !chapter_numbers(o).is_empty()
}

/// The final chapter number: the greatest value among its chapter abilities, `None` (0)
/// if it somehow has none (CR 714.2d).
pub fn final_chapter(o: &GameObject) -> Option<u32> {
    chapter_numbers(o).into_iter().max()
}

/// Whether the Saga has read ahead (CR 702.155).
pub fn has_read_ahead(o: &GameObject) -> bool {
    o.chars.has_keyword(KeywordKind::ReadAhead)
}

/// The lore counters the permanent `id` enters with: one for a Saga without read ahead
/// (CR 714.3a); for one with read ahead, the number between one and its final chapter
/// number its controller chooses as it enters (CR 714.3b, 702.155b). Zero for anything
/// else.
pub fn entering_lore_counters(g: &mut Game, id: ObjectId) -> u32 {
    let o = g.obj(id);
    if !o.chars.has_subtype("Saga") || o.face_down {
        return 0;
    }
    if !has_read_ahead(o) {
        return 1;
    }
    let max = final_chapter(o).unwrap_or(0).max(1);
    let ctl = o.controller;
    g.ask_number(
        ctl,
        Some(id),
        "Read ahead: choose a number of lore counters",
        1,
        max as i64,
    ) as u32
}

/// CR 702.155a: a chapter ability of a Saga with read ahead can't trigger the turn the
/// Saga entered the battlefield unless it has exactly `lore` = N lore counters.
pub fn chapter_may_trigger(g: &Game, saga: ObjectId, n: u32, lore: u32) -> bool {
    let o = g.obj(saga);
    !(has_read_ahead(o) && o.entered_turn == g.turn.number) || lore == n
}
