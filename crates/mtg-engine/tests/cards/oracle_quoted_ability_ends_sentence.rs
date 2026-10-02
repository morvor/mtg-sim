//! A quoted ability that ends a sentence ("... with \"This creature can't block.\" Creatures
//! you control gain haste until end of turn."): the next sentence is its own instruction.

use mtg_engine::oracle::effects::split_sentences;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_quoted_ability_ending_in_a_period_ends_the_sentence() {
    assert_eq!(
        split_sentences(
            "Create X 1/1 black Rat creature tokens with \"~ can't block.\" Creatures you control gain haste until end of turn."
        ),
        vec![
            "Create X 1/1 black Rat creature tokens with \"~ can't block.\"",
            "Creatures you control gain haste until end of turn.",
        ]
    );
    // A quoted ability followed by more of the same sentence stays in it.
    assert_eq!(
        split_sentences("Target creature gains \"{T}: Draw a card.\" until end of turn."),
        vec!["Target creature gains \"{T}: Draw a card.\" until end of turn."]
    );
}

#[test]
fn song_of_totentanz_creates_rats_then_gives_haste() {
    cr!("608.2c");
    let c = mtg_engine::card::card("Song of Totentanz");
    assert!(c.unsupported_text().is_empty(), "{:?}", c.unsupported_text());
    let mut t = TestGame::new(2);
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let s = t.hand(P0, "Song of Totentanz");
    t.cast(P0, s).x(2).go();
    t.resolve_all();
    let rats = t.named_on_battlefield("Rat Token");
    assert_eq!(rats.len(), 2, "{}", t.dump_log());
    for r in rats.iter().chain([&bears]) {
        assert!(t.obj_now(*r).has_keyword(keywords::KeywordKind::Haste));
    }
}
