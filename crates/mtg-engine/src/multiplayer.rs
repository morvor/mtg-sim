//! Multiplayer rules (CR 800–811): players leaving the game.

use crate::game::Game;
use crate::object::Zone;
use crate::types::*;

/// CR 800.4a: when a player leaves the game, all objects owned by that player leave the
/// game, effects giving them control of objects end, and spells/abilities they control
/// on the stack cease to exist.
pub fn remove_player_objects(g: &mut Game, p: PlayerId) {
    // Objects owned by the player leave the game.
    let owned: Vec<ObjectId> = g
        .objects
        .iter()
        .filter(|o| o.owner == p && o.next.is_none() && o.zone != Zone::Nowhere)
        .map(|o| o.id)
        .collect();
    // CR 603.6c: a phased-in permanent leaving the game because its owner left triggers
    // leaves-the-battlefield abilities (which look back in time, CR 603.10a).
    let lookback = std::sync::Arc::new(g.lookback_snapshot());
    for id in owned {
        let zone = g.obj(id).zone;
        if let Some(list) = g.zone_list_mut(zone) {
            list.retain(|x| *x != id);
        }
        g.objects[id.0 as usize].zone = Zone::Nowhere;
        if zone == Zone::Battlefield && !g.obj(id).phased_out {
            g.emit(crate::events::Event::ZoneChange {
                old: id,
                new: id,
                from: Zone::Battlefield,
                to: Zone::Nowhere,
                cause: crate::events::MoveCause::Other,
                by: None,
                lookback: Some(lookback.clone()),
            });
        }
    }
    // Spells and abilities controlled by the player on the stack cease to exist.
    let stack: Vec<ObjectId> = g
        .stack
        .iter()
        .copied()
        .filter(|s| g.obj(*s).controller == p)
        .collect();
    for s in stack {
        g.stack.retain(|x| *x != s);
        g.objects[s.0 as usize].zone = Zone::Nowhere;
    }
    // Effects controlled by the player end (control effects in particular, CR 800.4a).
    g.effects.retain(|e| e.controller != p);
    g.rule_effects.retain(|e| e.controller != p);
    g.player_effects.retain(|e| e.controller != p);
    g.replacements.retain(|e| e.controller != p);
    g.delayed_triggers.retain(|d| d.controller != p);
    g.pending_triggers.retain(|t| t.controller != p);
    // Objects controlled (but not owned) by that player are exiled (CR 800.4a).
    let controlled: Vec<ObjectId> = g
        .battlefield
        .iter()
        .copied()
        .filter(|id| g.obj(*id).controller == p)
        .collect();
    for id in controlled {
        g.exile_object(id, None);
    }
    if g.monarch == Some(p) {
        // CR 725.4: the monarch leaving makes the active player (or next) the monarch.
        let next = if g.turn.active != p {
            g.turn.active
        } else {
            g.next_player(p)
        };
        g.monarch = Some(next);
    }
    if g.initiative == Some(p) {
        let next = if g.turn.active != p {
            g.turn.active
        } else {
            g.next_player(p)
        };
        g.initiative = Some(next);
    }
    if let Some(c) = g.combat.as_mut() {
        c.defending_players.retain(|x| *x != p);
    }
    g.dirty = true;
}
