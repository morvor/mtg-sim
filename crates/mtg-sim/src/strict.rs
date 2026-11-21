//! Strict mode (`--cards verified|round-trip|compiled`): the lowest card tier
//! (`mtg_engine::card_tiers`) a deck's cards may have. Ordinary games (`--deck` files and the
//! default deck) default to `verified`; the fuzz modes (`--random-decks`, `--every-card`)
//! keep every fully supported card unless given the flag. A deck with a card below the bar
//! is rejected before the game starts, listing every such card and the checks it fails.
//! `--tiers FILE` reads another tier file than `docs/verified-cards.tsv`.

use crate::decks::DeckList;
use mtg_engine::card_tiers::{check_cards, BelowBar, Tier, TierTable};

/// The `--cards` and `--tiers` options.
#[derive(Clone, Debug, Default)]
pub struct Strict {
    /// `--cards`, if given.
    pub cards: Option<Tier>,
    /// `--tiers FILE`, if given.
    pub tiers: Option<String>,
}

impl Strict {
    /// Handles `--cards T` and `--tiers FILE` at `args[i]`: true if it was one of them.
    pub fn parse_arg(&mut self, args: &[String], i: usize) -> bool {
        let value = || {
            args.get(i + 1)
                .cloned()
                .unwrap_or_else(|| panic!("{} needs a value", args[i]))
        };
        match args[i].as_str() {
            "--cards" => {
                self.cards = Some(parse_bar(&value()).unwrap_or_else(|e| {
                    eprintln!("{e}");
                    std::process::exit(2)
                }))
            }
            "--tiers" => self.tiers = Some(value()),
            _ => return false,
        }
        true
    }

    /// The tier table: `--tiers FILE` or the default one.
    pub fn table(&self) -> TierTable {
        match &self.tiers {
            Some(f) => {
                if !std::path::Path::new(f).exists() {
                    eprintln!("can't read the tier file {f}");
                    std::process::exit(2);
                }
                TierTable::load(std::path::Path::new(f))
            }
            None => TierTable::global().clone(),
        }
    }

    /// The bar: `--cards`, or `default` (verified for ordinary games, none for fuzzing).
    pub fn bar(&self, default: Option<Tier>) -> Option<Tier> {
        self.cards.or(default)
    }

    /// The options again, for a reproduce command.
    pub fn repro_args(&self) -> String {
        let mut s = String::new();
        if let Some(t) = self.cards {
            s.push_str(&format!(" --cards {t}"));
        }
        if let Some(f) = &self.tiers {
            s.push_str(&format!(" --tiers {f}"));
        }
        s
    }
}

/// Parses `--cards`' value: verified, round-trip or compiled.
pub fn parse_bar(s: &str) -> Result<Tier, String> {
    match Tier::parse(s) {
        Some(Tier::Unsupported) | None => Err(format!(
            "--cards takes verified, round-trip or compiled, not {s:?}"
        )),
        Some(t) => Ok(t),
    }
}

/// Checks every card of a deck (main deck and sideboard) against the bar.
pub fn check_deck(table: &TierTable, deck: &DeckList, bar: Tier) -> Result<(), BelowBar> {
    check_cards(
        table,
        deck.main
            .iter()
            .chain(deck.sideboard.iter())
            .map(|c| c.as_ref()),
        bar,
    )
}

/// Checks the decks, printing what's below the bar and exiting with status 2 if any card
/// is. `what` names each deck ("--deck d.txt", "the default deck").
pub fn enforce(table: &TierTable, decks: &[(String, &DeckList)], bar: Tier) {
    let mut bad = false;
    for (what, d) in decks {
        if let Err(e) = check_deck(table, d, bar) {
            eprint!("{what}: {e}");
            bad = true;
        }
    }
    if bad {
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decks::{parse_decklist, random_deck};
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn table() -> TierTable {
        TierTable::parse(
            "name\ttier\tfailed\tnotes\n\
             Mountain\tverified\t\t\n\
             Forest\tverified\t\t\n\
             Grizzly Bears\tverified\t\t\n\
             Llanowar Elves\tverified\t\t\n\
             Hill Giant\tcompiled\tround-trip; fuzz-cast (never cast)\t\n\
             Raging Goblin\tround-trip\tstructure (1 of 1 unexercised)\t\n",
        )
    }

    #[test]
    fn a_deck_with_an_unverified_card_is_rejected_and_accepted_with_a_lower_bar() {
        let deck = parse_decklist("10 Forest\n4 Grizzly Bears\n4 Hill Giant\n2 Raging Goblin");
        let t = table();
        let e = check_deck(&t, &deck, Tier::Verified).unwrap_err();
        let names: Vec<&str> = e.cards.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, vec!["Hill Giant", "Raging Goblin"]);
        let text = e.to_string();
        assert!(text.contains("Hill Giant: compiled; fails round-trip; fuzz-cast (never cast)"));
        assert!(text.contains("Raging Goblin: round-trip; fails structure (1 of 1 unexercised)"));
        // Round-trip lets Raging Goblin in, not Hill Giant.
        let e = check_deck(&t, &deck, Tier::RoundTrip).unwrap_err();
        assert_eq!(e.cards.len(), 1);
        assert!(check_deck(&t, &deck, Tier::Compiled).is_ok());
    }

    #[test]
    fn a_card_missing_from_the_tier_file_is_at_most_compiled() {
        let deck = parse_decklist("Shock");
        let e = check_deck(&table(), &deck, Tier::RoundTrip).unwrap_err();
        assert_eq!(e.cards[0].1.tier, Tier::Compiled);
        assert!(e.cards[0]
            .1
            .failed_text()
            .contains("not in docs/verified-cards.tsv"));
        assert!(check_deck(&table(), &deck, Tier::Compiled).is_ok());
    }

    #[test]
    fn random_decks_with_a_bar_draw_only_cards_at_the_bar() {
        let t = table();
        let basics = ["Plains", "Island", "Swamp", "Mountain", "Forest"];
        let mut spells = 0;
        for seed in 0..20 {
            let mut rng = StdRng::seed_from_u64(seed);
            let d = random_deck(&mut rng, Some((&t, Tier::Verified)));
            for c in d.main.iter().filter(|c| !basics.contains(&c.name.as_str())) {
                assert_eq!(t.tier_of(c).tier, Tier::Verified, "{}", c.name);
                spells += 1;
            }
        }
        assert!(spells > 0, "some green deck drew the verified spells");
    }

    #[test]
    fn the_bar_parses() {
        assert_eq!(parse_bar("verified"), Ok(Tier::Verified));
        assert_eq!(parse_bar("round-trip"), Ok(Tier::RoundTrip));
        assert_eq!(parse_bar("compiled"), Ok(Tier::Compiled));
        assert!(parse_bar("unsupported").is_err());
        assert!(parse_bar("all").is_err());
    }
}
