//! Restarting the game (CR 104.6, 727): the game immediately ends — nobody wins, loses or
//! draws — and all players still in it start a new game following the procedures of
//! CR 103, with the restarting effect's controller as the starting player (CR 727.1a).
//! Every card involved in the old game is involved in the new one (CR 727.2), except
//! cards the effect leaves in exile (CR 727.5); the rest of the effect happens just before
//! the first turn's untap step (CR 727.4).

use crate::ability::*;
use crate::card::CardDef;
use crate::eval::Ctx;
use crate::game::{Game, GameConfig};
use crate::object::{ObjKind, Zone};
use crate::types::*;
use smol_str::SmolStr;
use std::sync::Arc;

/// A pending restart of the game.
#[derive(Clone, Debug)]
pub struct RestartRequest {
    /// The controller of the spell or ability that restarted the game (CR 727.1a).
    pub controller: PlayerId,
    /// Cards left in exile rather than put into their owners' decks (CR 727.5).
    pub keep: Vec<ObjectId>,
    /// The rest of the effect (CR 727.4).
    pub then: Vec<Effect>,
}

/// Executes [`Effect::RestartGame`]: records the request. The game restarts once the
/// resolving spell or ability has finished resolving in the old game.
pub fn request_restart(g: &mut Game, keep: Option<&Sel>, ctx: &mut Ctx) {
    let keep: Vec<ObjectId> = keep
        .map(|s| {
            g.resolve_sel(s, ctx)
                .into_iter()
                .filter_map(|e| e.object())
                .filter(|o| g.is_live(*o) && g.obj(*o).zone == Zone::Exile)
                .collect()
        })
        .unwrap_or_default();
    g.log(|_| "the game restarts".to_string());
    g.end.restart = Some(RestartRequest {
        controller: ctx.controller,
        keep,
        then: vec![],
    });
}

/// Performs a requested restart, if any.
pub fn restart_if_requested(g: &mut Game) {
    let Some(req) = g.end.restart.take() else {
        return;
    };
    restart_game(g, req);
}

fn restart_game(g: &mut Game, req: RestartRequest) {
    let n = g.players.len();
    let in_game: Vec<bool> = g.players.iter().map(|p| p.in_game()).collect();
    // CR 727.2: every card involved in the game, wherever it is (phased-out permanents,
    // nontraditional cards, cards on the stack), goes into its owner's new deck; ownership
    // doesn't change. Tokens, copies and emblems don't survive. Cards outside the game
    // (sideboards) stay there.
    let mut decks: Vec<Vec<Arc<CardDef>>> = vec![Vec::new(); n];
    let mut kept: Vec<(PlayerId, Arc<CardDef>, bool)> = Vec::new();
    let mut sideboards: Vec<Vec<(ObjectId, Arc<CardDef>)>> = vec![Vec::new(); n];
    for (i, o) in g.objects.iter().enumerate() {
        let id = ObjectId(i as u32);
        if o.next.is_some() || o.zone == Zone::Nowhere || o.kind != ObjKind::Card {
            continue;
        }
        let Some(card) = o.card.clone() else {
            continue;
        };
        let owner = o.owner;
        if !in_game[owner.idx()] {
            continue;
        }
        if matches!(o.zone, Zone::Outside(_)) {
            sideboards[owner.idx()].push((id, card));
        } else if req.keep.contains(&id) {
            kept.push((owner, card, o.is_commander));
        } else {
            decks[owner.idx()].push(card);
        }
    }
    let commanders: Vec<Vec<SmolStr>> = g
        .players
        .iter()
        .map(|p| p.commander_names.clone())
        .collect();
    let config = GameConfig {
        starting_player: Some(req.controller),
        first_turn_chooser: None,
        ..g.config.clone()
    };
    let mut new = Game::new(config, decks, vec![]);
    new.agents = g.agents.clone();
    new.rng = g.rng.clone();
    new.logging = g.logging;
    new.log = std::mem::take(&mut g.log);
    new.start.sticker_sheets = std::mem::take(&mut g.start.sticker_sheets);
    for (i, p) in g.players.iter().enumerate() {
        new.players[i].name = p.name.clone();
        new.players[i].team = p.team;
        if !in_game[i] {
            // Only the players still in the game play the new game.
            new.players[i].left_game = true;
            new.players[i].has_lost = p.has_lost;
        }
    }
    // Each card outside the old game: (old object, new object).
    let mut outside: Vec<(ObjectId, ObjectId)> = Vec::new();
    for (i, side) in sideboards.into_iter().enumerate() {
        let (olds, cards): (Vec<ObjectId>, Vec<Arc<CardDef>>) = side.into_iter().unzip();
        let news = new.add_to_sideboard(PlayerId(i as u8), cards);
        outside.extend(olds.into_iter().zip(news));
    }
    // CR 727.6: a restarted subgame is still the subgame.
    crate::subgame::restarted(g, &mut new, &outside);
    for (i, names) in commanders.into_iter().enumerate() {
        for name in names {
            // CR 727.5a: an exempted commander remains that deck's commander.
            if !new.designate_commander(PlayerId(i as u8), &name) {
                new.players[i].commander_names.push(name);
            }
        }
    }
    // CR 727.5: exempted cards begin the new game in exile.
    let mut kept_ids: Vec<Entity> = Vec::new();
    for (owner, card, is_commander) in kept {
        let id = new.create_card_object(card, owner, Zone::Exile);
        new.objects[id.0 as usize].is_commander = is_commander;
        new.exile.push(id);
        kept_ids.push(Entity::Object(id));
    }
    new.log(|_| "--- The game restarts ---".to_string());
    // CR 727.1: the new game starts following CR 103.
    let starting = new.pregame();
    // CR 727.4: the rest of the effect finishes resolving after the pregame procedure,
    // just before the first turn; no player has priority, and abilities that trigger wait
    // until a player would receive priority.
    let mut ctx = Ctx::new(None, req.controller);
    ctx.set_var(vars::IT, kept_ids);
    for e in &req.then {
        new.exec(e, &mut ctx);
    }
    new.flush_events();
    new.begin_turn(starting, false);
    *g = new;
}
