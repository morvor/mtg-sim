//! `ScryfallCard::is_paper`: the one rule every tool uses to decide whether a card is a
//! paper card (docs/DEFERRED.md, "Digital-only cards").

use mtg_data::scryfall::ScryfallCard;

/// Paper cards whose oracle record is an online-only printing (`games` lacks "paper"),
/// counted as paper by their paper-format legality.
#[test]
fn paper_cards_with_an_online_only_oracle_printing() {
    let cards = mtg_data::cards();
    for (name, set) in [
        ("City of Traitors", "tpr"),
        ("Fastbond", "vma"),
        ("Memnite", "td2"),
        ("Raging Goblin", "oana"),
        ("Pyrogoyf", "aa2"),
    ] {
        let c = cards.by_name(name).unwrap_or_else(|| panic!("{name}"));
        // The case the rule exists for: the oracle printing isn't a paper printing.
        assert_eq!(c.set, set, "{name}");
        assert!(
            !c.games.iter().any(|g| g == "paper"),
            "{name}: {:?}",
            c.games
        );
        assert!(c.is_paper(), "{name}");
    }
    // An ordinary paper card.
    assert!(cards.by_name("Lightning Bolt").unwrap().is_paper());
}

/// Cards with no paper printing: legal only in Arena formats.
#[test]
fn arena_only_cards_are_not_paper() {
    let c = mtg_data::cards().by_name("Ethereal Grasp").unwrap();
    assert_eq!(c.set, "j21");
    assert!(c.is_legal_somewhere());
    assert!(!c.is_paper());
}

/// Digital versions of cards: an Alchemy rebalanced "A-" card (the oracle bulk data
/// carries none, so this one is built the way Scryfall records them).
#[test]
fn alchemy_rebalanced_cards_are_not_paper() {
    let c: ScryfallCard = serde_json::from_value(serde_json::json!({
        "id": "x",
        "oracle_id": "y",
        "name": "A-Luminarch Aspirant",
        "layout": "normal",
        "set": "ymid",
        "games": ["arena"],
        "legalities": {
            "standard": "not_legal", "pioneer": "not_legal", "modern": "not_legal",
            "legacy": "not_legal", "vintage": "not_legal", "commander": "not_legal",
            "alchemy": "legal", "historic": "legal", "timeless": "legal",
            "brawl": "legal", "gladiator": "legal"
        }
    }))
    .unwrap();
    assert!(c.is_legal_somewhere());
    assert!(!c.is_paper());

    // A paper printing (or no `games` at all) is enough; so is a paper-format legality
    // of any kind, banned included.
    let mut paper = c.clone();
    paper.games = vec!["paper".into(), "arena".into()];
    assert!(paper.is_paper());
    paper.games.clear();
    assert!(paper.is_paper());
    let mut banned = c.clone();
    banned.games = vec!["mtgo".into()];
    banned.legalities.insert("vintage".into(), "banned".into());
    assert!(banned.is_paper());
}

/// A digital version listed legal in a paper format: "Name Sticker" Goblin, the MTGO
/// version of _____ Goblin, is not paper; the paper card is.
#[test]
fn the_mtgo_name_sticker_goblin_is_a_digital_version() {
    let cards = mtg_data::cards();
    let mtgo = cards.by_name("\"Name Sticker\" Goblin").unwrap();
    assert_eq!(mtgo.games, vec!["mtgo".to_string()]);
    assert_eq!(mtgo.legality("commander"), Some("legal"));
    assert!(!mtgo.is_paper());
    let paper = cards.by_name("_____ Goblin").unwrap();
    assert!(paper.is_paper());
}
