//! Subgames (CR 729): Shahrazad's "Players play a Magic subgame, using their libraries as
//! their decks."
//!
//! A subgame is a completely separate [`Game`] (CR 729.1a): the main game is discontinued
//! while it's played and nothing of either game has any meaning in the other (CR 729.1b).
//!
//! * As it starts, each player's main-game library becomes their subgame deck, with their
//!   supplementary decks (planar, scheme, attraction decks), vanguard, and a commander in
//!   the main-game command zone (CR 729.2, 729.2a–c); the starting player is determined at
//!   random. The main-game objects and cards outside the main game are outside the
//!   subgame (CR 729.4): each player's cards there can be brought in (e.g. by a wish).
//! * As it ends, each player's traditional cards in the subgame (other than in its command
//!   zone) are shuffled into their main-game library (CR 729.5), supplementary decks,
//!   vanguards and commanders return to the main-game command zone (CR 729.5a–c), and
//!   everything else ceases to exist. Cards brought in from the main game left their
//!   main-game zones: main-game abilities that trigger on that wait for the main game to
//!   resume (CR 729.4a).
//! * A subgame can be created within a subgame (CR 729.6), and restarted (CR 727.6).

use crate::ability::*;
use crate::card::CardDef;
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::{Game, GameConfig, GameResult};
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::{ObjKind, Zone};
use crate::types::*;
use rand::Rng;
use smol_str::SmolStr;
use std::sync::Arc;

/// `Effect::Custom` name: "Players play a Magic subgame, using their libraries as their
/// decks." The winners are stored in [`WINNERS`].
pub const PLAY_SUBGAME: &str = "play a subgame";
/// `Effect::Custom` name: "Each player who doesn't win the subgame loses half their
/// life, rounded up."
pub const NON_WINNERS_LOSE_HALF: &str = "subgame: non-winners lose half their life";
/// The variable holding the players who won the subgame (none if it was a draw).
pub const WINNERS: Var = vars::USER + 729;

/// Subgame bookkeeping, in `Game::subgames`.
#[derive(Clone, Debug, Default)]
pub struct SubgameState {
    /// How deeply nested this game is: 0 for the main game (CR 729.6).
    pub depth: u32,
    /// Subgames played from this game.
    pub played: u32,
    /// The last subgame played from this game, as it ended.
    pub last: Option<Box<Game>>,
}

/// A subgame in progress.
pub struct Subgame {
    pub game: Game,
    /// Cards outside the subgame that are main-game cards: (object in the subgame's
    /// outside-the-game zone, main-game object) (CR 729.4).
    pub outside: Vec<(ObjectId, ObjectId)>,
}

/// Whether a card is a supplementary-deck card: a nontraditional card that's part of a
/// deck (face down in the command zone), not a vanguard or dungeon.
fn is_supplementary(card: &CardDef) -> bool {
    let c = &card.front().chars;
    crate::variants::is_nontraditional(card)
        && !c.card_types.contains(CardType::Vanguard)
        && !c.card_types.contains(CardType::Dungeon)
}

/// Starts a subgame (CR 729.2): each player's main-game library, face-down supplementary
/// decks, vanguard, and a commander in the main-game command zone move into it. The
/// subgame's pregame procedure hasn't begun yet (see [`Game::start`]).
pub fn begin(g: &mut Game) -> Subgame {
    let n = g.players.len();
    let in_game: Vec<bool> = g.players.iter().map(|p| p.in_game()).collect();
    let mut decks: Vec<Vec<Arc<CardDef>>> = vec![Vec::new(); n];
    let mut commanders: Vec<(PlayerId, SmolStr)> = Vec::new();
    let mut moved: Vec<ObjectId> = Vec::new();
    for i in 0..n {
        if !in_game[i] {
            continue;
        }
        let p = PlayerId(i as u8);
        // CR 729.2: the whole library.
        for id in std::mem::take(&mut g.players[i].library) {
            if let Some(card) = g.obj(id).card.clone() {
                decks[i].push(card);
            }
            moved.push(id);
        }
        for id in g.command.clone() {
            let o = g.obj(id);
            if o.owner != p || o.kind != ObjKind::Card || !g.is_live(id) {
                continue;
            }
            let Some(card) = o.card.clone() else {
                continue;
            };
            let c = &card.front().chars;
            let take = if o.is_commander {
                // CR 729.2c: a commander in the main-game command zone.
                commanders.push((p, card.name.clone()));
                true
            } else if c.card_types.contains(CardType::Vanguard) {
                // CR 729.2b.
                true
            } else {
                // CR 729.2a: supplementary decks; face-up nontraditional cards stay.
                o.face_down && is_supplementary(&card)
            };
            if take {
                decks[i].push(card);
                g.command.retain(|x| *x != id);
                moved.push(id);
            }
        }
    }
    for id in moved {
        g.objects[id.0 as usize].zone = Zone::Nowhere;
    }
    // CR 729.2: randomly determine which player goes first.
    let players: Vec<PlayerId> = g.players_in_game();
    let chooser = players[g.random_range(0, players.len() as u32 - 1) as usize];
    let seed: u64 = g.rng.gen();
    let config = GameConfig {
        starting_player: None,
        first_turn_chooser: Some(chooser),
        seed,
        ..g.config.clone()
    };
    let mut sub = Game::new(config, decks, vec![]);
    sub.agents = g.agents.clone();
    sub.logging = g.logging;
    sub.subgames.depth = g.subgames.depth + 1;
    for (i, p) in g.players.iter().enumerate() {
        sub.players[i].name = p.name.clone();
        sub.players[i].team = p.team;
        if !in_game[i] {
            sub.players[i].left_game = true;
            sub.players[i].has_lost = p.has_lost;
        }
    }
    for (p, name) in commanders {
        sub.designate_commander(p, &name);
    }
    // CR 729.4: every main-game card a player owns, and their cards outside the main
    // game, are outside the subgame.
    let mut outside = Vec::new();
    let main_cards: Vec<ObjectId> = g
        .objects
        .iter()
        .filter(|o| {
            o.kind == ObjKind::Card
                && o.next.is_none()
                && o.zone != Zone::Nowhere
                && in_game[o.owner.idx()]
        })
        .map(|o| o.id)
        .collect();
    for id in main_cards {
        let o = g.obj(id);
        let Some(card) = o.card.clone() else {
            continue;
        };
        let owner = o.owner;
        let s = sub.create_card_object(card, owner, Zone::Outside(owner));
        sub.players[owner.idx()].sideboard.push(s);
        outside.push((s, id));
    }
    g.subgames.played += 1;
    g.log(|_| "--- A subgame begins ---".to_string());
    Subgame { game: sub, outside }
}

/// The cards represented by an object in the subgame: a merged or melded permanent's
/// components (CR 730.3), or the object itself.
fn represented_cards(sub: &Game, id: ObjectId) -> Vec<(PlayerId, Arc<CardDef>)> {
    let comps = crate::merge::physical_components(sub, id);
    let ids = if comps.is_empty() { vec![id] } else { comps };
    ids.into_iter()
        .filter_map(|c| {
            let o = sub.obj(c);
            (o.kind == ObjKind::Card)
                .then(|| o.card.clone().map(|card| (o.owner, card)))
                .flatten()
        })
        .collect()
}

/// Ends a subgame (CR 729.5): the cards return to the main game and the main game
/// continues. Returns the subgame's result.
pub fn finish(g: &mut Game, sub: Subgame) -> Option<GameResult> {
    let Subgame { game: sub, outside } = sub;
    let result = sub.result.clone();
    let n = g.players.len();
    let in_game: Vec<bool> = g.players.iter().map(|p| p.in_game()).collect();
    // Cards brought into the subgame from the main game (CR 729.4a): their main-game
    // objects left their zones; they go to their owners' libraries with the rest.
    let mut brought: Vec<ObjectId> = Vec::new();
    for (s, m) in &outside {
        if sub.is_live(*s) && matches!(sub.obj(*s).zone, Zone::Outside(_)) {
            continue;
        }
        brought.push(sub.current(*s));
        if g.is_live(*m) {
            let owner = g.obj(*m).owner;
            g.move_object(*m, Zone::Library(owner), MoveCause::Other, None);
        }
    }
    let mut to_library: Vec<(PlayerId, Arc<CardDef>)> = Vec::new();
    let mut to_decks: Vec<(PlayerId, Arc<CardDef>)> = Vec::new();
    let mut to_command: Vec<(PlayerId, Arc<CardDef>, bool)> = Vec::new();
    let mut commanders_elsewhere: Vec<(PlayerId, Arc<CardDef>)> = Vec::new();
    for o in &sub.objects {
        if o.next.is_some()
            || matches!(o.zone, Zone::Nowhere | Zone::Outside(_))
            || brought.contains(&o.id)
            || !in_game.get(o.owner.idx()).copied().unwrap_or(false)
        {
            continue;
        }
        for (owner, card) in represented_cards(&sub, o.id) {
            let c = &card.front().chars;
            let vanguard = c.card_types.contains(CardType::Vanguard);
            if crate::variants::is_nontraditional(&card) && !vanguard {
                // CR 729.5a: back into its supplementary deck (dungeons aren't part of
                // one and cease to exist).
                if is_supplementary(&card) {
                    to_decks.push((owner, card));
                }
            } else if o.zone == Zone::Command {
                // CR 729.5b, 729.5c: a vanguard or commander in the subgame command zone.
                if vanguard || o.is_commander {
                    to_command.push((owner, card, o.is_commander));
                }
            } else if o.is_commander {
                commanders_elsewhere.push((owner, card));
            } else {
                // CR 729.5: traditional cards anywhere else, exile and phased-out
                // permanents included.
                to_library.push((owner, card));
            }
        }
    }
    for (owner, card, is_commander) in to_command {
        let id = g.create_card_object(card, owner, Zone::Command);
        g.objects[id.0 as usize].is_commander = is_commander;
        g.command.push(id);
    }
    // A commander elsewhere in the subgame may be put into the main-game command zone
    // instead of the library (CR 903.9).
    for (owner, card) in commanders_elsewhere {
        let prompt = format!("Put {} into the command zone?", card.name);
        if g.ask_yes_no(owner, None, &prompt, true) {
            let id = g.create_card_object(card, owner, Zone::Command);
            g.objects[id.0 as usize].is_commander = true;
            g.command.push(id);
        } else {
            to_library.push((owner, card));
        }
    }
    for (owner, card) in to_library {
        let id = g.create_card_object(card, owner, Zone::Library(owner));
        g.players[owner.idx()].library.push(id);
    }
    for (owner, card) in to_decks {
        let id = g.create_card_object(card, owner, Zone::Command);
        g.objects[id.0 as usize].face_down = true;
        g.command.push(id);
    }
    for i in 0..n {
        if in_game[i] {
            g.shuffle_library(PlayerId(i as u8));
        }
    }
    shuffle_supplementary_decks(g);
    g.log(|_| "--- The subgame ends ---".to_string());
    // The subgame's objects cease to exist; it's kept for inspection.
    g.subgames.last = Some(Box::new(sub));
    g.dirty = true;
    result
}

/// Shuffles each player's face-down supplementary-deck cards in the command zone
/// (CR 729.5a).
fn shuffle_supplementary_decks(g: &mut Game) {
    use rand::seq::SliceRandom;
    for p in g.player_ids() {
        let slots: Vec<usize> = g
            .command
            .iter()
            .enumerate()
            .filter(|(_, id)| {
                let o = g.obj(**id);
                o.owner == p && o.face_down && o.card.as_deref().is_some_and(is_supplementary)
            })
            .map(|(i, _)| i)
            .collect();
        let mut cards: Vec<ObjectId> = slots.iter().map(|i| g.command[*i]).collect();
        cards.shuffle(&mut g.rng);
        for (k, i) in slots.into_iter().enumerate() {
            g.command[i] = cards[k];
        }
    }
}

/// Plays a subgame to its end (CR 729). Returns its result.
pub fn play(g: &mut Game) -> Option<GameResult> {
    let mut sub = begin(g);
    sub.game.run();
    finish(g, sub)
}

/// The players who won a subgame with this result.
pub fn winners(result: &Option<GameResult>) -> Vec<PlayerId> {
    match result {
        Some(GameResult::Win(v)) => v.clone(),
        _ => vec![],
    }
}

struct SubgameRules;

impl KeywordRules for SubgameRules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        match name {
            PLAY_SUBGAME => {
                let result = play(g);
                let w = winners(&result).into_iter().map(Entity::Player).collect();
                ctx.set_var(WINNERS, w);
                true
            }
            NON_WINNERS_LOSE_HALF => {
                let won: Vec<PlayerId> = ctx
                    .vars
                    .get(&WINNERS)
                    .map(|v| v.iter().filter_map(|e| e.player()).collect())
                    .unwrap_or_default();
                for p in g.apnap() {
                    if won.contains(&p) {
                        continue;
                    }
                    let life = g.player(p).life;
                    if life > 0 {
                        g.lose_life(p, ((life + 1) / 2) as u32);
                    }
                }
                true
            }
            _ => false,
        }
    }
}

inventory::submit! { KeywordRegistration(&SubgameRules) }
