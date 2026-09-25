//! Sagas (CR 714). Chapter abilities are triggered abilities with the trigger
//! `TriggerCond::Custom("chapter:N")`, which trigger when lore counters are put on the
//! Saga bringing the count from below N to at least N (CR 714.2c).

use crate::ability::*;
use crate::object::GameObject;

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

/// The final chapter number (CR 714.2b).
pub fn final_chapter(o: &GameObject) -> Option<u32> {
    chapter_numbers(o).into_iter().max()
}
