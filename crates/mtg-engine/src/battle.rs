//! Battles (CR 310): protectors and defense.

use crate::game::Game;
use crate::object::Zone;
use crate::types::*;

/// The protector of a battle (stored as the battle's chosen player).
pub fn protector(g: &Game, battle: ObjectId) -> Option<PlayerId> {
    g.obj(battle).choices.player
}

/// SBAs 704.5x and 704.5y. Returns true if any action was performed.
pub fn protector_sba(g: &mut Game, to_graveyard: &mut Vec<ObjectId>) -> bool {
    let mut did = false;
    let battles: Vec<ObjectId> = g
        .permanents()
        .filter(|o| o.is(CardType::Battle))
        .map(|o| o.id)
        .collect();
    for b in battles {
        let o = g.obj(b);
        let controller = o.controller;
        let siege = o.chars.has_subtype("Siege");
        let current = o.choices.player;
        let valid = |g: &Game, p: PlayerId| -> bool {
            g.player(p).in_game()
                && if siege {
                    g.are_opponents(controller, p)
                } else {
                    true
                }
        };
        let needs = match current {
            None => true,
            Some(p) => !valid(g, p),
        };
        if !needs {
            continue;
        }
        let cands: Vec<PlayerId> = if siege {
            g.opponents(controller)
        } else {
            vec![controller]
        };
        if cands.is_empty() {
            to_graveyard.push(b);
            continue;
        }
        let ents = cands.iter().map(|p| Entity::Player(*p)).collect();
        let chosen = g.ask_entities(
            controller,
            Some(b),
            "Choose the battle's protector",
            ents,
            1,
            1,
        );
        if let Some(p) = chosen.first().and_then(|e| e.player()) {
            g.objects[b.0 as usize].choices.player = Some(p);
            did = true;
        }
        let _ = Zone::Battlefield;
    }
    did
}
