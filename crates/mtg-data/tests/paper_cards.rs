//! Paper status (`ScryfallCard::is_paper_card`) counts every printing of a card, not only the
//! one printing the Oracle Cards file has for it.

use mtg_data::scryfall::{read_oracle_ids, ScryfallCard, PAPER_ORACLE_IDS};

#[test]
fn a_card_is_paper_when_any_printing_is() {
    let card = |games: &[&str], paper_printing| ScryfallCard {
        games: games.iter().map(|g| g.to_string()).collect(),
        paper_printing,
        ..Default::default()
    };
    assert!(!card(&["mtgo"], false).is_paper_card());
    assert!(!card(&["arena", "mtgo"], false).is_paper_card());
    // Another printing is a paper printing.
    assert!(card(&["mtgo"], true).is_paper_card());
    // This printing is (a card newer than the paper list).
    assert!(card(&["paper", "mtgo"], false).is_paper_card());
    // No printing data: not a card from Scryfall.
    assert!(card(&[], false).is_paper_card());
}

/// Paper cards whose Oracle Cards printing is an MTGO- or Arena-only reprint: Vintage
/// Masters, Masters Edition III, an MTGO duel deck and an Arena starter deck.
#[test]
fn a_paper_card_with_a_digital_oracle_printing_is_paper() {
    let cards = mtg_data::cards();
    for (name, set) in [
        ("Tundra", "vma"),
        ("Palinchron", "vma"),
        ("Heal", "me3"),
        ("Illusionary Mask", "me3"),
        ("Memnite", "td2"),
        ("Raging Goblin", "oana"),
    ] {
        let c = cards.by_name(name).unwrap();
        assert_eq!(c.set, set, "{name}'s Oracle Cards printing");
        assert!(!c.games.iter().any(|g| g == "paper"), "{name}");
        assert!(c.is_paper_card(), "{name}");
    }
    assert!(cards.by_name("Lightning Bolt").unwrap().is_paper_card());
}

/// Cards that were never printed on paper (MTG Arena cards) stay digital-only.
#[test]
fn a_card_with_no_paper_printing_is_not_paper() {
    let cards = mtg_data::cards();
    for name in ["Manor Guardian", "Gitrog, Horror of Zhava"] {
        assert!(!cards.by_name(name).unwrap().is_paper_card(), "{name}");
    }
}

/// The paper list lists every card whose Oracle Cards printing is a paper printing; a list
/// older than the card data misses the newer cards.
#[test]
fn the_paper_list_covers_the_card_data() {
    let paper = read_oracle_ids(&mtg_data::data_dir().join(PAPER_ORACLE_IDS)).unwrap();
    let missing: Vec<&str> = mtg_data::cards()
        .iter()
        .filter(|c| c.games.iter().any(|g| g == "paper") && !paper.contains(&c.oracle_id))
        .map(|c| c.name.as_str())
        .collect();
    assert!(
        missing.is_empty(),
        "{} paper cards aren't in data/{PAPER_ORACLE_IDS} (regenerate it with scripts/paper_oracle_ids.py), e.g. {:?}",
        missing.len(),
        &missing[..missing.len().min(5)]
    );
}
