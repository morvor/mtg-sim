//! Where a found card goes when the card offers a choice or a split: Illuna, Apex of
//! Wishes ("Put that card onto the battlefield or into your hand.",
//! `oracle/patterns/put_it_or.rs`), and split searches that find fewer cards than they
//! could (Jarad's Orders, Kodama's Reach).

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn illuna_puts_the_exiled_nonland_permanent_card_onto_the_battlefield_or_into_your_hand() {
    cr!("608.2d", "702.140d");
    assert_supported("Illuna, Apex of Wishes");
    ruling!(
        "Illuna, Apex of Wishes",
        "You choose whether to put the card into your hand or onto the battlefield after seeing what the card is."
    );
    ruling!(
        "Illuna, Apex of Wishes",
        "Any land, instant, and sorcery cards exiled this way remain in exile."
    );
    for (choice, battlefield) in [(0, true), (1, false)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Island", 4);
        t.lands(P0, "Mountain", 2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let ox = t.library_top(P0, "Hill Giant");
        let bolt = t.library_top(P0, "Lightning Bolt");
        let forest = t.library_top(P0, "Forest");
        let illuna = t.hand(P0, "Illuna, Apex of Wishes");
        t.cast(P0, illuna)
            .method(CastMethod::Keyword(KeywordKind::Mutate))
            .target(bears)
            .go();
        t.answer(P0, DecisionKind::Option, Answer::Index(0)); // on top
        t.answer(P0, DecisionKind::Option, Answer::Index(choice));
        t.resolve_all();
        let found = t.g.current(ox);
        if battlefield {
            assert_eq!(t.zone(found), Zone::Battlefield);
            assert_eq!(t.g.obj(found).controller, P0);
        } else {
            assert_eq!(t.zone(found), Zone::Hand(P0));
        }
        // The cards exiled before it stay in exile.
        assert_eq!(t.zone(t.g.current(bolt)), Zone::Exile);
        assert_eq!(t.zone(t.g.current(forest)), Zone::Exile);
        // The choice was offered (two places) after the card was found.
        let offered = t.asked().into_iter().any(|(p, d)| {
            p == P0
                && matches!(&d, Decision::ChooseOption { options, .. }
                    if options.len() == 2 && options[0].contains("battlefield") && options[1].contains("hand"))
        });
        assert!(offered);
    }
}

#[test]
fn jarads_orders_finding_one_card_puts_it_into_your_hand() {
    cr!("701.23b");
    assert_supported("Jarad's Orders");
    ruling!(
        "Jarad's Orders",
        "You can choose to find just one creature card. If you do, you’ll put that card into your hand."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Swamp", 2);
    let bears = t.library_top(P0, "Grizzly Bears");
    let giant = t.library_top(P0, "Hill Giant");
    let spell = t.hand(P0, "Jarad's Orders");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bears)), Zone::Hand(P0));
    assert_eq!(t.zone(giant), Zone::Library(P0));
    assert!(!t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn kodamas_reach_finding_one_land_puts_it_onto_the_battlefield_tapped() {
    cr!("701.23b");
    assert_supported("Kodama's Reach");
    ruling!(
        "Kodama's Reach",
        "If you get only one land card, you put it onto the battlefield tapped."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let plains = t.library_top(P0, "Plains");
    let island = t.library_top(P0, "Island");
    let spell = t.hand(P0, "Kodama's Reach");
    t.answer_choose(P0, &[Entity::Object(plains)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    let now = t.g.current(plains);
    assert_eq!(t.zone(now), Zone::Battlefield);
    assert!(t.g.obj(now).tapped);
    assert_eq!(t.zone(island), Zone::Library(P0));
}
