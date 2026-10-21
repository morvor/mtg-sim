//! Card tiers: how sure we are that the engine plays a card right, lowest first.
//!
//! * [`Tier::Unsupported`]: some ability isn't compiled.
//! * [`Tier::Compiled`]: every ability on every face compiled or hand-written
//!   (`CardDef::is_fully_supported`).
//! * [`Tier::RoundTrip`]: compiled, and the Oracle round trip passes
//!   (`docs/roundtrip-passing.txt`); a card with hand-written abilities also needs
//!   `manual-check` to pass for it.
//! * [`Tier::Verified`]: round-trip, and every ability structure on the card is exercised by
//!   some test (`structure-coverage`), and the latest every-card sweep cast or played it
//!   with no panic, hang, slow game or rules-check violation in its games.
//!
//! `mtg-tools verified --write` computes every card's tier from those inputs into
//! `docs/verified-cards.tsv` ([`TierTable`]). Simulations (`mtg-sim`, `mtg_api::Session`)
//! use [`TierTable::tier_of`] to keep cards below a bar out of their decks; tests and the
//! engine's `Game` never check tiers. A card missing from the file (card data newer than the
//! last regeneration) is at most [`Tier::Compiled`], and when the file and the card's live
//! status disagree (it no longer compiles or round-trips) the lower tier wins.

use crate::card::CardDef;
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// The file `mtg-tools verified --write` writes, relative to the repository root.
pub const TIERS_FILE: &str = "docs/verified-cards.tsv";
/// Environment variable naming another tier file to load instead of [`TIERS_FILE`].
pub const TIERS_ENV: &str = "MTG_VERIFIED_CARDS";

/// A card's tier. Ordered lowest first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tier {
    Unsupported,
    Compiled,
    RoundTrip,
    Verified,
}

impl Tier {
    pub const ALL: [Tier; 4] = [
        Tier::Unsupported,
        Tier::Compiled,
        Tier::RoundTrip,
        Tier::Verified,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Tier::Unsupported => "unsupported",
            Tier::Compiled => "compiled",
            Tier::RoundTrip => "round-trip",
            Tier::Verified => "verified",
        }
    }

    /// Parses a tier name ("verified", "round-trip", "compiled", "unsupported").
    pub fn parse(s: &str) -> Option<Tier> {
        match s.trim().to_ascii_lowercase().as_str() {
            "unsupported" => Some(Tier::Unsupported),
            "compiled" => Some(Tier::Compiled),
            "round-trip" | "roundtrip" | "round_trip" => Some(Tier::RoundTrip),
            "verified" => Some(Tier::Verified),
            _ => None,
        }
    }
}

impl fmt::Display for Tier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl std::str::FromStr for Tier {
    type Err = String;
    fn from_str(s: &str) -> Result<Tier, String> {
        Tier::parse(s).ok_or_else(|| {
            format!("unknown card tier {s:?} (verified, round-trip, compiled or unsupported)")
        })
    }
}

/// The names of the checks, as the tier file and every report write them.
pub mod check {
    /// Every ability compiled or hand-written.
    pub const COMPILED: &str = "compiled";
    /// The Oracle round trip passes.
    pub const ROUND_TRIP: &str = "round-trip";
    /// `manual-check` passes for a card with hand-written abilities.
    pub const MANUAL: &str = "manual-check";
    /// Every ability structure is exercised by some test.
    pub const STRUCTURE: &str = "structure";
    /// The every-card sweep played games with the card.
    pub const FUZZ_SWEPT: &str = "fuzz-swept";
    /// The sweep cast or played it.
    pub const FUZZ_CAST: &str = "fuzz-cast";
    /// No panic, hang, slow game or rules violation in its games.
    pub const FUZZ_CLEAN: &str = "fuzz-clean";
    /// The card is in the tier file (checked at run time only).
    pub const LISTED: &str = "listed";
    /// Every check the tier file records, in the order they're reported.
    pub const ALL: [&str; 7] = [
        COMPILED, ROUND_TRIP, MANUAL, STRUCTURE, FUZZ_SWEPT, FUZZ_CAST, FUZZ_CLEAN,
    ];
}

/// A failed check: its name (one of [`check`]) and what failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failed {
    pub check: String,
    pub detail: String,
}

impl Failed {
    pub fn new(check: &str, detail: impl Into<String>) -> Failed {
        Failed {
            check: check.to_string(),
            detail: detail.into(),
        }
    }

    /// Parses "check (detail)" or "check".
    fn parse(s: &str) -> Failed {
        let s = s.trim();
        match s.split_once(" (") {
            Some((c, d)) => Failed::new(c, d.strip_suffix(')').unwrap_or(d)),
            None => Failed::new(s, ""),
        }
    }
}

impl fmt::Display for Failed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.detail.is_empty() {
            f.write_str(&self.check)
        } else {
            write!(f, "{} ({})", self.check, self.detail)
        }
    }
}

/// A card's tier and the checks it fails.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardTier {
    pub tier: Tier,
    pub failed: Vec<Failed>,
}

impl CardTier {
    /// The failed checks, "; "-separated.
    pub fn failed_text(&self) -> String {
        self.failed
            .iter()
            .map(Failed::to_string)
            .collect::<Vec<_>>()
            .join("; ")
    }

    fn fails(&self, check: &str) -> bool {
        self.failed.iter().any(|f| f.check == check)
    }
}

/// What the every-card sweep found for a card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FuzzOutcome {
    /// The card wasn't in the sweep's report.
    Missing,
    Swept {
        /// It was cast or played.
        cast: bool,
        /// Why it doesn't need to be cast (a supplementary-deck card, no mana cost), if so.
        exempt: Option<String>,
        /// Why it wasn't, if it wasn't (the report's reason).
        not_cast: Option<String>,
        /// The kinds of failed games: "panic", "hang", "slow", "rules".
        failures: Vec<String>,
    },
}

/// Everything a card's tier is computed from.
#[derive(Clone, Debug)]
pub struct Evidence {
    /// The ability texts that didn't compile (empty when the card is fully supported).
    pub unsupported: Vec<String>,
    /// The Oracle round trip passes (in `docs/roundtrip-passing.txt`, and live).
    pub round_trip: bool,
    /// `manual-check`'s problems for this card (hand-written abilities).
    pub manual_problems: Vec<String>,
    /// Number of the card's distinct ability structures, and how many no test exercises.
    pub structures: usize,
    pub unexercised: usize,
    pub fuzz: FuzzOutcome,
}

/// Computes a card's tier from its evidence: the highest tier whose checks all pass, with
/// every failed check recorded (the round-trip, structure and fuzz checks are reported
/// for every compiled card, whatever its tier).
pub fn classify(e: &Evidence) -> CardTier {
    let mut failed = Vec::new();
    if !e.unsupported.is_empty() {
        failed.push(Failed::new(
            check::COMPILED,
            format!("{} unsupported", e.unsupported.len()),
        ));
        return CardTier {
            tier: Tier::Unsupported,
            failed,
        };
    }
    if !e.round_trip {
        failed.push(Failed::new(check::ROUND_TRIP, ""));
    }
    if !e.manual_problems.is_empty() {
        failed.push(Failed::new(check::MANUAL, e.manual_problems.join(" / ")));
    }
    if e.unexercised > 0 {
        failed.push(Failed::new(
            check::STRUCTURE,
            format!("{} of {} unexercised", e.unexercised, e.structures),
        ));
    }
    match &e.fuzz {
        FuzzOutcome::Missing => failed.push(Failed::new(check::FUZZ_SWEPT, "not in the report")),
        FuzzOutcome::Swept {
            cast,
            exempt,
            not_cast,
            failures,
        } => {
            if !cast && exempt.is_none() {
                failed.push(Failed::new(
                    check::FUZZ_CAST,
                    not_cast.clone().unwrap_or_default(),
                ));
            }
            if !failures.is_empty() {
                failed.push(Failed::new(check::FUZZ_CLEAN, failures.join(", ")));
            }
        }
    }
    let t = CardTier {
        tier: Tier::Compiled,
        failed,
    };
    let tier = if t.fails(check::ROUND_TRIP) || t.fails(check::MANUAL) {
        Tier::Compiled
    } else if t.failed.is_empty() {
        Tier::Verified
    } else {
        Tier::RoundTrip
    };
    CardTier { tier, ..t }
}

/// A card's live tier, from its compiled definition alone: unsupported, compiled, or (when
/// the round trip passes now) round-trip. Structure coverage and fuzzing aren't live, so
/// this is never [`Tier::Verified`]; it caps what the tier file says.
pub fn live_tier(def: &CardDef) -> CardTier {
    let unsupported = def.unsupported_text();
    if !unsupported.is_empty() {
        return CardTier {
            tier: Tier::Unsupported,
            failed: vec![Failed::new(
                check::COMPILED,
                format!("{} unsupported", unsupported.len()),
            )],
        };
    }
    if !crate::oracle::render::compare::check_card(def).pass {
        return CardTier {
            tier: Tier::Compiled,
            failed: vec![Failed::new(check::ROUND_TRIP, "fails now")],
        };
    }
    CardTier {
        tier: Tier::RoundTrip,
        failed: vec![],
    }
}

/// The tiers of `docs/verified-cards.tsv`, by card name.
#[derive(Clone, Debug, Default)]
pub struct TierTable {
    by_name: HashMap<String, CardTier>,
    /// Where it was loaded from (for messages).
    pub source: String,
}

impl TierTable {
    /// Parses the tier file: `name<TAB>tier<TAB>failed checks[<TAB>notes]` lines, with a
    /// header line and `#` comments.
    pub fn parse(text: &str) -> TierTable {
        let mut by_name = HashMap::new();
        for line in text.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            let mut cols = line.split('\t');
            let (Some(name), Some(tier)) = (cols.next(), cols.next()) else {
                continue;
            };
            let Some(tier) = Tier::parse(tier) else {
                continue; // the header line
            };
            let failed = cols
                .next()
                .unwrap_or("")
                .split("; ")
                .filter(|s| !s.trim().is_empty())
                .map(Failed::parse)
                .collect();
            by_name.insert(name.to_lowercase(), CardTier { tier, failed });
        }
        TierTable {
            by_name,
            source: "(text)".into(),
        }
    }

    /// Loads a tier file; a missing or unreadable file gives an empty table (every card at
    /// most compiled).
    pub fn load(path: &Path) -> TierTable {
        let text = std::fs::read_to_string(path).unwrap_or_default();
        TierTable {
            source: path.display().to_string(),
            ..TierTable::parse(&text)
        }
    }

    /// The default tier file: `$MTG_VERIFIED_CARDS`, or the repository's
    /// `docs/verified-cards.tsv`.
    pub fn default_path() -> PathBuf {
        match std::env::var_os(TIERS_ENV) {
            Some(p) => PathBuf::from(p),
            None => PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")).join(TIERS_FILE),
        }
    }

    /// The default tier file, loaded once.
    pub fn global() -> &'static TierTable {
        static T: OnceLock<TierTable> = OnceLock::new();
        T.get_or_init(|| TierTable::load(&TierTable::default_path()))
    }

    pub fn len(&self) -> usize {
        self.by_name.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }

    /// What the file says about a card, if it lists it.
    pub fn recorded(&self, name: &str) -> Option<&CardTier> {
        self.by_name.get(&name.to_lowercase())
    }

    /// A card's tier: what the file says, capped by its live status ([`live_tier`]); a card
    /// the file doesn't list is at most compiled. The lower tier wins, with the reasons of
    /// both.
    pub fn tier_of(&self, def: &CardDef) -> CardTier {
        let recorded = self
            .recorded(&def.name)
            .cloned()
            .unwrap_or_else(|| CardTier {
                tier: Tier::Compiled,
                failed: vec![Failed::new(
                    check::LISTED,
                    format!(
                    "not in {TIERS_FILE}: not a paper card legal in some format, or newer than the file"
                ),
                )],
            });
        // The live check costs a render: skip it when the file already puts the card at
        // the bottom.
        if recorded.tier == Tier::Unsupported {
            return recorded;
        }
        // The live status can only lower the tier: a live round-trip pass says nothing
        // about the checks that aren't live.
        let live = live_tier(def);
        if live.tier == Tier::RoundTrip {
            recorded
        } else {
            combine(recorded, live)
        }
    }

    /// The cards of `cards` below `bar` (each name once, in first-seen order), with their
    /// tiers.
    pub fn below<'a>(
        &self,
        cards: impl IntoIterator<Item = &'a CardDef>,
        bar: Tier,
    ) -> Vec<(String, CardTier)> {
        let mut out: Vec<(String, CardTier)> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for c in cards {
            if !seen.insert(c.name.to_lowercase()) {
                continue;
            }
            let t = self.tier_of(c);
            if t.tier < bar {
                out.push((c.name.to_string(), t));
            }
        }
        out
    }
}

/// The lower of two tiers, keeping every reason either gives.
pub fn combine(a: CardTier, b: CardTier) -> CardTier {
    let tier = a.tier.min(b.tier);
    let mut failed = a.failed;
    for f in b.failed {
        if !failed.iter().any(|g| g.check == f.check) {
            failed.push(f);
        }
    }
    CardTier { tier, failed }
}

/// A deck rejected for holding cards below the bar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BelowBar {
    pub bar: Tier,
    /// (card, its tier and failed checks).
    pub cards: Vec<(String, CardTier)>,
}

impl fmt::Display for BelowBar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{} card(s) below the {} tier (see {TIERS_FILE}; a lower bar lets them in):",
            self.cards.len(),
            self.bar
        )?;
        for (name, t) in &self.cards {
            writeln!(f, "  {name}: {}; fails {}", t.tier, t.failed_text())?;
        }
        Ok(())
    }
}

impl std::error::Error for BelowBar {}

/// Checks that every card of `cards` is at `bar` or above.
pub fn check_cards<'a>(
    table: &TierTable,
    cards: impl IntoIterator<Item = &'a CardDef>,
    bar: Tier,
) -> Result<(), BelowBar> {
    let below = table.below(cards, bar);
    if below.is_empty() {
        Ok(())
    } else {
        Err(BelowBar { bar, cards: below })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence() -> Evidence {
        Evidence {
            unsupported: vec![],
            round_trip: true,
            manual_problems: vec![],
            structures: 2,
            unexercised: 0,
            fuzz: FuzzOutcome::Swept {
                cast: true,
                exempt: None,
                not_cast: None,
                failures: vec![],
            },
        }
    }

    fn checks(t: &CardTier) -> Vec<&str> {
        t.failed.iter().map(|f| f.check.as_str()).collect()
    }

    #[test]
    fn a_card_passing_everything_is_verified() {
        let t = classify(&evidence());
        assert_eq!(t.tier, Tier::Verified);
        assert!(t.failed.is_empty());
    }

    #[test]
    fn each_failed_check_puts_the_card_in_its_tier() {
        let mut e = evidence();
        e.unsupported = vec!["Some text.".into()];
        let t = classify(&e);
        assert_eq!((t.tier, checks(&t)), (Tier::Unsupported, vec!["compiled"]));

        let mut e = evidence();
        e.round_trip = false;
        let t = classify(&e);
        assert_eq!((t.tier, checks(&t)), (Tier::Compiled, vec!["round-trip"]));

        let mut e = evidence();
        e.manual_problems = vec!["no test".into()];
        let t = classify(&e);
        assert_eq!((t.tier, checks(&t)), (Tier::Compiled, vec!["manual-check"]));

        let mut e = evidence();
        e.unexercised = 1;
        let t = classify(&e);
        assert_eq!((t.tier, checks(&t)), (Tier::RoundTrip, vec!["structure"]));
        assert_eq!(t.failed_text(), "structure (1 of 2 unexercised)");

        let mut e = evidence();
        e.fuzz = FuzzOutcome::Missing;
        let t = classify(&e);
        assert_eq!((t.tier, checks(&t)), (Tier::RoundTrip, vec!["fuzz-swept"]));

        let mut e = evidence();
        e.fuzz = FuzzOutcome::Swept {
            cast: false,
            exempt: None,
            not_cast: Some("never legal to cast".into()),
            failures: vec![],
        };
        let t = classify(&e);
        assert_eq!((t.tier, checks(&t)), (Tier::RoundTrip, vec!["fuzz-cast"]));

        let mut e = evidence();
        e.fuzz = FuzzOutcome::Swept {
            cast: true,
            exempt: None,
            not_cast: None,
            failures: vec!["panic".into(), "slow".into()],
        };
        let t = classify(&e);
        assert_eq!((t.tier, checks(&t)), (Tier::RoundTrip, vec!["fuzz-clean"]));
        assert_eq!(t.failed_text(), "fuzz-clean (panic, slow)");
    }

    #[test]
    fn a_card_the_sweep_cannot_cast_is_exempt_from_being_cast() {
        let mut e = evidence();
        e.fuzz = FuzzOutcome::Swept {
            cast: false,
            exempt: Some("no mana cost".into()),
            not_cast: Some("never cast: no mana cost".into()),
            failures: vec![],
        };
        assert_eq!(classify(&e).tier, Tier::Verified);
    }

    #[test]
    fn every_failed_check_of_a_compiled_card_is_recorded() {
        let mut e = evidence();
        e.round_trip = false;
        e.unexercised = 2;
        e.fuzz = FuzzOutcome::Missing;
        let t = classify(&e);
        assert_eq!(t.tier, Tier::Compiled);
        assert_eq!(checks(&t), vec!["round-trip", "structure", "fuzz-swept"]);
    }

    #[test]
    fn the_tier_file_round_trips_and_the_lower_tier_wins() {
        let table = TierTable::parse(
            "# comment\nname\ttier\tfailed\tnotes\nGrizzly Bears\tverified\t\t\nHill Giant\tround-trip\tstructure (1 of 1 unexercised)\t\n",
        );
        assert_eq!(table.len(), 2);
        let bears = table.recorded("grizzly bears").unwrap();
        assert_eq!(bears.tier, Tier::Verified);
        let giant = table.recorded("Hill Giant").unwrap();
        assert_eq!(
            giant.failed,
            vec![Failed::new("structure", "1 of 1 unexercised")]
        );
        let low = combine(
            bears.clone(),
            CardTier {
                tier: Tier::Compiled,
                failed: vec![Failed::new(check::ROUND_TRIP, "fails now")],
            },
        );
        assert_eq!(low.tier, Tier::Compiled);
        assert_eq!(low.failed_text(), "round-trip (fails now)");
    }

    #[test]
    fn the_live_status_lowers_the_recorded_tier_and_unlisted_cards_are_at_most_compiled() {
        let table = TierTable::parse("Grizzly Bears\tverified\t\t\n");
        let bears = crate::card::CardDb::global().get("Grizzly Bears").unwrap();
        assert_eq!(table.tier_of(&bears).tier, Tier::Verified);
        // Its Oracle text changed, so the round trip fails now.
        let mut changed = (*bears).clone();
        changed.faces[0].chars.rules_text = "Flying".into();
        let t = table.tier_of(&changed);
        assert_eq!(t.tier, Tier::Compiled);
        assert_eq!(t.failed_text(), "round-trip (fails now)");
        // An ability no longer compiles.
        let mut broken = (*bears).clone();
        broken.faces[0].unsupported = vec!["Some new text.".into()];
        let t = table.tier_of(&broken);
        assert_eq!(
            (t.tier, t.failed_text()),
            (Tier::Unsupported, "compiled (1 unsupported)".into())
        );
        // A card the file doesn't list.
        let giant = crate::card::CardDb::global().get("Hill Giant").unwrap();
        let t = table.tier_of(&giant);
        assert_eq!(t.tier, Tier::Compiled);
        assert_eq!(t.failed[0].check, check::LISTED);
        assert!(check_cards(&table, [bears.as_ref()], Tier::Verified).is_ok());
        let e = check_cards(
            &table,
            [bears.as_ref(), giant.as_ref(), giant.as_ref()],
            Tier::Verified,
        )
        .unwrap_err();
        assert_eq!(e.cards.len(), 1);
        assert!(e.to_string().contains("Hill Giant: compiled; fails listed"));
    }

    #[test]
    fn tiers_parse_and_order() {
        assert!(Tier::Unsupported < Tier::Compiled);
        assert!(Tier::RoundTrip < Tier::Verified);
        for t in Tier::ALL {
            assert_eq!(Tier::parse(t.name()), Some(t));
        }
        assert!("bogus".parse::<Tier>().is_err());
    }
}
