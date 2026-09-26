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
        // CR 810.10d: in Two-Headed Giant, a player has their team's poison counters.
        if !crate::kwa::proliferate_teams::player_counter_kinds(g, pl).is_empty() {
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
    // CR 701.34b: one additional poison counter per team.
    let no_poison =
        crate::kwa::proliferate_teams::players_without_poison(g, p, &chosen, ctx.source);
    for e in chosen {
        let kinds: Vec<CounterKind> = match e {
            Entity::Object(o) => g
                .obj(o)
                .counters
                .iter()
                .filter(|(_, n)| **n > 0)
                .map(|(k, _)| k.clone())
                .collect(),
            Entity::Player(pl) => crate::kwa::proliferate_teams::player_counter_kinds(g, pl)
                .into_iter()
                .filter(|k| !(no_poison.contains(&pl) && k.as_str() == counters::POISON))
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
    let _ = Zone::Battlefield;
    match action {
        // CR 701.15a: goad creatures until the goading player's next turn (their
        // designation is cleared as that turn begins, CR 701.15b). The same player goading
        // a creature again has no effect (CR 701.15d); several players can goad it
        // (CR 701.15c).
        KeywordAction::Goad => {
            for o in objs {
                if !g.is_live(*o) || g.obj(*o).zone != Zone::Battlefield || !g.obj(*o).is_creature()
                {
                    continue;
                }
                let by = ctx.controller;
                let obj = &mut g.objects[o.0 as usize];
                if !obj.goaded_by.contains(&by) {
                    obj.goaded_by.push(by);
                    g.log(|g| format!("{by} goads {}", g.describe(*o)));
                }
            }
        }
        // CR 701.49: venture into the dungeon.
        KeywordAction::Venture => {
            for p in players {
                crate::dungeons::venture(g, *p, ctx.source);
            }
        }
        // CR 701.32c: set schemes in motion one at a time.
        KeywordAction::SetInMotion => {
            for p in players {
                for _ in 0..n.max(1) {
                    crate::variants::set_in_motion(g, *p);
                }
            }
        }
        // CR 701.33: abandon a scheme.
        KeywordAction::Abandon => {
            for o in objs {
                crate::variants::abandon(g, *o);
            }
        }
        // CR 701.52: roll to visit your Attractions.
        KeywordAction::RollAttractions => {
            for p in players {
                crate::variants::roll_to_visit(g, *p);
            }
        }
        // CR 701.31: planeswalk (only the planar controller can).
        KeywordAction::Planeswalk => {
            for p in players {
                crate::planechase::planeswalk(g, *p);
            }
        }
        _ => {}
    }
}
