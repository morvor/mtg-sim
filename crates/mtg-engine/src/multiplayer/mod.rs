//! Multiplayer rules (CR 800–811): players leaving the game (CR 800.4), the limited range
//! of influence option ([`range`], CR 801), the attack options ([`attack`], CR 802, 803),
//! the deploy creatures option ([`deploy`], CR 804), setting up the variants and seating
//! the players ([`setup`], CR 806–811) and Grand Melee's turn markers ([`grand_melee`],
//! CR 807.4, 807.5). The shared team turns option (CR 805) is in [`crate::teams`].

pub mod attack;
pub mod deploy;
pub mod grand_melee;
pub mod range;
pub mod setup;
pub mod two_headed;

use crate::decision::Decision;
use crate::events::Event;
use crate::game::Game;
use crate::object::{ObjKind, Zone};
use crate::types::*;
use serde::{Deserialize, Serialize};

/// Multiplayer bookkeeping, in `Game::multiplayer`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MultiplayerState {
    /// Who sits in each seat: the participant index for each seat, when the game was
    /// created by [`setup::new_seated`] (CR 800.5). Empty otherwise.
    pub seats: Vec<usize>,
    /// Ranges of influence as of the beginning of the turn (CR 801.2c).
    pub range: range::RangeState,
    /// Grand Melee's turn markers and stacks (CR 807.4, 807.5).
    pub grand_melee: Option<grand_melee::GrandMelee>,
    /// Last known information about players who have left the game (CR 800.4i).
    pub departed: std::collections::BTreeMap<PlayerId, DepartedPlayer>,
}

/// What a player's zones held just before they left the game (CR 800.4i).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DepartedPlayer {
    pub hand: usize,
    pub library: usize,
    pub graveyard: usize,
}

/// The number of cards in a player's hand, library or graveyard: for a player who has
/// left the game, as they last were before they left (CR 800.4i).
pub fn zone_size(g: &Game, p: PlayerId, zone: crate::ability::ZoneKind) -> usize {
    use crate::ability::ZoneKind;
    if let Some(d) = g.multiplayer.departed.get(&p) {
        return match zone {
            ZoneKind::Hand => d.hand,
            ZoneKind::Library => d.library,
            ZoneKind::Graveyard => d.graveyard,
            _ => 0,
        };
    }
    let pl = g.player(p);
    match zone {
        ZoneKind::Hand => pl.hand.len(),
        ZoneKind::Library => pl.library.len(),
        ZoneKind::Graveyard => pl.graveyard.len(),
        _ => 0,
    }
}

/// Called as a (non-extra or extra) turn of `active` begins, after the turn of
/// `previous`: players within each player's range of influence are determined
/// (CR 801.2c), and for players who left the game seated between the two, the turns
/// that would have begun end their "until your next turn" effects (CR 800.4m).
pub fn turn_began(g: &mut Game, previous: Option<PlayerId>, active: PlayerId, extra: bool) {
    if let Some(prev) = previous.filter(|_| !extra) {
        for q in left_players_seated_between(g, prev, active) {
            g.expire_until_next_turn(q);
        }
    }
    range::determine(g);
}

/// Players who have left the game seated after `after` and before `next` in turn order:
/// their turns would have begun in between (CR 800.4k, 800.4m).
pub fn left_players_seated_between(g: &Game, after: PlayerId, next: PlayerId) -> Vec<PlayerId> {
    let n = g.players.len();
    let mut out = Vec::new();
    if n == 0 {
        return out;
    }
    let mut i = (after.idx() + 1) % n;
    while i != next.idx() && i != after.idx() {
        let q = PlayerId(i as u8);
        if !g.player(q).in_game() {
            out.push(q);
        }
        i = (i + 1) % n;
    }
    out
}

/// CR 800.4a: when a player leaves the game, all objects owned by that player leave the
/// game and any effects which give that player control of any objects or players end.
/// Then objects that player controlled on the stack that aren't represented by cards
/// cease to exist, and any other objects they still control are exiled. This isn't a
/// state-based action; it happens as soon as the player leaves. Objects they own in the
/// ante zone stay (CR 800.4n). Other effects created by that player's spells and
/// abilities continue to apply (CR 800.4m).
pub fn remove_player_objects(g: &mut Game, p: PlayerId) {
    // CR 800.4i: remember what the player's zones held as they left.
    let pl = g.player(p);
    let info = DepartedPlayer {
        hand: pl.hand.len(),
        library: pl.library.len(),
        graveyard: pl.graveyard.len(),
    };
    g.multiplayer.departed.insert(p, info);
    // Objects owned by the player leave the game — except in the ante zone (CR 800.4n).
    let owned: Vec<ObjectId> = g
        .objects
        .iter()
        .filter(|o| {
            o.owner == p && o.next.is_none() && !matches!(o.zone, Zone::Nowhere | Zone::Ante)
        })
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
            g.emit(Event::ZoneChange {
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
    // Effects that give the player control of objects end (CR 800.4a).
    let gives_control = |mods: &[crate::ability::Modification]| {
        mods.iter().any(|m| {
            matches!(m, crate::ability::Modification::SetController(
                crate::ability::PlayerRef::Player(q)) if *q == p)
        })
    };
    g.effects.retain(|e| !gives_control(&e.mods));
    // ... and effects that give them control of players (CR 800.4a, 800.4b).
    g.player_control.active.retain(|e| e.controller != p);
    g.player_control.pending.retain(|e| e.controller != p);
    // Triggered abilities they would control are never put on the stack (CR 800.4d).
    g.delayed_triggers.retain(|d| d.controller != p);
    g.pending_triggers.retain(|t| t.controller != p);
    // With those control effects gone, control reverts to each object's default
    // controller (CR 110.2b) before checking what the player still controls.
    g.recompute();
    // Spells and abilities the player controls on the stack that aren't represented by
    // cards cease to exist (copies of spells, activated and triggered abilities).
    let stack: Vec<ObjectId> = g
        .stack
        .iter()
        .copied()
        .filter(|s| g.obj(*s).controller == p && g.obj(*s).kind != ObjKind::Card)
        .collect();
    for s in stack {
        g.stack.retain(|x| *x != s);
        g.objects[s.0 as usize].zone = Zone::Nowhere;
    }
    // Any other objects the player still controls are exiled (CR 800.4a): permanents, and
    // spells represented by cards they don't own.
    let controlled: Vec<ObjectId> = g
        .battlefield
        .iter()
        .chain(g.stack.iter())
        .copied()
        .filter(|id| g.obj(*id).controller == p)
        .collect();
    for id in controlled {
        g.exile_object(id, None);
    }
    // CR 725.4, 726.4: the monarch or the player with the initiative leaving passes the
    // designation to the active player (or the next player in turn order).
    crate::monarch_initiative::player_left(g, p);
    if let Some(c) = g.combat.as_mut() {
        c.defending_players.retain(|x| *x != p);
    }
    grand_melee::player_left(g, p);
    g.dirty = true;
}

/// CR 800.4c: if an effect giving a player still in the game control of an object ends,
/// no other effect gives control of it to another player in the game, and the player who
/// controls it by default has left the game, the object is exiled. Not a state-based
/// action: it happens as soon as the control-changing effect ends (checked whenever the
/// game settles). Returns true if anything was exiled.
pub fn exile_orphaned_objects(g: &mut Game) -> bool {
    if g.players.iter().all(|p| p.in_game()) || g.players_in_game().len() <= 1 {
        return false;
    }
    if g.dirty {
        g.recompute();
    }
    let orphans: Vec<ObjectId> = g
        .battlefield
        .iter()
        .copied()
        .filter(|id| !g.player(g.obj(*id).controller).in_game())
        .collect();
    for id in &orphans {
        g.exile_object(*id, None);
    }
    !orphans.is_empty()
}

/// CR 800.4b: an object that would be put onto the battlefield or onto the stack under
/// the control of a player who has left the game remains in its current zone (and a
/// token that would be created under such a player's control isn't created, CR 800.4b,
/// 800.4d).
pub fn stays_in_zone(g: &Game, m: &crate::replacement::MoveEv) -> bool {
    if !matches!(m.to, Zone::Battlefield | Zone::Stack) {
        return false;
    }
    let o = g.obj(m.obj);
    let controller = m
        .etb
        .controller
        .or(if o.zone == Zone::Stack {
            Some(o.controller)
        } else {
            None
        })
        .or(m.by)
        .unwrap_or(o.owner);
    !g.player(controller).in_game() || !g.player(o.owner).in_game()
}

/// Whether a player who has left the game can't pay a cost: a cost that an object
/// requires a player who has left the game to pay (or choose whether to pay) isn't paid
/// (CR 800.4f).
pub fn cant_pay(g: &Game, p: PlayerId) -> bool {
    !g.player(p).in_game() && g.players.len() > 2
}

/// The source object of a decision, if it's asked on behalf of one.
fn decision_source(d: &Decision) -> Option<ObjectId> {
    match d {
        Decision::ChooseModes { source, .. }
        | Decision::ChooseX { source, .. }
        | Decision::OptionalCost { source, .. }
        | Decision::Divide { source, .. }
        | Decision::ChooseTargets { source, .. } => Some(*source),
        Decision::YesNo { source, .. }
        | Decision::ChooseEntities { source, .. }
        | Decision::ChooseOption { source, .. }
        | Decision::ChooseNumber { source, .. }
        | Decision::NameCard { source, .. } => *source,
        _ => None,
    }
}

/// Who makes a choice that a player who has left the game would make (CR 800.4g,
/// 800.4h). If an object requires it, the object's controller chooses another player to
/// make it — another opponent if the original chooser was their opponent and there is
/// one (CR 800.4g). If a rule requires it, the next player in turn order makes it
/// (CR 800.4h). Returns `p` itself if they're still in the game.
pub fn substitute_chooser(g: &mut Game, p: PlayerId, d: &Decision) -> PlayerId {
    if g.player(p).in_game() || g.result.is_some() || g.players_in_game().is_empty() {
        return p;
    }
    let Some(src) = decision_source(d).filter(|s| g.try_obj(*s).is_some()) else {
        // CR 800.4h: the next player in turn order who's still in the game.
        return g.next_player(p);
    };
    let controller = g.obj(src).controller;
    if !g.player(controller).in_game() {
        return g.next_player(p);
    }
    let others: Vec<PlayerId> = g
        .players_in_game()
        .into_iter()
        .filter(|q| *q != p)
        .collect();
    let opponents: Vec<PlayerId> = others
        .iter()
        .copied()
        .filter(|q| g.are_opponents(controller, *q))
        .collect();
    let cands = if g.are_opponents(controller, p) && !opponents.is_empty() {
        opponents
    } else {
        others
    };
    if cands.len() <= 1 {
        return cands.first().copied().unwrap_or(controller);
    }
    g.ask_entities(
        controller,
        Some(src),
        "Choose the player who makes the choice in place of a player who left the game",
        cands.iter().map(|q| Entity::Player(*q)).collect(),
        1,
        1,
    )
    .first()
    .and_then(|e| e.player())
    .unwrap_or(cands[0])
}
