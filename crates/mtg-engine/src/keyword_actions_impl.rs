//! Implementations of individual keyword actions (CR 701). Extend `perform` with new
//! actions; each should cite its rule.

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::object::*;
use crate::types::*;

/// CR 701.34a: choose any number of permanents and/or players, then give each another
/// counter of each kind already there.
pub fn proliferate(g: &mut Game, p: PlayerId, ctx: &Ctx) {
    let mut cands: Vec<Entity> = Vec::new();
    for o in g.permanents() {
        if o.counters.values().any(|n| *n > 0) {
            cands.push(Entity::Object(o.id));
        }
    }
    for pl in g.players_in_game() {
        if g.player(pl).counters.values().any(|n| *n > 0) {
            cands.push(Entity::Player(pl));
        }
    }
    let n = cands.len() as u32;
    // Default: proliferate everything this player controls / themselves, and opponents' poison.
    let chosen = g.ask_entities(
        p,
        ctx.source,
        "Proliferate: choose permanents and players",
        cands.clone(),
        0,
        n,
    );
    for e in chosen {
        let kinds: Vec<CounterKind> = match e {
            Entity::Object(o) => g
                .obj(o)
                .counters
                .iter()
                .filter(|(_, n)| **n > 0)
                .map(|(k, _)| k.clone())
                .collect(),
            Entity::Player(pl) => g
                .player(pl)
                .counters
                .iter()
                .filter(|(_, n)| **n > 0)
                .map(|(k, _)| k.clone())
                .collect(),
        };
        for k in kinds {
            g.add_counters(e, &k, 1, ctx.source);
        }
    }
    g.emit(crate::events::Event::Custom {
        name: "proliferate".into(),
        player: Some(p),
        obj: None,
        amount: 0,
    });
}

pub fn perform(
    g: &mut Game,
    action: KeywordAction,
    players: &[PlayerId],
    objs: &[ObjectId],
    n: u32,
    ctx: &mut Ctx,
) {
    let _ = (g, action, players, objs, n, ctx);
    let _ = Zone::Battlefield;
}
