//! "different names among unlocked doors of Rooms you control" (Central Elevator //
//! Promising Stairs, CR 709.5): each unlocked door (half) of a Room the controller
//! controls has its own name; the value counts the distinct ones (CR 201.2: a door's name
//! is the name of that half).

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use std::collections::BTreeSet;

/// `Value::Custom`: the number of different names among the unlocked doors of Rooms the
/// controller controls.
pub const DIFFERENT_DOOR_NAMES: &str = "rooms:different names among unlocked doors you control";

pub struct RoomDoorNames;

impl KeywordRules for RoomDoorNames {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name != DIFFERENT_DOOR_NAMES {
            return None;
        }
        let mut names: BTreeSet<String> = BTreeSet::new();
        for &o in &g.battlefield {
            if g.obj(o).controller != ctx.controller || !crate::rooms::is_room(g, o) {
                continue;
            }
            let Some(card) = crate::rooms::room_card(g, o) else {
                continue;
            };
            let u = crate::rooms::unlocked(g, o);
            for (i, face) in card.faces.iter().enumerate().take(2) {
                if u[i] {
                    names.insert(face.chars.name.to_string());
                }
            }
        }
        Some(names.len() as i64)
    }
}

inventory::submit! { KeywordRegistration(&RoomDoorNames) }
