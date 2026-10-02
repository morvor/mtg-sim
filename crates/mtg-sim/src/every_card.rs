//! Every-card fuzzing (`mtg-sim --every-card`): every fully supported card, in a
//! deterministic order, gets games built around it — both players' decks hold several
//! copies, on-color lands and random supported spells, with copies in the opening hand and
//! enough lands to cast it early, in the variant the card needs (a Commander game for
//! commander cards, Planechase for planes, ...). The games run with the rules checks, and
//! the engine's events tell which cards were cast or played and which of their activated
//! and triggered abilities were used.
//!
//! Usage: mtg-sim --every-card [--games-per-card K] [--from N] [--count M]
//!                [--filter TEXT] [--kind KIND] [--game G] [--threads T] [--max-turns N]
//!                [--check N] [--slow SECS] [--timeout SECS] [--report FILE] [--log]
//!                [--list]
//!
//! Card N is the Nth card of the pool (sorted by name); its games' seeds are derived from
//! N, so `--from N --count 1 --game G --log` replays game G of card N.

use crate::coverage::{tracked_abilities, Focus, Tracked, Usage, Use};
use crate::decks::DeckList;
use crate::runner::{kind_of, play, GameSpec, Outcome};
use mtg_engine::card::Layout;
use mtg_engine::types::{CardType, Supertype};
use mtg_engine::*;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use smol_str::SmolStr;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

/// A card of the pool.
pub struct PoolCard {
    pub def: Arc<CardDef>,
    /// Oracle text of every face, lowercase.
    pub text: String,
    /// Color identity, WUBRG bits.
    pub mask: u8,
    pub mana_value: u32,
    /// Legal in some format (filler cards are, to keep games ordinary).
    pub legal: bool,
    /// Lowercase words of its type lines and its keywords ("goblin", "artifact",
    /// "flying"), for finding cards that go with others.
    pub words: Vec<String>,
}

impl PoolCard {
    fn front_types(&self) -> &mtg_engine::types::CardTypeSet {
        &self.def.front().chars.card_types
    }
    fn is(&self, t: CardType) -> bool {
        self.def
            .faces
            .iter()
            .any(|f| f.chars.card_types.contains(t))
    }
    fn is_land(&self) -> bool {
        self.front_types().contains(CardType::Land)
    }
    fn is_creature(&self) -> bool {
        self.front_types().contains(CardType::Creature)
    }
    fn has_subtype(&self, s: &str) -> bool {
        self.def.faces.iter().any(|f| f.chars.has_subtype(s))
    }
    fn is_legendary_creature(&self) -> bool {
        let c = &self.def.front().chars;
        c.supertypes.contains(Supertype::Legendary) && c.card_types.contains(CardType::Creature)
    }
    /// An ordinary spell for filler: a normal layout, legal somewhere, no supplementary
    /// or variant card types.
    fn is_plain_spell(&self) -> bool {
        self.legal
            && self.def.layout == Layout::Normal
            && !self.is_land()
            && !mtg_engine::variants::is_nontraditional(&self.def)
            && !self.is(CardType::Conspiracy)
            && !self.text.contains("commander")
            && !self.text.contains("ante")
    }
}

/// Every fully supported card: real (paper) playable cards, one per name, sorted by
/// name. Compiled on `threads` threads.
pub fn pool(threads: usize) -> Vec<PoolCard> {
    let mut names: Vec<(String, bool, f64, Vec<String>)> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for c in mtg_data::cards().iter() {
        if !c.is_playable_card() {
            continue;
        }
        if !c.games.is_empty() && c.games.iter().all(|g| g != "paper") {
            continue;
        }
        if seen.insert(c.name.to_lowercase()) {
            let mut words: Vec<String> = c.keywords.iter().map(|k| k.to_lowercase()).collect();
            for f in c.faces() {
                let tl = f
                    .type_line
                    .clone()
                    .or_else(|| c.type_line.clone())
                    .unwrap_or_default();
                words.extend(
                    tl.split(|ch: char| !ch.is_alphanumeric() && ch != '\'')
                        .filter(|w| !w.is_empty())
                        .map(str::to_lowercase),
                );
            }
            words.sort();
            words.dedup();
            names.push((
                c.name.clone(),
                c.is_legal_somewhere(),
                c.cmc.unwrap_or(0.0),
                words,
            ));
        }
    }
    names.sort_by(|a, b| a.0.cmp(&b.0));
    let next = AtomicUsize::new(0);
    let out: Mutex<Vec<(usize, PoolCard)>> = Mutex::new(Vec::new());
    std::thread::scope(|s| {
        for _ in 0..threads.max(1) {
            s.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                let Some((name, legal, cmc, words)) = names.get(i) else {
                    break;
                };
                let Some(def) = CardDb::global().get(name) else {
                    continue;
                };
                if !def.is_fully_supported() {
                    continue;
                }
                let text = def
                    .faces
                    .iter()
                    .map(|f| f.chars.rules_text.to_lowercase())
                    .collect::<Vec<_>>()
                    .join("\n");
                let mask = def.color_identity.0;
                out.lock().unwrap().push((
                    i,
                    PoolCard {
                        def,
                        text,
                        mask,
                        mana_value: cmc.max(0.0).ceil() as u32,
                        legal: *legal,
                        words: words.clone(),
                    },
                ));
            });
        }
    });
    let mut v = out.into_inner().unwrap();
    v.sort_by_key(|(i, _)| *i);
    v.into_iter().map(|(_, c)| c).collect()
}

const BASICS: [&str; 5] = ["Plains", "Island", "Swamp", "Mountain", "Forest"];
const SNOW_BASICS: [&str; 5] = [
    "Snow-Covered Plains",
    "Snow-Covered Island",
    "Snow-Covered Swamp",
    "Snow-Covered Mountain",
    "Snow-Covered Forest",
];

/// The kind of game a card needs: "standard", or the variant or supplementary deck its
/// card type or text calls for ("planechase", "archenemy", "vanguard", "attractions",
/// "dungeon", "conspiracy", "commander").
pub fn game_kind(x: &PoolCard) -> &'static str {
    if x.is(CardType::Plane) || x.is(CardType::Phenomenon) {
        "planechase"
    } else if x.is(CardType::Scheme) {
        "archenemy"
    } else if x.is(CardType::Vanguard) {
        "vanguard"
    } else if x.has_subtype("Attraction") && x.is(CardType::Artifact) {
        "attractions"
    } else if x.is(CardType::Dungeon) {
        "dungeon"
    } else if x.is(CardType::Conspiracy) {
        "conspiracy"
    } else if x.text.contains("commander") || x.text.contains("command zone") {
        "commander"
    } else {
        "standard"
    }
}

/// What the games of a card are built from.
pub struct Setup {
    pub config: GameConfig,
    pub decks: Vec<DeckList>,
    pub commanders: Vec<Vec<SmolStr>>,
    /// What kind of game: "standard", "commander", "planechase", ...
    pub kind: &'static str,
}

/// Indexes of the pool used to pick filler and companions for the games.
pub struct Picker<'a> {
    pool: &'a [PoolCard],
    /// Cheap creatures and other spells for filler.
    creatures: Vec<usize>,
    spells: Vec<usize>,
    planes: Vec<usize>,
    schemes: Vec<usize>,
    attractions: Vec<usize>,
    lessons: Vec<usize>,
    /// Cards that venture into the dungeon or take the initiative.
    venturers: Vec<usize>,
    /// Cards that open Attractions.
    openers: Vec<usize>,
    /// Legendary creatures that can be commanders, and those that can choose a
    /// Background.
    commanders: Vec<usize>,
    background_choosers: Vec<usize>,
    /// Plain spells by the words of their type lines and keywords.
    by_word: HashMap<String, Vec<usize>>,
    /// Plain spells whose text has each [`PHRASES`] filler phrase.
    by_phrase: Vec<Vec<usize>>,
}

/// Words of card text that don't call for particular cards (every deck has them).
const STOP_WORDS: [&str; 8] = [
    "creature",
    "land",
    "basic",
    "card",
    "spell",
    "token",
    "enchant",
    "legendary",
];

/// Phrases of a card's text, and a phrase of the text of other cards that make them
/// matter in a game: "whenever you discard" wants discard effects, and so on.
const PHRASES: [(&str, &str); 16] = [
    ("spell that targets", "target creature gets +"),
    ("becomes the target", "target creature"),
    ("discard", "discards"),
    ("gain life", "you gain"),
    ("gains life", "you gain"),
    ("+1/+1 counter", "+1/+1 counter"),
    ("graveyard", "mill"),
    ("sacrifice", "sacrifice a"),
    ("{e}", "{e}"),
    ("poison", "toxic"),
    ("cycle", "cycling"),
    ("draw", "draw a card"),
    ("token", "create a"),
    ("tapped", "tap target"),
    ("lose life", "loses"),
    ("counter target", "counter target"),
];

impl<'a> Picker<'a> {
    pub fn new(pool: &'a [PoolCard]) -> Self {
        let mut p = Picker {
            pool,
            creatures: vec![],
            spells: vec![],
            planes: vec![],
            schemes: vec![],
            attractions: vec![],
            lessons: vec![],
            venturers: vec![],
            openers: vec![],
            commanders: vec![],
            background_choosers: vec![],
            by_word: HashMap::new(),
            by_phrase: vec![vec![]; PHRASES.len()],
        };
        for (i, c) in pool.iter().enumerate() {
            if c.is_plain_spell() && c.mana_value <= 5 {
                for w in &c.words {
                    p.by_word.entry(w.clone()).or_default().push(i);
                }
                for (k, (_, has)) in PHRASES.iter().enumerate() {
                    if c.text.contains(has) {
                        p.by_phrase[k].push(i);
                    }
                }
            }
            if c.is_plain_spell() && c.mana_value <= 4 {
                if c.is_creature() && c.mana_value <= 3 {
                    p.creatures.push(i);
                } else if !c.is_creature() {
                    p.spells.push(i);
                }
            }
            if c.is(CardType::Plane) {
                p.planes.push(i);
            }
            if c.is(CardType::Scheme) {
                p.schemes.push(i);
            }
            if c.has_subtype("Attraction") && c.is(CardType::Artifact) {
                p.attractions.push(i);
            }
            if c.legal && c.has_subtype("Lesson") {
                p.lessons.push(i);
            }
            if c.legal
                && c.mana_value <= 5
                && (c.text.contains("venture into the dungeon")
                    || c.text.contains("take the initiative"))
                && !c.is_land()
            {
                p.venturers.push(i);
            }
            if c.legal && c.mana_value <= 5 && c.text.contains("open an attraction") {
                p.openers.push(i);
            }
            if c.legal && c.is_legendary_creature() && c.def.is_legal_in("commander") {
                p.commanders.push(i);
                if c.text.contains("choose a background") {
                    p.background_choosers.push(i);
                }
            }
        }
        p
    }

    fn def(&self, i: usize) -> Arc<CardDef> {
        self.pool[i].def.clone()
    }

    /// Cards of `from` whose color identity fits in `mask`.
    fn fitting(&self, from: &[usize], mask: u8) -> Vec<usize> {
        from.iter()
            .copied()
            .filter(|&i| self.pool[i].mask & !mask == 0)
            .collect()
    }

    /// `n` filler cards in `mask`: four copies each of random cheap creatures (about
    /// half) and other spells.
    fn filler(&self, mask: u8, n: usize, rng: &mut StdRng) -> Vec<Arc<CardDef>> {
        let creatures = self.fitting(&self.creatures, mask);
        let spells = self.fitting(&self.spells, mask);
        let mut out = Vec::new();
        let mut used: HashSet<usize> = HashSet::new();
        let mut tries = 0;
        while out.len() < n && tries < 1000 {
            tries += 1;
            let from = if out.len() < n / 2 || spells.is_empty() {
                &creatures
            } else {
                &spells
            };
            let Some(&i) = from.choose(rng) else {
                break;
            };
            if !used.insert(i) {
                continue;
            }
            for _ in 0..4.min(n - out.len()) {
                out.push(self.def(i));
            }
        }
        out
    }

    /// Cards that make the card's abilities matter: four copies each of up to four
    /// cards with types or keywords its text names ("Goblin", "artifact", "flying"),
    /// or that do what its text cares about ("whenever you discard ...").
    fn synergy(&self, x: &PoolCard, mask: u8, rng: &mut StdRng) -> Vec<Arc<CardDef>> {
        let mut picks: Vec<Vec<usize>> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        for raw in x.text.split(|c: char| !c.is_alphanumeric() && c != '\'') {
            let w = raw.to_lowercase();
            let singular = match w.as_str() {
                "elves" => "elf".to_string(),
                "dwarves" => "dwarf".to_string(),
                "wolves" => "wolf".to_string(),
                "sorceries" => "sorcery".to_string(),
                _ => w.strip_suffix('s').unwrap_or(&w).to_string(),
            };
            for cand in [w.clone(), singular] {
                if cand.len() < 3
                    || STOP_WORDS.contains(&cand.as_str())
                    || !seen.insert(cand.clone())
                {
                    continue;
                }
                if let Some(v) = self.by_word.get(&cand) {
                    let fit = self.fitting(v, mask);
                    if !fit.is_empty() {
                        picks.push(fit);
                    }
                }
            }
        }
        for (k, (wants, _)) in PHRASES.iter().enumerate() {
            if x.text.contains(wants) {
                let fit = self.fitting(&self.by_phrase[k], mask);
                if !fit.is_empty() {
                    picks.push(fit);
                }
            }
        }
        picks.shuffle(rng);
        let mut out = Vec::new();
        let mut used: HashSet<usize> = HashSet::new();
        for from in picks.iter().take(4) {
            if let Some(&i) = from
                .iter()
                .filter(|i| !used.contains(i))
                .collect::<Vec<_>>()
                .choose(rng)
                .copied()
            {
                used.insert(i);
                for _ in 0..4 {
                    out.push(self.def(i));
                }
            }
        }
        out
    }

    /// Four copies each of up to `n` cards of `from` that let the card be used (cards
    /// that open Attractions, say), whatever their colors: the deck's colors grow to
    /// include theirs.
    fn enablers(
        &self,
        from: &[usize],
        n: usize,
        mask: &mut u8,
        rng: &mut StdRng,
    ) -> Vec<Arc<CardDef>> {
        let mut v = from.to_vec();
        v.shuffle(rng);
        // Those in the deck's colors first.
        v.sort_by_key(|&i| self.pool[i].mask & !*mask != 0);
        let mut out = Vec::new();
        for &i in v.iter().take(n) {
            *mask |= self.pool[i].mask;
            out.extend(std::iter::repeat_n(self.def(i), 4));
        }
        out
    }

    /// Up to `n` distinct cards of `from` other than `not`.
    fn some(&self, from: &[usize], not: &str, n: usize, rng: &mut StdRng) -> Vec<Arc<CardDef>> {
        let mut v: Vec<usize> = from
            .iter()
            .copied()
            .filter(|&i| *self.pool[i].def.name != *not)
            .collect();
        v.shuffle(rng);
        v.truncate(n);
        v.into_iter().map(|i| self.def(i)).collect()
    }

    /// The games' setup for pool card `idx`.
    pub fn setup(&self, idx: usize, rng: &mut StdRng) -> Setup {
        let x = &self.pool[idx];
        let name = x.def.name.clone();
        let mut config = GameConfig::default();
        let kind = game_kind(x);
        // Deck colors: the card's color identity, or a random color for a colorless card.
        let mut mask = x.mask;
        if mask == 0 {
            mask = 1 << rng.gen_range(0..5);
        }
        let snow = x.text.contains("{s}") || x.text.contains("snow");
        let basics = if snow { SNOW_BASICS } else { BASICS };
        let wastes = x.text.contains("{c}")
            || x.def.faces.iter().any(|f| {
                f.chars
                    .mana_cost
                    .as_ref()
                    .is_some_and(|m| format!("{m:?}").contains("Colorless"))
            });
        let mut commanders: Vec<Vec<SmolStr>> = vec![vec![], vec![]];
        // Copies of the card in each deck (none for a supplementary-deck card or a
        // commander), and cards for the players' supplementary decks.
        let mut copies = 8;
        let mut extra: Vec<Arc<CardDef>> = Vec::new();
        let mut extra_filler: Vec<Arc<CardDef>> = Vec::new();
        let mut sideboard: Vec<Arc<CardDef>> = Vec::new();
        if kind == "planechase" {
            config = GameConfig::planechase_game();
            copies = 0;
            extra.push(x.def.clone());
            extra.extend(self.some(&self.planes, &name, 9, rng));
        } else if kind == "archenemy" {
            config = GameConfig::supervillain_rumble();
            copies = 0;
            extra.push(x.def.clone());
            extra.push(x.def.clone());
            for d in self.some(&self.schemes, &name, 9, rng) {
                extra.push(d.clone());
                extra.push(d);
            }
        } else if kind == "vanguard" {
            config = GameConfig::vanguard_game();
            copies = 0;
            extra.push(x.def.clone());
        } else if kind == "attractions" {
            copies = 0;
            extra.push(x.def.clone());
            extra.extend(self.some(&self.attractions, &name, 9, rng));
            extra_filler.extend(self.enablers(&self.openers, 3, &mut mask, rng));
        } else if kind == "dungeon" {
            copies = 0;
            extra.push(x.def.clone());
            extra_filler.extend(self.enablers(&self.venturers, 3, &mut mask, rng));
        } else if kind == "conspiracy" {
            copies = 1;
        } else if kind == "commander" {
            config = GameConfig::commander_game();
            if x.has_subtype("Background") {
                let choosers = self.fitting(&self.background_choosers, 0b11111);
                if let Some(&c) = choosers.choose(rng) {
                    mask |= self.pool[c].mask;
                    let n = self.pool[c].def.name.clone();
                    extra_filler.push(self.def(c));
                    extra_filler.push(x.def.clone());
                    commanders = vec![vec![n.clone(), name.clone()], vec![n, name.clone()]];
                    copies = 0;
                }
            } else if x.is_legendary_creature() || x.text.contains("can be your commander") {
                extra_filler.push(x.def.clone());
                commanders = vec![vec![name.clone()], vec![name.clone()]];
                copies = 0;
            } else {
                let cands: Vec<usize> = self
                    .commanders
                    .iter()
                    .copied()
                    .filter(|&i| self.pool[i].mask & x.mask == x.mask && self.pool[i].mask != 0)
                    .collect();
                if let Some(&c) = cands.choose(rng) {
                    mask = self.pool[c].mask;
                    extra_filler.push(self.def(c));
                    let n = self.pool[c].def.name.clone();
                    commanders = vec![vec![n.clone()], vec![n]];
                }
            }
        }
        extra_filler.extend(self.synergy(x, mask, rng));
        let colors: Vec<usize> = (0..5).filter(|c| mask & (1 << c) != 0).collect();
        let land = |i: usize| -> SmolStr {
            if wastes && i % 3 == 2 {
                SmolStr::new("Wastes")
            } else {
                SmolStr::new(basics[colors[i % colors.len()]])
            }
        };
        if x.text.contains(" ante") || x.text.starts_with("ante") {
            config.ante = true;
        }
        if x.text.contains("learn") {
            sideboard.extend(self.some(&self.lessons, "", 3, rng));
        }
        if x.text.contains("outside the game") || x.text.contains("sideboard") {
            sideboard.extend(self.filler(mask, 4, rng));
        }
        if x.is(CardType::Conspiracy) {
            sideboard.push(x.def.clone());
        }
        let n_lands = if x.is_land() {
            16
        } else if x.mana_value >= 5 {
            24
        } else {
            22
        };
        let mv = if x.is_land() { 0 } else { x.mana_value };
        let copies = if x.is(CardType::Conspiracy) {
            0
        } else {
            copies
        };
        let mut decks = Vec::new();
        let mut tops: Vec<Vec<SmolStr>> = Vec::new();
        for _ in 0..2 {
            let mut main: Vec<Arc<CardDef>> = Vec::new();
            for _ in 0..copies {
                main.push(x.def.clone());
            }
            let db = CardDb::global();
            for i in 0..n_lands {
                main.push(db.get(&land(i)).expect("basic land"));
            }
            main.extend(extra_filler.iter().cloned());
            let rest = 60usize.saturating_sub(main.len());
            let filler = self.filler(mask, rest, rng);
            main.extend(filler.iter().cloned());
            // The opening hand: two copies and enough lands; then draws that bring the
            // lands to its mana value, with another copy every few draws.
            let mut top: Vec<SmolStr> = Vec::new();
            let in_hand = copies.min(2);
            for _ in 0..in_hand {
                top.push(name.clone());
            }
            let mut lands = 0;
            let hand_lands = (mv as usize).clamp(2, 4).min(7 - in_hand);
            while lands < hand_lands {
                top.push(land(lands));
                lands += 1;
            }
            let mut copies_left = copies.saturating_sub(in_hand);
            let mut names: Vec<SmolStr> = filler
                .iter()
                .chain(extra_filler.iter())
                .map(|d| d.name.clone())
                .filter(|n| *n != name)
                .collect();
            names.shuffle(rng);
            let mut filler_names = names.into_iter();
            while top.len() < 7 {
                match filler_names.next() {
                    Some(n) => top.push(n),
                    None => break,
                }
            }
            for d in 0..10 {
                if lands < (mv as usize + 1).max(5) && lands < n_lands {
                    top.push(land(lands));
                    lands += 1;
                } else if d % 3 == 2 && copies_left > 0 {
                    top.push(name.clone());
                    copies_left -= 1;
                } else if let Some(n) = filler_names.next() {
                    top.push(n);
                }
            }
            // The supplementary deck's top card is the card.
            if !extra.is_empty() {
                top.push(name.clone());
            }
            main.extend(extra.iter().cloned());
            decks.push(DeckList {
                main,
                sideboard: sideboard.clone(),
            });
            tops.push(top);
        }
        config.top_of_library = tops;
        Setup {
            config,
            decks,
            commanders,
            kind,
        }
    }
}

/// The result of one card's games.
struct CardResult {
    index: usize,
    games: u32,
    usage: Usage,
    /// Failures: (game, kind of failure, message, reproduce command).
    failures: Vec<(u32, &'static str, String)>,
    hangs: u32,
    slow: u32,
    kind: &'static str,
}

struct Options {
    games_per_card: u32,
    from: usize,
    count: Option<usize>,
    filter: Option<String>,
    kind: Option<String>,
    game: Option<u32>,
    threads: usize,
    max_turns: u32,
    max_actions: u64,
    check: u32,
    slow: u64,
    timeout: u64,
    report: Option<String>,
    log: bool,
    list: bool,
}

fn parse(args: &[String]) -> Options {
    let mut o = Options {
        games_per_card: 2,
        from: 0,
        count: None,
        filter: None,
        kind: None,
        game: None,
        threads: std::thread::available_parallelism().map_or(1, |n| n.get()),
        max_turns: 30,
        max_actions: 30_000,
        check: 4,
        slow: 60,
        timeout: 60,
        report: None,
        log: false,
        list: false,
    };
    let mut i = 0;
    let value = |i: usize| -> String {
        args.get(i + 1)
            .cloned()
            .unwrap_or_else(|| panic!("{} needs a value", args[i]))
    };
    while i < args.len() {
        let mut step = 2;
        match args[i].as_str() {
            "--every-card" => step = 1,
            "--log" => {
                o.log = true;
                step = 1;
            }
            "--list" => {
                o.list = true;
                step = 1;
            }
            "--games-per-card" => o.games_per_card = value(i).parse().expect("--games-per-card K"),
            "--from" => o.from = value(i).parse().expect("--from N"),
            "--count" => o.count = Some(value(i).parse().expect("--count M")),
            "--filter" => o.filter = Some(value(i).to_lowercase()),
            "--kind" => o.kind = Some(value(i)),
            "--game" => o.game = Some(value(i).parse().expect("--game G")),
            "--threads" => o.threads = value(i).parse().expect("--threads T"),
            "--max-turns" => o.max_turns = value(i).parse().expect("--max-turns N"),
            "--max-actions" => o.max_actions = value(i).parse().expect("--max-actions N"),
            "--check" => o.check = value(i).parse().expect("--check N"),
            "--slow" => o.slow = value(i).parse().expect("--slow SECS"),
            "--timeout" => o.timeout = value(i).parse().expect("--timeout SECS"),
            "--report" => o.report = Some(value(i)),
            other => panic!("unknown argument {other} (with --every-card)"),
        }
        i += step;
    }
    o
}

/// The seed of game `game` of pool card `index`.
fn seed_of(index: usize, game: u32) -> u64 {
    (index as u64)
        .wrapping_mul(0x9e37_79b9_7f4a_7c15)
        .wrapping_add(game as u64 * 1_000_003)
        ^ 0xeca7_d5ee_d000_0000
}

fn repro(index: usize, game: u32, o: &Options) -> String {
    format!(
        "mtg-sim --every-card --from {index} --count 1 --game {game} --max-turns {} --log",
        o.max_turns
    )
}

/// Plays the games of pool card `index`.
fn run_card(picker: &Picker, index: usize, o: &Options, out: &Mutex<()>) -> CardResult {
    let x = &picker.pool[index];
    let focus = Arc::new(Focus::new(&x.def));
    let mut result = CardResult {
        index,
        games: 0,
        usage: Usage::default(),
        failures: vec![],
        hangs: 0,
        slow: 0,
        kind: "standard",
    };
    let games: Vec<u32> = match o.game {
        Some(g) => vec![g],
        None => (0..o.games_per_card).collect(),
    };
    for gi in games {
        let seed = seed_of(index, gi);
        let mut rng = StdRng::seed_from_u64(seed);
        let setup = picker.setup(index, &mut rng);
        result.kind = setup.kind;
        let usage: Arc<Mutex<Usage>> = Arc::default();
        let decks = setup.decks.clone();
        let spec = GameSpec {
            config: GameConfig {
                seed,
                max_turns: o.max_turns,
                max_actions: o.max_actions,
                ..setup.config
            },
            decks: setup.decks,
            agent_seed: seed.wrapping_mul(7),
            logging: true,
            check: o.check,
            commanders: setup.commanders,
            focus: Some(focus.clone()),
            usage: Some(usage.clone()),
        };
        let r = play(
            spec,
            Duration::from_secs(o.slow),
            Duration::from_secs(o.timeout),
        );
        result.games += 1;
        result
            .usage
            .merge(&usage.lock().unwrap_or_else(|e| e.into_inner()));
        let cmd = repro(index, gi, o);
        let say = |head: String, log: &[String], tail: usize| {
            let _l = out.lock().unwrap_or_else(|e| e.into_inner());
            println!("{head}");
            println!("  card {index}: {} ({})", x.def.name, setup.kind);
            for (p, d) in decks.iter().enumerate() {
                println!("  deck {}: {}", p + 1, d.summary());
            }
            println!("  reproduce: {cmd}");
            for l in &log[log.len().saturating_sub(tail)..] {
                println!("    {l}");
            }
        };
        let Some((outcome, violations)) = r else {
            say(
                format!(
                    "HANG: {} game {gi}: no decision for {}s",
                    x.def.name, o.timeout
                ),
                &[],
                0,
            );
            result.hangs += 1;
            result
                .failures
                .push((gi, "hang", format!("no decision for {}s", o.timeout)));
            continue;
        };
        let tail = if o.log { usize::MAX } else { 12 };
        if let Some(first) = violations.first() {
            say(
                format!(
                    "RULES: {} game {gi}: {} violation(s); first on turn {}: {}",
                    x.def.name,
                    violations.len(),
                    first.turn,
                    first.what
                ),
                &[],
                0,
            );
            result.failures.push((gi, "rules", first.what.clone()));
        }
        match outcome {
            Outcome::Finished { log, .. } => {
                if o.log {
                    let _l = out.lock().unwrap_or_else(|e| e.into_inner());
                    for l in log {
                        println!("{l}");
                    }
                }
            }
            Outcome::Slow { turn, log } => {
                say(
                    format!(
                        "SLOW: {} game {gi} stopped after {}s, on turn {turn}",
                        x.def.name, o.slow
                    ),
                    &log,
                    if o.log { usize::MAX } else { 8 },
                );
                result.slow += 1;
                result
                    .failures
                    .push((gi, "slow", format!("stopped on turn {turn}")));
            }
            Outcome::Panicked { message, turn, log } => {
                say(
                    format!("PANIC: {} game {gi}, turn {turn}: {message}", x.def.name),
                    &log,
                    tail,
                );
                result.failures.push((gi, "panic", message));
            }
        }
    }
    result
}

/// Why a card wasn't fully used, for the report.
fn reasons(c: &PoolCard, abilities: &[Tracked], u: &Usage) -> Vec<String> {
    let mut out = Vec::new();
    let supplementary = mtg_engine::variants::is_nontraditional(&c.def)
        || c.def
            .front()
            .chars
            .card_types
            .contains(CardType::Conspiracy);
    if !supplementary && !u.cast.contains(&c.def.name) {
        let what = if c.is_land() { "played" } else { "cast" };
        out.push(if u.offered_cast.contains(&c.def.name) {
            format!("never {what}: offered but never chosen")
        } else if c.def.front().chars.mana_cost.is_none() && !c.is_land() {
            format!("never {what}: no mana cost")
        } else {
            format!(
                "never {what}: never legal to {}",
                if c.is_land() { "play" } else { "cast" }
            )
        });
    }
    for t in abilities {
        let done = u.resolved.contains(&t.uid) || (t.mana && u.activated.contains(&t.uid));
        if done {
            continue;
        }
        let face =
            if t.face > 0 && c.def.layout != Layout::Split && c.def.layout != Layout::Adventure {
                "other face: "
            } else {
                ""
            };
        let r = match t.kind {
            Use::Activated if u.activated.contains(&t.uid) => {
                "activated ability: activated, never resolved".to_string()
            }
            Use::Activated if u.offered.contains(&t.uid) => {
                "activated ability: offered, never activated".to_string()
            }
            Use::Activated => {
                // The keyword's name, without its cost ("Cycling {2}" -> "Cycling").
                let k = t
                    .keyword
                    .as_deref()
                    .and_then(|k| k.split([' ', '{']).next())
                    .map(|k| format!(" ({k})"))
                    .unwrap_or_default();
                format!("activated ability: never activatable{k}")
            }
            Use::Triggered if u.triggered.contains(&t.uid) => {
                "triggered ability: triggered, never resolved".to_string()
            }
            Use::Triggered => format!(
                "triggered ability: never triggered ({})",
                t.trigger.as_deref().unwrap_or("?")
            ),
        };
        out.push(format!("{face}{r}\t{}", t.text.replace('\n', " / ")));
    }
    out
}

/// Runs `mtg-sim --every-card ...`.
pub fn run(args: &[String]) {
    let o = parse(args);
    crate::runner::install_panic_hook();
    let t0 = Instant::now();
    let pool = pool(o.threads);
    eprintln!(
        "pool: {} fully supported cards (compiled in {:.1?})",
        pool.len(),
        t0.elapsed()
    );
    let picker = Picker::new(&pool);
    let end = o.count.map_or(pool.len(), |n| (o.from + n).min(pool.len()));
    let selected: Vec<usize> = (o.from.min(end)..end)
        .filter(|&i| {
            o.filter.as_ref().is_none_or(|f| {
                pool[i].def.name.to_lowercase().contains(f.as_str())
                    || pool[i].text.contains(f.as_str())
            })
        })
        .filter(|&i| o.kind.as_ref().is_none_or(|k| game_kind(&pool[i]) == k))
        .collect();
    if o.list {
        let mut kinds: BTreeMap<&str, usize> = BTreeMap::new();
        let mut rng = StdRng::seed_from_u64(0);
        for &i in &selected {
            let s = picker.setup(i, &mut rng);
            *kinds.entry(s.kind).or_default() += 1;
            println!("{i}\t{}\t{}", pool[i].def.name, s.kind);
        }
        eprintln!("{kinds:?}");
        return;
    }
    let next = AtomicUsize::new(0);
    let leaked = AtomicUsize::new(0);
    let abort = AtomicBool::new(false);
    let out = Mutex::new(());
    let (tx, rx) = mpsc::channel::<CardResult>();
    let started = Instant::now();
    let mut results: Vec<CardResult> = Vec::new();
    std::thread::scope(|s| {
        for _ in 0..o.threads.max(1) {
            let tx = tx.clone();
            let (next, leaked, abort, out, picker, selected, o) =
                (&next, &leaked, &abort, &out, &picker, &selected, &o);
            s.spawn(move || loop {
                if abort.load(Ordering::Relaxed) {
                    break;
                }
                let k = next.fetch_add(1, Ordering::Relaxed);
                let Some(&i) = selected.get(k) else {
                    break;
                };
                let r = run_card(picker, i, o, out);
                if r.hangs > 0 {
                    // A hung game's thread keeps running: stop before they take every CPU.
                    let n =
                        leaked.fetch_add(r.hangs as usize, Ordering::Relaxed) + r.hangs as usize;
                    if n >= o.threads.max(2) {
                        abort.store(true, Ordering::Relaxed);
                    }
                }
                let _ = tx.send(r);
            });
        }
        drop(tx);
        let mut done = 0usize;
        for r in rx {
            done += 1;
            if done % 200 == 0 {
                let _l = out.lock().unwrap_or_else(|e| e.into_inner());
                eprintln!(
                    "  {done}/{} cards in {:.0?}",
                    selected.len(),
                    started.elapsed()
                );
            }
            results.push(r);
        }
    });
    results.sort_by_key(|r| r.index);
    summarize(
        &pool,
        &results,
        &o,
        started.elapsed(),
        abort.load(Ordering::Relaxed),
    );
}

fn summarize(pool: &[PoolCard], results: &[CardResult], o: &Options, el: Duration, aborted: bool) {
    let mut usage = Usage::default();
    for r in results {
        usage.merge(&r.usage);
    }
    let games: u32 = results.iter().map(|r| r.games).sum();
    let (mut castable, mut cast, mut n_abilities, mut used_abilities, mut exercised) =
        (0, 0, 0, 0, 0);
    let mut by_reason: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    let mut by_kind: BTreeMap<&str, usize> = BTreeMap::new();
    let mut rows: Vec<String> = Vec::new();
    for r in results {
        let c = &pool[r.index];
        *by_kind.entry(r.kind).or_default() += 1;
        let abilities = tracked_abilities(&c.def);
        let why = reasons(c, &abilities, &usage);
        let supplementary = mtg_engine::variants::is_nontraditional(&c.def)
            || c.def
                .front()
                .chars
                .card_types
                .contains(CardType::Conspiracy);
        if !supplementary {
            castable += 1;
            if usage.cast.contains(&c.def.name) {
                cast += 1;
            }
        }
        n_abilities += abilities.len();
        used_abilities += abilities
            .iter()
            .filter(|t| {
                usage.resolved.contains(&t.uid) || (t.mana && usage.activated.contains(&t.uid))
            })
            .count();
        if why.is_empty() {
            exercised += 1;
        }
        for w in &why {
            let reason = w.split('\t').next().unwrap_or("");
            by_reason
                .entry(reason.to_string())
                .or_default()
                .push(&c.def.name);
            rows.push(format!("{}\t{}\t{}", r.index, c.def.name, w));
        }
    }
    let pct = |a: usize, b: usize| 100.0 * a as f64 / b.max(1) as f64;
    println!(
        "every card: {} cards, {games} games in {:.1?} ({:.1} games/s){}",
        results.len(),
        el,
        games as f64 / el.as_secs_f64().max(0.001),
        if aborted {
            " (stopped early: too many hung games)"
        } else {
            ""
        }
    );
    println!("  game kinds: {by_kind:?}");
    println!(
        "  cast or played: {cast}/{castable} ({:.1}%)",
        pct(cast, castable)
    );
    println!(
        "  activated and triggered abilities used: {used_abilities}/{n_abilities} ({:.1}%)",
        pct(used_abilities, n_abilities)
    );
    println!(
        "  cards fully exercised: {exercised}/{} ({:.1}%)",
        results.len(),
        pct(exercised, results.len())
    );
    let mut reasons: Vec<(&String, &Vec<&str>)> = by_reason.iter().collect();
    reasons.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then(a.0.cmp(b.0)));
    println!("  not used, by likely reason:");
    for (why, cards) in reasons {
        let ex: Vec<&str> = cards.iter().take(4).copied().collect();
        println!("    {:5} {why}  (e.g. {})", cards.len(), ex.join("; "));
    }
    let mut failures: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for r in results {
        for (g, kind, what) in &r.failures {
            let first = what.lines().next().unwrap_or("");
            failures
                .entry(format!("{kind}: {}", kind_of(first)))
                .or_default()
                .push(format!(
                    "{} (--from {} --game {g})",
                    pool[r.index].def.name, r.index
                ));
        }
    }
    let n_fail: usize = failures.values().map(Vec::len).sum();
    println!("  failures: {n_fail} games, {} kinds", failures.len());
    for (k, v) in &failures {
        println!("    {} x {k}", v.len());
        for e in v.iter().take(3) {
            println!("        {e}");
        }
    }
    if let Some(path) = &o.report {
        let mut f =
            std::fs::File::create(path).unwrap_or_else(|e| panic!("can't write {path}: {e}"));
        let _ = writeln!(f, "index\tcard\tnot used\tability");
        for r in &rows {
            let _ = writeln!(f, "{r}");
        }
        for (k, v) in &failures {
            for e in v {
                let _ = writeln!(f, "failure\t{e}\t{k}");
            }
        }
    }
    if n_fail > 0 {
        std::process::exit(1);
    }
}
