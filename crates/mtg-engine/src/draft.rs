//! Booster drafts: the Conspiracy Draft variant's draft (CR 905.1, 905.2) and the
//! Commander Draft option's (CR 903.13).
//!
//! In each draft round each player opens a booster pack, drafts one card from it (two in
//! Commander Draft) and passes the rest; then each player drafts from the pack passed to
//! them, and so on until every card of the round has been drafted (CR 905.1a, 903.13b).
//! Packs go to each player's left in the first and third rounds and to the right in the
//! second (CR 905.1b, 903.13c). Seats are player indices; a player's left is the next
//! seat (the next player in turn order).
//!
//! There's no active player or priority during a draft: every player drafts from the
//! pack in front of them, in any order, and the packs are passed once all have drafted;
//! players who want to act at the same time act in a random order (CR 905.2a). A player
//! sees only the pack they're drafting from, the cards they've drafted, cards revealed as
//! they were drafted, and cards drafted face up (CR 905.1c, 903.13d).
//!
//! Some cards function during the draft (CR 905.2): "Draft [this card] face up"
//! ([`DRAFT_FACE_UP`], CR 905.2c) and "Reveal [this card] as you draft it and note ..."
//! ([`DRAFT_NOTE_COUNT`], [`DRAFT_NOTE_PASSER`], CR 905.2b). Noted information is kept in
//! [`DraftInfo`], which a game can refer to ([`HIGHEST_NOTED`]).

use crate::card::CardDef;
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::*;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use smol_str::SmolStr;
use std::sync::Arc;

/// `StaticEffect::Custom` name of "Draft [this card] face up." (CR 905.2c).
pub const DRAFT_FACE_UP: &str = "draft:face up";
/// `StaticEffect::Custom` name of "Reveal [this card] as you draft it and note how many
/// cards you've drafted this draft round, including [this card]." (CR 905.2b).
pub const DRAFT_NOTE_COUNT: &str = "draft:reveal and note cards drafted this round";
/// `StaticEffect::Custom` name of "Reveal [this card] as you draft it and note the player
/// who passed it to you." (CR 905.2b).
pub const DRAFT_NOTE_PASSER: &str = "draft:reveal and note the passer";
/// `Value::Custom` name: "the highest number you noted for cards named [this card's
/// name]" (CR 905.2b).
pub const HIGHEST_NOTED: &str = "draft:highest number noted for cards with this name";

/// How many cards a player drafts from each pack passed to them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DraftStyle {
    /// Conspiracy Draft: one card (CR 905.1a).
    Conspiracy,
    /// Commander Draft: two cards (CR 903.13b).
    Commander,
}

impl DraftStyle {
    fn picks(self) -> usize {
        match self {
            DraftStyle::Conspiracy => 1,
            DraftStyle::Commander => 2,
        }
    }
}

/// Which way packs are passed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PassDirection {
    Left,
    Right,
}

/// A card in a player's drafted cards pile.
#[derive(Clone, Debug)]
pub struct DraftedCard {
    pub card: Arc<CardDef>,
    /// Drafted face up (CR 905.2c): all players may look at it.
    pub face_up: bool,
}

/// Information noted as a card was drafted (CR 905.2b). Any player can look at it during
/// the draft or the game.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DraftNote {
    /// The player who drafted the card.
    pub player: PlayerId,
    /// The card's name.
    pub card: SmolStr,
    /// A noted number, if any.
    pub number: Option<i64>,
    /// A noted player, if any.
    pub noted_player: Option<PlayerId>,
}

/// What a game needs to know about the draft before it (CR 905.2b).
#[derive(Clone, Debug, Default)]
pub struct DraftInfo {
    pub notes: Vec<DraftNote>,
}

/// Why a draft action isn't allowed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DraftError {
    /// The player already drafted from the pack in front of them; it's waiting to be
    /// passed.
    AlreadyDrafted,
    /// The wrong number of cards, or cards that aren't in the pack.
    InvalidPick,
    /// The draft is over.
    DraftComplete,
    /// No such face-up drafted card.
    NotFaceUp,
}

/// A booster draft in progress.
#[derive(Clone, Debug)]
pub struct Draft {
    style: DraftStyle,
    /// Each player's unopened booster packs, in the order they'll be opened.
    boosters: Vec<Vec<Vec<Arc<CardDef>>>>,
    /// The pack in front of each player.
    packs: Vec<Vec<Arc<CardDef>>>,
    /// Who passed the pack in front of each player to them (none for a pack they opened).
    passer: Vec<Option<PlayerId>>,
    /// Whether each player has drafted from the pack in front of them.
    drafted_from_pack: Vec<bool>,
    /// The current draft round (1-based; 0 before the first).
    pub round: usize,
    /// Each player's drafted cards pile.
    drafted: Vec<Vec<DraftedCard>>,
    /// Cards each player drafted this round.
    this_round: Vec<usize>,
    /// Cards revealed as they were drafted (CR 905.2b).
    revealed: Vec<(PlayerId, Arc<CardDef>)>,
    pub info: DraftInfo,
    rng: ChaCha8Rng,
}

fn has_draft_ability(card: &CardDef, name: &str) -> bool {
    card.front().chars.abilities.iter().any(|a| {
        matches!(&a.kind, crate::ability::AbilityKind::Static(s)
            if matches!(&s.effect, crate::ability::StaticEffect::Custom(n) if n.as_str() == name))
    })
}

impl Draft {
    /// A draft among `boosters.len()` players; `boosters[p]` are the packs player `p` will
    /// open, one per round. The first round begins at once.
    pub fn new(boosters: Vec<Vec<Vec<Arc<CardDef>>>>, style: DraftStyle, seed: u64) -> Draft {
        let n = boosters.len();
        let mut d = Draft {
            style,
            boosters: boosters
                .into_iter()
                .map(|mut v| {
                    v.reverse();
                    v
                })
                .collect(),
            packs: vec![vec![]; n],
            passer: vec![None; n],
            drafted_from_pack: vec![false; n],
            round: 0,
            drafted: vec![vec![]; n],
            this_round: vec![0; n],
            revealed: vec![],
            info: DraftInfo::default(),
            rng: ChaCha8Rng::seed_from_u64(seed),
        };
        d.start_round();
        d
    }

    pub fn players(&self) -> usize {
        self.packs.len()
    }

    /// The direction packs are passed in a draft round (CR 905.1b, 903.13c): left in the
    /// first and third rounds, right in the second (alternating after that).
    pub fn direction(round: usize) -> PassDirection {
        if round % 2 == 0 {
            PassDirection::Right
        } else {
            PassDirection::Left
        }
    }

    /// Each player opens their next booster pack. Returns false if none are left.
    fn start_round(&mut self) -> bool {
        if self.boosters.iter().all(|b| b.is_empty()) {
            return false;
        }
        self.round += 1;
        for p in 0..self.players() {
            self.packs[p] = self.boosters[p].pop().unwrap_or_default();
            self.passer[p] = None;
            self.drafted_from_pack[p] = false;
            self.this_round[p] = 0;
        }
        true
    }

    /// Whether every card has been drafted.
    pub fn is_complete(&self) -> bool {
        self.packs.iter().all(|p| p.is_empty()) && self.boosters.iter().all(|b| b.is_empty())
    }

    /// The pack `p` is drafting from.
    pub fn pack(&self, p: PlayerId) -> &[Arc<CardDef>] {
        &self.packs[p.idx()]
    }

    /// The player who passed `p` the pack in front of them.
    pub fn passed_by(&self, p: PlayerId) -> Option<PlayerId> {
        self.passer[p.idx()]
    }

    /// `p`'s drafted cards pile.
    pub fn drafted(&self, p: PlayerId) -> &[DraftedCard] {
        &self.drafted[p.idx()]
    }

    /// The cards `p` may look at (CR 905.1c, 903.13d): the pack they're drafting from,
    /// the cards they've drafted, cards revealed as they were drafted (CR 905.2b), and
    /// cards other players drafted face up (CR 905.2c).
    pub fn visible(&self, p: PlayerId) -> Vec<Arc<CardDef>> {
        let mut out: Vec<Arc<CardDef>> = self.packs[p.idx()].clone();
        out.extend(self.drafted[p.idx()].iter().map(|d| d.card.clone()));
        for (q, c) in &self.revealed {
            if *q != p {
                out.push(c.clone());
            }
        }
        for (q, pile) in self.drafted.iter().enumerate() {
            if q != p.idx() {
                out.extend(pile.iter().filter(|d| d.face_up).map(|d| d.card.clone()));
            }
        }
        out
    }

    /// Whether `p` may look at a card named `name` right now.
    pub fn can_see(&self, p: PlayerId, name: &str) -> bool {
        self.visible(p).iter().any(|c| c.name == name)
    }

    /// `p` drafts the cards at `picks` from the pack in front of them: as many as the
    /// draft style says, or all that are left (CR 905.1a, 903.13b). Once every player
    /// with a pack has drafted, the packs are passed; when a round's cards are all
    /// drafted, the next round begins.
    pub fn pick(&mut self, p: PlayerId, picks: &[usize]) -> Result<(), DraftError> {
        if self.is_complete() {
            return Err(DraftError::DraftComplete);
        }
        let i = p.idx();
        if self.drafted_from_pack[i] {
            return Err(DraftError::AlreadyDrafted);
        }
        let want = self.style.picks().min(self.packs[i].len());
        let mut idx: Vec<usize> = picks.to_vec();
        idx.sort_unstable();
        idx.dedup();
        if idx.len() != want || idx.iter().any(|k| *k >= self.packs[i].len()) {
            return Err(DraftError::InvalidPick);
        }
        for k in idx.into_iter().rev() {
            let card = self.packs[i].remove(k);
            self.this_round[i] += 1;
            self.draft_card(p, card);
        }
        self.drafted_from_pack[i] = true;
        let waiting =
            (0..self.players()).any(|q| !self.drafted_from_pack[q] && !self.packs[q].is_empty());
        if !waiting {
            self.pass();
        }
        Ok(())
    }

    /// Puts a drafted card in `p`'s pile, applying its draft abilities (CR 905.2).
    fn draft_card(&mut self, p: PlayerId, card: Arc<CardDef>) {
        // CR 905.2b: reveal it, note the information, then turn it face down.
        let count = has_draft_ability(&card, DRAFT_NOTE_COUNT);
        let passer = has_draft_ability(&card, DRAFT_NOTE_PASSER);
        if count || passer {
            self.revealed.push((p, card.clone()));
            self.info.notes.push(DraftNote {
                player: p,
                card: card.name.clone(),
                number: count.then_some(self.this_round[p.idx()] as i64),
                noted_player: if passer { self.passer[p.idx()] } else { None },
            });
        }
        // CR 905.2c: drafted face up.
        let face_up = has_draft_ability(&card, DRAFT_FACE_UP);
        self.drafted[p.idx()].push(DraftedCard { card, face_up });
    }

    /// Passes each pack to the next player in the round's direction.
    fn pass(&mut self) {
        let n = self.players();
        if self.packs.iter().all(|p| p.is_empty()) {
            self.start_round();
            return;
        }
        let dir = Draft::direction(self.round);
        let mut packs = vec![vec![]; n];
        let mut passer = vec![None; n];
        for from in 0..n {
            let to = match dir {
                PassDirection::Left => (from + 1) % n,
                PassDirection::Right => (from + n - 1) % n,
            };
            packs[to] = std::mem::take(&mut self.packs[from]);
            passer[to] = Some(PlayerId(from as u8));
        }
        self.packs = packs;
        self.passer = passer;
        self.drafted_from_pack = vec![false; n];
    }

    /// An effect instructs `p` to turn a card they drafted face up face down
    /// (CR 905.2c). `k` indexes their drafted cards pile.
    pub fn turn_face_down(&mut self, p: PlayerId, k: usize) -> Result<(), DraftError> {
        match self.drafted[p.idx()].get_mut(k) {
            Some(d) if d.face_up => {
                d.face_up = false;
                Ok(())
            }
            _ => Err(DraftError::NotFaceUp),
        }
    }

    /// Players who wish to take an action at the same time during the draft and can't
    /// agree on an order take them in a random order (CR 905.2a).
    pub fn simultaneous_order(&mut self, players: &[PlayerId]) -> Vec<PlayerId> {
        let mut v = players.to_vec();
        v.shuffle(&mut self.rng);
        v
    }

    /// The cards `p` has drafted: their card pool once the draft is complete
    /// (CR 905.1d, 903.13e).
    pub fn pool(&self, p: PlayerId) -> Vec<Arc<CardDef>> {
        self.drafted[p.idx()]
            .iter()
            .map(|d| d.card.clone())
            .collect()
    }

    /// Ends the draft: face-up drafted cards remain face up only until the draft is
    /// complete (CR 905.2c). Returns each player's card pool and the draft information
    /// the game may refer to.
    pub fn finish(mut self) -> (Vec<Vec<Arc<CardDef>>>, DraftInfo) {
        for pile in self.drafted.iter_mut() {
            for d in pile.iter_mut() {
                d.face_up = false;
            }
        }
        let pools = (0..self.players())
            .map(|p| self.pool(PlayerId(p as u8)))
            .collect();
        (pools, self.info)
    }
}

/// Values referring to information noted during the draft (CR 905.2b): "the highest
/// number you noted for cards named [this card's name]".
pub fn custom_value(g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
    if name != HIGHEST_NOTED {
        return None;
    }
    let card = ctx
        .source
        .and_then(|s| g.try_obj(g.ability_source_of(s)))
        .map(|o| o.chars.name.clone())?;
    Some(
        g.start
            .draft
            .notes
            .iter()
            .filter(|n| n.player == ctx.controller && n.card == card)
            .filter_map(|n| n.number)
            .max()
            .unwrap_or(0),
    )
}
