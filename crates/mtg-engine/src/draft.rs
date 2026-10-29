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
//! sees only the pack they're drafting from, the cards they've drafted, cards currently
//! revealed as they're drafted, and cards drafted face up (CR 905.1c, 903.13d). A card
//! revealed as it's drafted stays revealed while the players draft from the packs in
//! front of them; it's turned face down as the packs are passed, and only the noted
//! information stays public (CR 905.2b).
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

/// `StaticEffect::Custom` name of "Reveal [this card] as you draft it. Reveal the next card
/// you draft and note its name." (Aether Searcher).
pub const DRAFT_NOTE_NEXT_NAME: &str = "draft:reveal and note the next card's name";
/// `StaticEffect::Custom` names of "As you draft a card, you may remove it from the draft
/// face up." (Animus of Predation) / "... face down." (Cogwork Grinder): see
/// [`DraftOption::Remove`].
pub const DRAFT_REMOVE_FACE_UP: &str = "draft:may remove drafted cards face up";
pub const DRAFT_REMOVE_FACE_DOWN: &str = "draft:may remove drafted cards face down";
/// `StaticEffect::Custom` names of "As you draft a [creature] card, you may reveal it, note
/// its name, then turn [this card] face down." (Noble Banneret, Smuggler Captain) and
/// "... note its creature types ..." (Paliano Vanguard): see
/// [`DraftOption::RevealAndNote`]. The suffix `:creature` limits it to creature cards.
pub const DRAFT_NOTE_NAME: &str = "draft:may reveal and note a drafted card's name";
pub const DRAFT_NOTE_TYPES: &str = "draft:may reveal and note a drafted card's creature types";
/// `StaticEffect::Custom` name of "Reveal [this card] as you draft it. The player to your
/// right chooses a color, you choose another color, then the player to your left chooses
/// a third color." (Paliano, the High City; Regicide).
pub const DRAFT_THREE_COLORS: &str = "draft:reveal and note three colors";
/// `StaticEffect::Custom` name of "As you draft a card, you may draft an additional card
/// from that booster pack. If you do, put [this card] into that booster pack." (Cogwork
/// Librarian): see [`DraftOption::Additional`].
pub const DRAFT_ADDITIONAL_PUT_BACK: &str = "draft:may draft an additional card, put this back";
/// `StaticEffect::Custom` name of "As you draft a card, you may draft an additional card
/// from that booster pack. If you do, turn [this card] face down, then pass the next
/// booster pack without drafting a card from it." (Leovold's Operative).
pub const DRAFT_ADDITIONAL_SKIP_NEXT: &str = "draft:may draft an additional card, skip the next";
/// `StaticEffect::Custom` name of "Reveal [this card] as you draft it. The next time a
/// player drafts a card from this booster pack, guess that card's name. Then that player
/// reveals the drafted card." (Spire Phantasm): see [`Draft::guess`].
pub const DRAFT_GUESS_NEXT: &str = "draft:reveal and guess the next card drafted from this pack";
/// `StaticEffect::Custom` name of "Reveal [this card] as you draft it. You may look at the
/// next card drafted from this booster pack." (Cogwork Spy).
pub const DRAFT_LOOK_NEXT: &str = "draft:reveal and look at the next card drafted from this pack";
/// `StaticEffect::Custom` name of "Reveal [this card] as you draft it. After you draft
/// [this card], you may add a booster pack to the draft." (Lore Seeker): see
/// [`Draft::add_booster`].
pub const DRAFT_ADD_BOOSTER: &str = "draft:reveal; after you draft it, you may add a booster pack";
/// `StaticEffect::Custom` name of "Each player passes the last card from each booster pack
/// to a player who drafted a card named [this card's name]." (Canal Dredger): see
/// [`Draft::choose_last_card_recipient`].
pub const DRAFT_LAST_CARD_TO_DRAFTER: &str = "draft:last cards go to its drafters";
/// `Condition::Custom` name: "you guessed correctly for a card named [this card's name]".
pub const GUESSED_CORRECTLY: &str = "draft:guessed correctly for cards with this name";

/// `Filter::Custom` names for the game (CR 905.2b): a card with a name the filter's
/// player noted for cards with the source's name; a creature of a creature type they
/// noted; an object that's one or more of the colors chosen as they drafted cards with
/// the source's name.
pub const NOTED_NAME: &str = "draft:a name noted for cards with this name";
pub const NOTED_TYPE: &str = "draft:a creature type noted for cards with this name";
pub const CHOSEN_COLOR: &str = "draft:a color chosen as cards with this name were drafted";
/// `Value::Custom` name: "the number of cards you removed from the draft with cards named
/// [this card's name]".
pub const REMOVED_COUNT: &str = "draft:cards removed with cards with this name";
/// `Condition::Custom` prefix: "you removed a creature card with [keyword] from the draft
/// with cards named [this card's name]" (`draft:removed creature with:Flying`).
pub const REMOVED_CREATURE_WITH: &str = "draft:removed a creature card with";

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
    /// A noted card name (Aether Searcher, Noble Banneret).
    pub noted_name: Option<SmolStr>,
    /// Noted creature types (Paliano Vanguard).
    pub noted_types: Vec<SmolStr>,
    /// Colors chosen as the card was drafted (Paliano, the High City).
    pub colors: Vec<Color>,
    /// Whether the player guessed the name of the card drafted next from the booster pack
    /// correctly (Spire Phantasm).
    pub guessed_correctly: Option<bool>,
}

impl DraftNote {
    fn new(player: PlayerId, card: SmolStr) -> DraftNote {
        DraftNote {
            player,
            card,
            number: None,
            noted_player: None,
            noted_name: None,
            noted_types: vec![],
            colors: vec![],
            guessed_correctly: None,
        }
    }
}

/// A card a player removed from the draft with the ability of a card named `with`
/// (CR 905.2): it isn't in anyone's card pool.
#[derive(Clone, Debug)]
pub struct RemovedCard {
    pub player: PlayerId,
    pub with: SmolStr,
    pub card: Arc<CardDef>,
    pub face_up: bool,
}

/// What a game needs to know about the draft before it (CR 905.2b).
#[derive(Clone, Debug, Default)]
pub struct DraftInfo {
    pub notes: Vec<DraftNote>,
    /// Cards removed from the draft (Animus of Predation, Cogwork Grinder).
    pub removed: Vec<RemovedCard>,
}

/// A choice made with a drafted card's ability as a card is drafted (CR 905.2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DraftOption {
    /// "As you draft a card, you may remove it from the draft face up/down" with the
    /// face-up card named `with` (CR 905.2c): the card is drafted, then removed.
    Remove { with: SmolStr },
    /// "As you draft a creature card, you may reveal it, note its name (or creature
    /// types), then turn [the card named `with`] face down."
    RevealAndNote { with: SmolStr },
    /// "As you draft a card, you may draft an additional card from that booster pack" with
    /// the face-up card named `with`: the card at `extra` (an index in the pack before
    /// either is drafted) is drafted too, one at a time.
    Additional { with: SmolStr, extra: usize },
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
    /// The option can't be used for that card (no such ability, or not that kind of card).
    InvalidOption,
}

/// A booster pack waiting in front of a player behind the one they're drafting from: a
/// pack added to the draft (Lore Seeker) or passed to them in addition to another one
/// (Canal Dredger).
#[derive(Clone, Debug)]
struct QueuedPack {
    cards: Vec<Arc<CardDef>>,
    id: usize,
    passer: Option<PlayerId>,
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
    /// Cards currently revealed as they're drafted (CR 905.2b): until the packs are
    /// passed.
    revealed: Vec<(PlayerId, Arc<CardDef>)>,
    pub info: DraftInfo,
    rng: ChaCha8Rng,
    /// Players whose next drafted card's name is noted for a card name (Aether Searcher).
    next_name: Vec<(PlayerId, SmolStr)>,
    /// Players who pass the next booster pack without drafting from it, once each
    /// (Leovold's Operative).
    skip_next: Vec<PlayerId>,
    /// Colors set in advance for the next three-color note of each player (the right
    /// player's, theirs, the left player's), instead of the default choices.
    preset_colors: Vec<(PlayerId, [Color; 3])>,
    /// An identity for the pack in front of each player (booster packs move as they're
    /// passed), and the next one to give.
    pack_ids: Vec<usize>,
    next_pack_id: usize,
    /// Guesses of the next card drafted from a pack: (guesser, the card's name, the pack,
    /// the guessed name if any) (Spire Phantasm).
    guesses: Vec<(PlayerId, SmolStr, usize, Option<SmolStr>)>,
    /// Players who may look at the next card drafted from a pack (Cogwork Spy), and the
    /// cards they looked at, visible to them until the packs are passed.
    lookers: Vec<(PlayerId, usize)>,
    looked: Vec<(PlayerId, Arc<CardDef>)>,
    /// The packs waiting in front of each player after the one they're drafting from, in
    /// the order they'll draft from them.
    queued: Vec<std::collections::VecDeque<QueuedPack>>,
    /// Players who may add a booster pack to the draft, having just drafted a card that
    /// lets them (Lore Seeker).
    may_add: Vec<PlayerId>,
    /// (passer, recipient): to whom each player passes the last card of a booster pack
    /// when several players drafted a card named Canal Dredger (otherwise the first such
    /// player in the passing direction).
    last_card_to: Vec<(PlayerId, PlayerId)>,
}

/// Whether the card has a draft ability whose name is `name` or starts with `name:`.
fn has_draft_ability_kind(card: &CardDef, name: &str) -> bool {
    card.front().chars.abilities.iter().any(|a| {
        matches!(&a.kind, crate::ability::AbilityKind::Static(s)
            if matches!(&s.effect, crate::ability::StaticEffect::Custom(n)
                if n.as_str() == name || n.strip_prefix(name).is_some_and(|r| r.starts_with(':'))))
    })
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
            next_name: vec![],
            skip_next: vec![],
            preset_colors: vec![],
            pack_ids: vec![0; n],
            next_pack_id: 0,
            guesses: vec![],
            lookers: vec![],
            looked: vec![],
            queued: vec![Default::default(); n],
            may_add: vec![],
            last_card_to: vec![],
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
            self.next_pack_id += 1;
            self.pack_ids[p] = self.next_pack_id;
            self.passer[p] = None;
            self.drafted_from_pack[p] = false;
            self.this_round[p] = 0;
        }
        true
    }

    /// Whether every card has been drafted.
    pub fn is_complete(&self) -> bool {
        self.packs.iter().all(|p| p.is_empty())
            && self.queued.iter().all(|q| q.is_empty())
            && self.boosters.iter().all(|b| b.is_empty())
    }

    /// Whether `p` may add a booster pack to the draft now: right after drafting a card
    /// that lets them ("After you draft [this card], you may add a booster pack to the
    /// draft.", Lore Seeker), until their next pick.
    pub fn can_add_booster(&self, p: PlayerId) -> bool {
        self.may_add.contains(&p)
    }

    /// `p` adds the booster pack `pack` to the draft (Lore Seeker; CR 905.2). The pack the
    /// card was drafted from is passed as usual; `p` opens the new pack and drafts from it
    /// next, passes it in the same direction, and then drafts from the pack they would
    /// have drafted from otherwise (the card's rulings): every later pack reaches them one
    /// pick later this round.
    pub fn add_booster(
        &mut self,
        p: PlayerId,
        pack: Vec<Arc<CardDef>>,
    ) -> Result<(), DraftError> {
        let Some(k) = self.may_add.iter().position(|q| *q == p) else {
            return Err(DraftError::InvalidOption);
        };
        if pack.is_empty() {
            return Err(DraftError::InvalidPick);
        }
        self.may_add.remove(k);
        self.next_pack_id += 1;
        let new = QueuedPack {
            cards: pack,
            id: self.next_pack_id,
            passer: None,
        };
        let i = p.idx();
        if self.drafted_from_pack[i] {
            // The packs haven't been passed yet: the new pack comes before the one passed
            // to them.
            self.queued[i].push_front(new);
        } else {
            // They were passed (the others had drafted): the pack in front of them waits.
            let waiting = QueuedPack {
                cards: std::mem::replace(&mut self.packs[i], new.cards),
                id: std::mem::replace(&mut self.pack_ids[i], new.id),
                passer: std::mem::replace(&mut self.passer[i], None),
            };
            if !waiting.cards.is_empty() {
                self.queued[i].push_front(waiting);
            }
        }
        Ok(())
    }

    /// When several players drafted a card named Canal Dredger, `from` passes the last
    /// card of each booster pack to `to` (one of them; the card's rulings). Without a
    /// choice, they pass it to the first such player in the passing direction.
    pub fn choose_last_card_recipient(&mut self, from: PlayerId, to: PlayerId) {
        self.last_card_to.retain(|(f, _)| *f != from);
        self.last_card_to.push((from, to));
    }

    /// The players who drafted a card that takes the last cards of booster packs
    /// (Canal Dredger).
    fn last_card_drafters(&self) -> Vec<usize> {
        (0..self.players())
            .filter(|q| {
                self.drafted[*q]
                    .iter()
                    .any(|d| has_draft_ability(&d.card, DRAFT_LAST_CARD_TO_DRAFTER))
            })
            .collect()
    }

    /// Who `from` passes a booster pack's last card to (Canal Dredger): themselves if they
    /// drafted one, else their chosen drafter, else the first in the passing direction.
    fn last_card_recipient(&self, from: usize, drafters: &[usize], dir: PassDirection) -> usize {
        if drafters.contains(&from) {
            return from;
        }
        if let Some((_, to)) = self
            .last_card_to
            .iter()
            .find(|(f, t)| f.idx() == from && drafters.contains(&t.idx()))
        {
            return to.idx();
        }
        let n = self.players();
        (1..n)
            .map(|k| match dir {
                PassDirection::Left => (from + k) % n,
                PassDirection::Right => (from + n * k - k) % n,
            })
            .find(|q| drafters.contains(q))
            .unwrap_or((from + 1) % n)
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
    /// the cards they've drafted, cards currently revealed as they're drafted
    /// (CR 905.2b), and cards other players drafted face up (CR 905.2c). The information
    /// noted for revealed cards ([`Draft::info`]) is public at any time.
    pub fn visible(&self, p: PlayerId) -> Vec<Arc<CardDef>> {
        let mut out: Vec<Arc<CardDef>> = self.packs[p.idx()].clone();
        out.extend(self.drafted[p.idx()].iter().map(|d| d.card.clone()));
        for (q, c) in &self.revealed {
            if *q != p {
                out.push(c.clone());
            }
        }
        // "You may look at the next card drafted from this booster pack." (Cogwork Spy)
        out.extend(
            self.looked
                .iter()
                .filter(|(q, _)| *q == p)
                .map(|(_, c)| c.clone()),
        );
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
        self.pick_with(p, picks, None)
    }

    /// The index in `p`'s drafted cards pile of a face-up card named `with` that has the
    /// draft ability `ability`.
    fn face_up_with(&self, p: PlayerId, with: &str, ability: &[&str]) -> Option<usize> {
        self.drafted[p.idx()].iter().position(|d| {
            d.face_up
                && d.card.name == with
                && ability.iter().any(|a| has_draft_ability_kind(&d.card, a))
        })
    }

    /// Like [`Draft::pick`], using a drafted card's ability as the card is drafted
    /// (CR 905.2): see [`DraftOption`]. A player who must pass the next booster pack
    /// without drafting from it (Leovold's Operative) passes it with no picks.
    pub fn pick_with(
        &mut self,
        p: PlayerId,
        picks: &[usize],
        opt: Option<DraftOption>,
    ) -> Result<(), DraftError> {
        if self.is_complete() {
            return Err(DraftError::DraftComplete);
        }
        let i = p.idx();
        if self.drafted_from_pack[i] {
            return Err(DraftError::AlreadyDrafted);
        }
        // "After you draft [Lore Seeker], you may add a booster pack": until this pick.
        self.may_add.retain(|q| *q != p);
        if let Some(k) = self.skip_next.iter().position(|q| *q == p) {
            if !picks.is_empty() || opt.is_some() || self.packs[i].is_empty() {
                return Err(DraftError::InvalidPick);
            }
            self.skip_next.remove(k);
            self.finish_pick(p);
            return Ok(());
        }
        let want = self.style.picks().min(self.packs[i].len());
        let mut idx: Vec<usize> = picks.to_vec();
        idx.sort_unstable();
        idx.dedup();
        if idx.len() != want || idx.iter().any(|k| *k >= self.packs[i].len()) {
            return Err(DraftError::InvalidPick);
        }
        // Validate the option before anything is drafted.
        let ability_card = match &opt {
            None => None,
            Some(DraftOption::Remove { with }) => Some(
                self.face_up_with(p, with, &[DRAFT_REMOVE_FACE_UP, DRAFT_REMOVE_FACE_DOWN])
                    .ok_or(DraftError::InvalidOption)?,
            ),
            Some(DraftOption::RevealAndNote { with }) => {
                let k = self
                    .face_up_with(p, with, &[DRAFT_NOTE_NAME, DRAFT_NOTE_TYPES])
                    .ok_or(DraftError::InvalidOption)?;
                // "As you draft a creature card": only a creature card can be revealed
                // (Noble Banneret, Paliano Vanguard).
                let src = &self.drafted[i][k].card;
                let creature_only = has_draft_ability(src, &format!("{DRAFT_NOTE_NAME}:creature"))
                    || has_draft_ability_kind(src, DRAFT_NOTE_TYPES);
                if creature_only
                    && idx.iter().any(|c| {
                        !self.packs[i][*c]
                            .front()
                            .chars
                            .card_types
                            .contains(CardType::Creature)
                    })
                {
                    return Err(DraftError::InvalidOption);
                }
                Some(k)
            }
            Some(DraftOption::Additional { with, extra }) => {
                if *extra >= self.packs[i].len() || idx.contains(extra) {
                    return Err(DraftError::InvalidOption);
                }
                Some(
                    self.face_up_with(
                        p,
                        with,
                        &[DRAFT_ADDITIONAL_PUT_BACK, DRAFT_ADDITIONAL_SKIP_NEXT],
                    )
                    .ok_or(DraftError::InvalidOption)?,
                )
            }
        };
        // The cards drafted, one at a time: the picks, then an additional card.
        let mut order: Vec<usize> = idx.clone();
        if let Some(DraftOption::Additional { extra, .. }) = &opt {
            order.push(*extra);
        }
        let mut cards: Vec<Option<Arc<CardDef>>> =
            self.packs[i].iter().cloned().map(Some).collect();
        let drafted: Vec<Arc<CardDef>> = order.iter().filter_map(|k| cards[*k].take()).collect();
        self.packs[i] = cards.into_iter().flatten().collect();
        for card in drafted {
            self.this_round[i] += 1;
            self.draft_card(p, card.clone());
            match &opt {
                Some(DraftOption::Remove { with }) => {
                    // The card is drafted, then removed from the draft (CR 905.2): it
                    // isn't in the player's card pool.
                    let face_up = has_draft_ability(
                        &self.drafted[i][ability_card.unwrap_or(0)].card,
                        DRAFT_REMOVE_FACE_UP,
                    );
                    if let Some(k) = self.drafted[i]
                        .iter()
                        .rposition(|d| Arc::ptr_eq(&d.card, &card))
                    {
                        self.drafted[i].remove(k);
                    }
                    self.info.removed.push(RemovedCard {
                        player: p,
                        with: with.clone(),
                        card,
                        face_up,
                    });
                }
                Some(DraftOption::RevealAndNote { with }) => {
                    let k = ability_card.unwrap_or(0);
                    let src = self.drafted[i][k].card.clone();
                    let mut note = DraftNote::new(p, with.clone());
                    if has_draft_ability_kind(&src, DRAFT_NOTE_TYPES) {
                        note.noted_types = card.front().chars.subtypes.iter().cloned().collect();
                    } else {
                        note.noted_name = Some(card.name.clone());
                    }
                    self.revealed.push((p, card.clone()));
                    self.info.notes.push(note);
                    self.drafted[i][k].face_up = false;
                }
                _ => {}
            }
        }
        if let Some(DraftOption::Additional { .. }) = &opt {
            let k = ability_card.unwrap_or(0);
            if has_draft_ability(&self.drafted[i][k].card, DRAFT_ADDITIONAL_PUT_BACK) {
                // "Put [this card] into that booster pack."
                let d = self.drafted[i].remove(k);
                self.packs[i].push(d.card);
            } else {
                // "Turn [this card] face down, then pass the next booster pack without
                // drafting a card from it."
                self.drafted[i][k].face_up = false;
                self.skip_next.push(p);
            }
        }
        self.finish_pick(p);
        Ok(())
    }

    /// `p` has drafted from the pack in front of them; the packs are passed once everyone
    /// has.
    fn finish_pick(&mut self, p: PlayerId) {
        self.drafted_from_pack[p.idx()] = true;
        let waiting =
            (0..self.players()).any(|q| !self.drafted_from_pack[q] && !self.packs[q].is_empty());
        if !waiting {
            self.pass();
        }
    }

    /// `p` guesses the name of the next card drafted from the booster pack they drafted
    /// a card with the guessing ability from (Spire Phantasm); the latest such pending
    /// guess. An error if they have none.
    pub fn guess(&mut self, p: PlayerId, name: &str) -> Result<(), DraftError> {
        let g = self
            .guesses
            .iter_mut()
            .rev()
            .find(|(q, _, _, guess)| *q == p && guess.is_none())
            .ok_or(DraftError::InvalidOption)?;
        g.3 = Some(SmolStr::new(name));
        Ok(())
    }

    /// Sets the colors the next three-color note of `p` records (Paliano, the High City):
    /// the color the player to their right chooses, their own, and the color the player
    /// to their left chooses. Otherwise each chooses a color not yet chosen at random.
    pub fn preset_colors(&mut self, p: PlayerId, colors: [Color; 3]) {
        self.preset_colors.retain(|(q, _)| *q != p);
        self.preset_colors.push((p, colors));
    }

    /// Puts a drafted card in `p`'s pile, applying its draft abilities (CR 905.2).
    fn draft_card(&mut self, p: PlayerId, card: Arc<CardDef>) {
        // The next card drafted from this pack: guessed (Spire Phantasm), looked at
        // (Cogwork Spy) — unless the drafting player is the one who'd look.
        let pack = self.pack_ids[p.idx()];
        let mut i = 0;
        while i < self.guesses.len() {
            if self.guesses[i].2 == pack {
                let (guesser, with, _, guess) = self.guesses.remove(i);
                let mut note = DraftNote::new(guesser, with);
                note.guessed_correctly = Some(guess.is_some_and(|g| g == card.name));
                self.info.notes.push(note);
                // "Then that player reveals the drafted card."
                self.revealed.push((p, card.clone()));
            } else {
                i += 1;
            }
        }
        let lookers: Vec<PlayerId> = self
            .lookers
            .iter()
            .filter(|(_, k)| *k == pack)
            .map(|(q, _)| *q)
            .collect();
        self.lookers.retain(|(_, k)| *k != pack);
        for q in lookers {
            if q != p {
                self.looked.push((q, card.clone()));
            }
        }
        // "Reveal the next card you draft and note its name." (Aether Searcher, drafted
        // earlier.)
        let pending: Vec<SmolStr> = self
            .next_name
            .iter()
            .filter(|(q, _)| *q == p)
            .map(|(_, n)| n.clone())
            .collect();
        if !pending.is_empty() {
            self.next_name.retain(|(q, _)| *q != p);
            self.revealed.push((p, card.clone()));
            for with in pending {
                let mut note = DraftNote::new(p, with);
                note.noted_name = Some(card.name.clone());
                self.info.notes.push(note);
            }
        }
        // CR 905.2b: reveal it, note the information, then turn it face down.
        let count = has_draft_ability(&card, DRAFT_NOTE_COUNT);
        let passer = has_draft_ability(&card, DRAFT_NOTE_PASSER);
        if count || passer {
            self.revealed.push((p, card.clone()));
            let mut note = DraftNote::new(p, card.name.clone());
            note.number = count.then_some(self.this_round[p.idx()] as i64);
            note.noted_player = if passer { self.passer[p.idx()] } else { None };
            self.info.notes.push(note);
        }
        // Spire Phantasm, Cogwork Spy: about the next card drafted from this pack (the
        // one this card was drafted from).
        if has_draft_ability(&card, DRAFT_GUESS_NEXT) {
            self.revealed.push((p, card.clone()));
            self.guesses.push((p, card.name.clone(), pack, None));
        }
        if has_draft_ability(&card, DRAFT_LOOK_NEXT) {
            self.revealed.push((p, card.clone()));
            self.lookers.push((p, pack));
        }
        if has_draft_ability(&card, DRAFT_NOTE_NEXT_NAME) {
            self.revealed.push((p, card.clone()));
            self.next_name.push((p, card.name.clone()));
        }
        // "Reveal [this card] as you draft it. After you draft [this card], you may add a
        // booster pack to the draft." (Lore Seeker)
        if has_draft_ability(&card, DRAFT_ADD_BOOSTER) {
            self.revealed.push((p, card.clone()));
            self.may_add.push(p);
        }
        if has_draft_ability(&card, DRAFT_THREE_COLORS) {
            self.revealed.push((p, card.clone()));
            let colors = self.three_colors(p);
            let mut note = DraftNote::new(p, card.name.clone());
            note.colors = colors;
            self.info.notes.push(note);
        }
        // CR 905.2c: drafted face up.
        let face_up = has_draft_ability(&card, DRAFT_FACE_UP);
        self.drafted[p.idx()].push(DraftedCard { card, face_up });
    }

    /// "The player to your right chooses a color, you choose another color, then the
    /// player to your left chooses a third color": three different colors (Paliano, the
    /// High City's rulings).
    fn three_colors(&mut self, p: PlayerId) -> Vec<Color> {
        if let Some(k) = self.preset_colors.iter().position(|(q, _)| *q == p) {
            let (_, c) = self.preset_colors.remove(k);
            if c[0] != c[1] && c[1] != c[2] && c[0] != c[2] {
                return c.to_vec();
            }
        }
        let mut left: Vec<Color> = Color::ALL.to_vec();
        left.shuffle(&mut self.rng);
        left.truncate(3);
        left
    }

    /// Passes each pack to the next player in the round's direction.
    fn pass(&mut self) {
        let n = self.players();
        // CR 905.2b: the revealed cards were turned face down once their information was
        // noted; nobody sees them any more.
        self.revealed.clear();
        if self.packs.iter().all(|p| p.is_empty()) && self.queued.iter().all(|q| q.is_empty()) {
            self.start_round();
            return;
        }
        let dir = Draft::direction(self.round);
        self.looked.clear();
        // "Each player passes the last card from each booster pack to a player who drafted
        // a card named Canal Dredger."
        let drafters = self.last_card_drafters();
        let mut incoming: Vec<Vec<QueuedPack>> = vec![vec![]; n];
        for from in 0..n {
            let cards = std::mem::take(&mut self.packs[from]);
            if cards.is_empty() {
                continue;
            }
            let to = if cards.len() == 1 && !drafters.is_empty() {
                self.last_card_recipient(from, &drafters, dir)
            } else {
                match dir {
                    PassDirection::Left => (from + 1) % n,
                    PassDirection::Right => (from + n - 1) % n,
                }
            };
            incoming[to].push(QueuedPack {
                cards,
                id: self.pack_ids[from],
                passer: Some(PlayerId(from as u8)),
            });
        }
        // Each player drafts next from the first pack waiting in front of them.
        for (p, packs) in incoming.into_iter().enumerate() {
            self.queued[p].extend(packs);
            match self.queued[p].pop_front() {
                Some(q) => {
                    self.packs[p] = q.cards;
                    self.pack_ids[p] = q.id;
                    self.passer[p] = q.passer;
                }
                None => self.passer[p] = None,
            }
        }
        self.drafted_from_pack = vec![false; n];
        if self.packs.iter().all(|p| p.is_empty()) {
            self.start_round();
        }
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
    // "the number of cards you removed from the draft with cards named [this card's name]"
    // (Cogwork Grinder's rulings: the controller's, not the owner's).
    if name == REMOVED_COUNT {
        let src = source_name(g, ctx)?;
        return Some(
            g.start
                .draft
                .removed
                .iter()
                .filter(|r| r.player == ctx.controller && r.with == src)
                .count() as i64,
        );
    }
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

/// The name of the card whose ability is evaluated (the source of the ability, or the
/// spell itself).
fn source_name(g: &Game, ctx: &Ctx) -> Option<SmolStr> {
    ctx.source
        .and_then(|s| g.try_obj(g.ability_source_of(s)))
        .map(|o| o.chars.name.clone())
}

/// The notes `p` made for cards named `name` (CR 905.2b).
fn notes_for<'a>(g: &'a Game, p: PlayerId, name: &'a str) -> impl Iterator<Item = &'a DraftNote> {
    g.start
        .draft
        .notes
        .iter()
        .filter(move |n| n.player == p && n.card == name)
}

/// Filters referring to information noted during the draft (CR 905.2b), for the filter's
/// player (`ctx.controller`) and cards with the source's name: [`NOTED_NAME`],
/// [`NOTED_TYPE`], [`CHOSEN_COLOR`].
pub fn custom_filter(g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
    if !matches!(name, NOTED_NAME | NOTED_TYPE | CHOSEN_COLOR) {
        return None;
    }
    let Some(src) = source_name(g, ctx) else {
        return Some(false);
    };
    let o = g.obj(id);
    let mut notes = notes_for(g, ctx.controller, &src);
    Some(match name {
        NOTED_NAME => notes.any(|n| n.noted_name.as_ref().is_some_and(|x| o.chars.has_name(x))),
        NOTED_TYPE => notes.any(|n| n.noted_types.iter().any(|t| o.chars.subtypes.contains(t))),
        _ => notes.any(|n| n.colors.iter().any(|c| o.chars.colors.contains(*c))),
    })
}

/// Conditions about cards removed from the draft: "you removed a creature card with
/// flying from the draft with cards named [this card's name]" ([`REMOVED_CREATURE_WITH`]).
pub fn custom_condition(g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
    if name == GUESSED_CORRECTLY {
        let src = source_name(g, ctx)?;
        return Some(notes_for(g, ctx.controller, &src).any(|n| n.guessed_correctly == Some(true)));
    }
    let kw = name
        .strip_prefix(REMOVED_CREATURE_WITH)?
        .strip_prefix(':')?;
    let kind = crate::keywords::KeywordKind::from_name(kw)?;
    let src = source_name(g, ctx)?;
    Some(g.start.draft.removed.iter().any(|r| {
        r.player == ctx.controller
            && r.with == src
            && r.card.faces.iter().any(|f| {
                f.chars.card_types.contains(CardType::Creature) && f.chars.has_keyword(kind)
            })
    }))
}

/// Colors chosen as `p` drafted cards with the source's name (Paliano, the High City).
pub fn chosen_colors(g: &Game, ctx: &Ctx) -> Vec<Color> {
    let Some(src) = source_name(g, ctx) else {
        return vec![];
    };
    let mut out: Vec<Color> = Vec::new();
    for n in notes_for(g, ctx.controller, &src) {
        for c in &n.colors {
            if !out.contains(c) {
                out.push(*c);
            }
        }
    }
    out
}
