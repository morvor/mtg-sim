//! Starting the game (CR 103): determining the starting player, the additional steps taken
//! before decks are shuffled (sideboards, companions, commanders, sticker sheets and
//! conspiracies, CR 103.2), starting life totals (CR 103.4) and starting hand sizes
//! (CR 103.5a).

use crate::ability::*;
use crate::card::CardDef;
use crate::decision::Answer;
use crate::eval::Ctx;
use crate::game::{Game, Variant};
use crate::object::Zone;
use crate::types::*;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::collections::BTreeMap;
use std::sync::Arc;

/// `StaticEffect::Custom` name of "You are the starting player." (Power Play, CR 103.1c).
pub const YOU_ARE_STARTING_PLAYER: &str = "you are the starting player";

/// A companion's condition on its owner's starting deck (CR 702.139a).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DeckCondition {
    /// Each card in the starting deck that matches `each` also matches `must` ("Each
    /// permanent card in your starting deck has mana value 2 or less").
    Each { each: Filter, must: Filter },
}

/// What happened while starting the game.
#[derive(Clone, Debug, Default)]
pub struct StartState {
    /// The player who chose who takes the first turn (CR 103.1).
    pub chooser: Option<PlayerId>,
    /// With shared team turns, the team that takes the first turn (CR 103.1a).
    pub starting_team: Option<u8>,
    /// Each player's starting deck (CR 103.2a): the cards in it when it was determined.
    pub starting_decks: BTreeMap<PlayerId, Vec<ObjectId>>,
    /// The sticker sheets each player brings to the game (CR 123.2a, 123.2b); the ones
    /// chosen at the start of the game are the ones they have access to
    /// ([`crate::stickers::sheets_of`], CR 103.2d, 123.2c).
    pub sticker_sheets: BTreeMap<PlayerId, Vec<crate::stickers::StickerSheet>>,
    /// The names of the sticker sheets each player revealed (CR 103.2d).
    pub revealed_sticker_sheets: BTreeMap<PlayerId, Vec<SmolStr>>,
}

impl Game {
    /// The archenemy (CR 904.2a): the only player on a one-player team. In a Supervillain
    /// Rumble game every player is an archenemy (CR 904.12b), so there is no single one.
    pub fn archenemy(&self) -> Option<PlayerId> {
        let all: Vec<PlayerId> = self
            .player_ids()
            .into_iter()
            .filter(|p| crate::life_totals::is_archenemy(self, *p))
            .collect();
        (all.len() == 1).then(|| all[0])
    }

    /// Whether this is a Commander game using the Brawl option (CR 903.12).
    pub fn is_brawl(&self) -> bool {
        self.config.variant == Variant::Commander && self.config.brawl
    }

    /// The face-up vanguard card a player owns (CR 902.3).
    pub fn vanguard_of(&self, p: PlayerId) -> Option<ObjectId> {
        self.command.iter().copied().find(|id| {
            let o = self.obj(*id);
            o.owner == p && !o.face_down && o.base.is(CardType::Vanguard)
        })
    }

    /// A player's starting life total (CR 103.4, 119.1): 20 (or the configured amount), 30
    /// for each Two-Headed Giant team (CR 103.4a), 40 in Commander (CR 103.4c), 25 or 30 in
    /// Brawl (CR 103.4d), 40 for the archenemy (CR 103.4e); a vanguard's life modifier
    /// applies (CR 103.4b).
    pub fn starting_life(&self, p: PlayerId) -> i32 {
        crate::life_totals::starting_life(self, p)
    }

    /// A player's starting hand size (CR 103.5): normally seven, modified by their
    /// vanguard's hand modifier (CR 103.5a).
    pub fn starting_hand_size(&self, p: PlayerId) -> u32 {
        let modifier = self
            .vanguard_of(p)
            .and_then(|v| self.obj(v).base.hand_modifier)
            .unwrap_or(0);
        (self.config.starting_hand_size as i32 + modifier).max(0) as u32
    }

    /// Designates the card named `name` in `p`'s deck as their commander (CR 903.3), before
    /// the game starts. Returns false if the deck has no such card.
    pub fn designate_commander(&mut self, p: PlayerId, name: &str) -> bool {
        let Some(id) = self
            .player(p)
            .library
            .iter()
            .copied()
            .find(|id| self.obj(*id).base.name == name)
        else {
            return false;
        };
        self.objects[id.0 as usize].is_commander = true;
        self.players[p.idx()]
            .commander_names
            .push(SmolStr::new(name));
        true
    }

    /// Adds cards to a player's sideboard (CR 100.4), outside the game.
    pub fn add_to_sideboard(&mut self, p: PlayerId, cards: Vec<Arc<CardDef>>) -> Vec<ObjectId> {
        cards
            .into_iter()
            .map(|c| {
                let id = self.create_card_object(c, p, Zone::Outside(p));
                self.players[p.idx()].sideboard.push(id);
                id
            })
            .collect()
    }

    /// The sticker sheets a player has access to (CR 123.2c): those chosen at the start of
    /// the game, or none if they play without sticker sheets.
    pub fn accessible_sticker_sheets(&self, p: PlayerId) -> &[crate::stickers::StickerSheet] {
        crate::stickers::sheets_of(self, p)
    }

    /// The companion a player revealed (CR 103.2b), if any.
    pub fn companion_of(&self, p: PlayerId) -> Option<ObjectId> {
        self.special
            .companions
            .iter()
            .find(|(q, _, _)| *q == p)
            .map(|(_, c, _)| *c)
    }
}

/// CR 103.1: the players determine (randomly, by default) which of them chooses who takes
/// the first turn, and that player chooses. In a match, the configured chooser (the loser
/// of the previous game) chooses. In an Archenemy game the archenemy goes first
/// (CR 103.1b). With shared team turns a starting team is chosen (CR 103.1a, 805.3),
/// represented by its first player.
pub fn choose_starting_player(g: &mut Game) -> PlayerId {
    let p = choose_starting_player_inner(g);
    if g.uses_shared_team_turns() {
        g.start.starting_team = Some(g.player(p).team);
    }
    p
}

fn choose_starting_player_inner(g: &mut Game) -> PlayerId {
    if let Some(p) = g.config.starting_player {
        return p;
    }
    if let Some(a) = g.archenemy() {
        return a;
    }
    let n = g.players.len() as u32;
    let chooser = match g.config.first_turn_chooser {
        Some(c) => c,
        None => PlayerId(g.random_range(0, n - 1) as u8),
    };
    g.start.chooser = Some(chooser);
    let shared = g.uses_shared_team_turns();
    let options: Vec<PlayerId> = if shared {
        g.team_representatives()
    } else {
        g.player_ids()
    };
    // The chooser's own seat (or team) is the default answer.
    let own = options
        .iter()
        .copied()
        .find(|p| shared && g.player(*p).team == g.player(chooser).team)
        .unwrap_or(chooser);
    let mut cands: Vec<Entity> = vec![Entity::Player(own)];
    cands.extend(
        options
            .into_iter()
            .filter(|p| *p != own)
            .map(Entity::Player),
    );
    g.ask_entities(
        chooser,
        None,
        "Choose who takes the first turn",
        cands,
        1,
        1,
    )
    .first()
    .and_then(|e| e.player())
    .unwrap_or(own)
}

/// CR 103.2: the additional steps taken after the starting player has been determined, in
/// order, then effects that make a player the starting player (CR 103.1c).
pub fn additional_steps(g: &mut Game) {
    set_aside_sideboards(g);
    reveal_companions(g);
    commanders_to_command_zone(g);
    choose_sticker_sheets(g);
    conspiracies_to_command_zone(g);
    apply_starting_player_effects(g);
}

/// CR 103.2a: sideboard cards are set aside (they stay outside the game); each player's
/// deck is now their starting deck.
fn set_aside_sideboards(g: &mut Game) {
    for p in g.player_ids() {
        let deck = g.player(p).library.clone();
        g.start.starting_decks.insert(p, deck);
    }
}

/// Whether `p`'s starting deck fulfills a companion condition (CR 702.139a).
pub fn deck_fulfills(g: &Game, p: PlayerId, cond: &DeckCondition, companion: ObjectId) -> bool {
    let deck = g.start.starting_decks.get(&p).cloned().unwrap_or_default();
    let ctx = Ctx::new(Some(companion), p);
    match cond {
        DeckCondition::Each { each, must } => deck
            .iter()
            .filter(|c| g.matches(**c, each, &ctx))
            .all(|c| g.matches(*c, must, &ctx)),
    }
}

/// The companion condition of a card, if it has companion (CR 702.139a).
pub fn companion_condition(g: &Game, card: ObjectId) -> Option<DeckCondition> {
    g.obj(card)
        .base
        .abilities
        .iter()
        .find_map(|a| match &a.kind {
            AbilityKind::Static(s) => match &s.effect {
                StaticEffect::Companion(c) => Some(c.clone()),
                _ => None,
            },
            _ => None,
        })
}

/// CR 103.2b: each player, starting with the starting player (CR 101.4e), may reveal one
/// card with companion they own from outside the game whose condition their deck
/// fulfills. The card remains outside the game.
fn reveal_companions(g: &mut Game) {
    for p in g.apnap() {
        let cands: Vec<ObjectId> = g
            .player(p)
            .sideboard
            .clone()
            .into_iter()
            .filter(|c| {
                companion_condition(g, *c).is_some_and(|cond| deck_fulfills(g, p, &cond, *c))
            })
            .collect();
        if cands.is_empty() {
            continue;
        }
        let pick = g.ask_objects(p, None, "Reveal a companion?", cands, 0, 1);
        if let Some(c) = pick.first() {
            // Recorded for the companion's special action (CR 702.139a).
            crate::kw::companion::choose_companion(g, p, *c);
        }
    }
}

/// CR 103.2c: in a Commander game, each player puts their commander from their deck face
/// up into the command zone (CR 903.6).
fn commanders_to_command_zone(g: &mut Game) {
    if g.config.variant != Variant::Commander {
        return;
    }
    for p in g.player_ids() {
        let cmdrs: Vec<ObjectId> = g
            .player(p)
            .library
            .iter()
            .copied()
            .filter(|id| g.obj(*id).is_commander)
            .collect();
        for c in cmdrs {
            g.players[p.idx()].library.retain(|x| *x != c);
            g.objects[c.0 as usize].zone = Zone::Command;
            g.objects[c.0 as usize].face_down = false;
            g.command.push(c);
        }
    }
    g.dirty = true;
}

/// CR 103.2d: in a constructed game, each player playing with sticker sheets reveals all of
/// them and chooses three at random; in a limited game, each such player chooses up to
/// three and reveals them. Those are the only stickers the player has access to
/// (CR 123.2c).
fn choose_sticker_sheets(g: &mut Game) {
    use crate::stickers::{choose_sheets, SheetFormat};
    for p in g.apnap() {
        let Some(sheets) = g.start.sticker_sheets.get(&p).cloned() else {
            continue;
        };
        if sheets.is_empty() {
            continue;
        }
        if g.config.limited {
            let mut chosen: Vec<usize> = Vec::new();
            while chosen.len() < 3 {
                let rest: Vec<usize> = (0..sheets.len()).filter(|i| !chosen.contains(i)).collect();
                if rest.is_empty() {
                    break;
                }
                let mut options = vec!["Done".to_string()];
                options.extend(rest.iter().map(|i| sheets[*i].name.to_string()));
                let k = g.ask_option(p, None, "Choose a sticker sheet", options);
                if k == 0 {
                    break;
                }
                chosen.push(rest[k - 1]);
            }
            let picked: Vec<_> = chosen.iter().map(|i| sheets[*i].clone()).collect();
            if let Ok(names) = choose_sheets(g, p, picked, SheetFormat::Limited) {
                g.start.revealed_sticker_sheets.insert(p, names);
            }
        } else {
            let all: Vec<SmolStr> = sheets.iter().map(|s| s.name.clone()).collect();
            if choose_sheets(g, p, sheets, SheetFormat::Constructed).is_ok() {
                g.start.revealed_sticker_sheets.insert(p, all);
            }
        }
    }
}

/// CR 103.2e, 905.4: each player may put any number of conspiracy cards from their
/// sideboard into the command zone; those with hidden agenda go face down (CR 905.4a).
fn conspiracies_to_command_zone(g: &mut Game) {
    for p in g.apnap() {
        let cands: Vec<ObjectId> = g
            .player(p)
            .sideboard
            .iter()
            .copied()
            .filter(|c| g.obj(*c).base.is(CardType::Conspiracy))
            .collect();
        if cands.is_empty() {
            continue;
        }
        let n = cands.len() as u32;
        let pick = match g.ask(
            p,
            crate::decision::Decision::ChooseEntities {
                source: None,
                prompt: "Put conspiracies into the command zone".into(),
                candidates: cands.iter().map(|c| Entity::Object(*c)).collect(),
                min: 0,
                max: n,
            },
        ) {
            Answer::Entities(v)
                if v.iter()
                    .all(|e| e.object().is_some_and(|o| cands.contains(&o))) =>
            {
                v.into_iter().filter_map(|e| e.object()).collect()
            }
            _ => cands.clone(),
        };
        for c in pick {
            g.players[p.idx()].sideboard.retain(|x| *x != c);
            g.objects[c.0 as usize].zone = Zone::Command;
            // CR 702.106a: face down, with a secretly chosen card name.
            crate::kw::hidden_agenda::as_put_into_command_zone(g, p, c);
            g.command.push(c);
        }
    }
    g.dirty = true;
}

/// CR 103.1c: an effect that says its controller is the starting player (Power Play)
/// applies after the starting player has been determined and supersedes it. If several
/// players would be the starting player, one of them is chosen at random.
fn apply_starting_player_effects(g: &mut Game) {
    g.recompute();
    let mut claimants: Vec<PlayerId> = Vec::new();
    for id in g.command.clone() {
        let o = g.obj(id);
        if o.face_down {
            continue;
        }
        let says = o.chars.abilities.iter().any(|a| {
            matches!(&a.kind, AbilityKind::Static(s)
                if matches!(&s.effect, StaticEffect::Custom(n) if n == YOU_ARE_STARTING_PLAYER))
        });
        if says && !claimants.contains(&o.controller) {
            claimants.push(o.controller);
        }
    }
    let starting = match claimants.len() {
        0 => return,
        1 => claimants[0],
        n => claimants[g.random_range(0, n as u32 - 1) as usize],
    };
    g.log(|_| format!("{starting} is the starting player"));
    g.turn.starting_player = starting;
    g.turn.active = starting;
    if g.uses_shared_team_turns() {
        g.start.starting_team = Some(g.player(starting).team);
    }
}
