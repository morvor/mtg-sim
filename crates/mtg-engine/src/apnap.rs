//! Choices several players make at the same time (CR 101.4, "APNAP order"): the active
//! player chooses first, then each nonactive player in turn order, then the actions happen
//! simultaneously. Each player knows the choices made before theirs (CR 101.4b), except
//! that cards chosen in a hidden zone stay face down (CR 101.4a) — agents can read the
//! choices made so far from [`Game::known_apnap_choices`] while deciding.

use crate::ability::LibraryPosition;
use crate::events::{Event, MoveCause};
use crate::game::Game;
use crate::object::Zone;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::*;
use serde::{Deserialize, Serialize};

/// A choice a player made as part of simultaneous choices (CR 101.4).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApnapChoice {
    pub player: PlayerId,
    /// What was chosen.
    pub chosen: Vec<ObjectId>,
    /// Chosen in a hidden zone (a hand or library): the cards stay face down as they're
    /// chosen (CR 101.4a).
    pub hidden: bool,
}

impl Game {
    /// Makes simultaneous choices in APNAP order (CR 101.4): each request is one player's
    /// choice; `choose` makes it and returns any further choices it causes. The next
    /// choice is always the outstanding one of the player earliest in APNAP order, so if a
    /// nonactive player's choice makes the active player (or a nonactive player earlier in
    /// turn order) have to choose, APNAP order restarts for all outstanding choices
    /// (CR 101.4d). Requests of the same player keep their order.
    pub fn apnap_round<T>(
        &mut self,
        requests: Vec<(PlayerId, T)>,
        mut choose: impl FnMut(&mut Game, PlayerId, T) -> Vec<(PlayerId, T)>,
    ) {
        let order = self.pregame_order_or_apnap();
        let pos = |p: PlayerId| order.iter().position(|x| *x == p).unwrap_or(usize::MAX);
        let mut outstanding: Vec<(PlayerId, T)> = requests;
        while !outstanding.is_empty() {
            let i = (0..outstanding.len())
                .min_by_key(|i| (pos(outstanding[*i].0), *i))
                .unwrap_or(0);
            let (p, data) = outstanding.remove(i);
            let more = choose(self, p, data);
            outstanding.extend(more);
        }
    }

    /// APNAP order for simultaneous choices: with shared team turns the active team's
    /// players first, then each other team in turn order (CR 805.6).
    fn pregame_order_or_apnap(&self) -> Vec<PlayerId> {
        let mut order = self.pregame_order();
        // Players who have left the game make no choices, but keep them last so a request
        // for them is still handled.
        for p in self.player_ids() {
            if !order.contains(&p) {
                order.push(p);
            }
        }
        order
    }

    /// Records a player's choice among simultaneous choices, for players choosing later
    /// (CR 101.4b). A round of choices starts at `apnap_choices.len()`.
    pub fn record_apnap_choice(&mut self, player: PlayerId, chosen: Vec<ObjectId>) {
        let hidden = chosen
            .iter()
            .any(|o| matches!(self.obj(*o).zone, Zone::Hand(_) | Zone::Library(_)));
        self.apnap_choices.push(ApnapChoice {
            player,
            chosen,
            hidden,
        });
    }

    /// Ends a round of simultaneous choices once the actions have happened.
    pub fn end_apnap_choices(&mut self, from: usize) {
        self.apnap_choices.truncate(from);
    }

    /// The choices made so far in the current round of simultaneous choices, as `viewer`
    /// knows them: public choices are known (CR 101.4b); a card another player chose in a
    /// hidden zone remains face down, so only the fact that they chose is known
    /// (CR 101.4a).
    pub fn known_apnap_choices(&self, viewer: PlayerId) -> Vec<(PlayerId, Option<Vec<ObjectId>>)> {
        self.apnap_choices
            .iter()
            .map(|c| {
                let visible = !c.hidden || c.player == viewer;
                (c.player, visible.then(|| c.chosen.clone()))
            })
            .collect()
    }

    /// Sacrifices several permanents at the same time (CR 101.4, 701.21a): they leave the
    /// battlefield in one simultaneous event, so leaves-the-battlefield abilities of each
    /// see the others (CR 603.10a). Returns (sacrificed object, new object) pairs.
    pub fn sacrifice_simultaneously(
        &mut self,
        what: &[(ObjectId, PlayerId)],
    ) -> Vec<(ObjectId, ObjectId)> {
        if self.dirty {
            self.recompute();
        }
        let ok: Vec<(ObjectId, PlayerId)> = what
            .iter()
            .copied()
            .filter(|(o, by)| {
                self.is_live(*o)
                    && self.obj(*o).zone == Zone::Battlefield
                    && self.obj(*o).controller == *by
                    && !self.cant_be_sacrificed(*o)
            })
            .collect();
        let moves: Vec<MoveEv> = ok
            .iter()
            .map(|(o, by)| MoveEv {
                obj: *o,
                to: Zone::Graveyard(self.obj(*o).owner),
                pos: LibraryPosition::Top,
                cause: MoveCause::Sacrifice,
                by: Some(*by),
                etb: EtbInfo::default(),
                source: None,
            })
            .collect();
        let res = self.move_objects(moves);
        for (o, by) in &ok {
            self.history.sacrificed.push((*by, *o));
            self.emit(Event::Sacrificed {
                obj: *o,
                player: *by,
            });
        }
        ok.iter()
            .zip(res)
            .filter_map(|((o, _), new)| new.map(|n| (*o, n)))
            .collect()
    }
}

/// Executes [`crate::ability::Effect::KeepAndSacrificeRest`] (CR 101.4, 101.4c).
pub fn keep_and_sacrifice_rest(
    g: &mut Game,
    who: &crate::ability::PlayerRef,
    among: &crate::ability::Filter,
    keep: &[crate::ability::Filter],
    ctx: &mut crate::eval::Ctx,
) {
    let players = g.eval_players(who, ctx);
    let round = g.apnap_choices.len();
    let mut sacrifice: Vec<(ObjectId, PlayerId)> = Vec::new();
    let requests = players.into_iter().map(|p| (p, ())).collect();
    let rctx: &crate::eval::Ctx = ctx;
    g.apnap_round(requests, |g, p, ()| {
        let ctx = rctx;
        let mut pctx = ctx.clone();
        pctx.iter_player = Some(p);
        let mine: Vec<ObjectId> = g
            .battlefield
            .clone()
            .into_iter()
            .filter(|o| g.obj(*o).controller == p && g.matches(*o, among, &pctx))
            .collect();
        // CR 101.4c: the choices are made in the order specified.
        let mut kept: Vec<ObjectId> = Vec::new();
        for f in keep {
            let cands: Vec<ObjectId> = mine
                .iter()
                .copied()
                .filter(|o| !kept.contains(o) && g.matches(*o, f, &pctx))
                .collect();
            let pick = g.ask_objects(p, ctx.source, "Choose a permanent to keep", cands, 1, 1);
            kept.extend(pick);
        }
        g.record_apnap_choice(p, kept.clone());
        sacrifice.extend(
            mine.into_iter()
                .filter(|o| !kept.contains(o))
                .map(|o| (o, p)),
        );
        vec![]
    });
    let res = g.sacrifice_simultaneously(&sacrifice);
    g.end_apnap_choices(round);
    ctx.prev_value = res.len() as i64;
    ctx.prev_happened = !res.is_empty();
}
