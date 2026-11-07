//! In-game tests for parser bugs found while teaching the Oracle round-trip renderer
//! (`oracle/render/`) the AST nodes it couldn't put into words.

use mtg_engine::card::card;
use mtg_engine::oracle::render::compare::check_card;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_round_trips(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
    let r = check_card(&c);
    assert!(
        r.pass,
        "{name} doesn't round-trip:\n  oracle: {:?}\n  rendered: {:?}\n  gaps: {:?}",
        r.unmatched_oracle, r.unmatched_rendered, r.gaps
    );
}

/// "When this creature enters, each player discards a card. If you discarded a card this
/// way, draw a card." Only a card you discarded counts: when only an opponent discards
/// (your hand is empty), you don't draw. It used to count any player's discarded card.
#[test]
fn fanatic_of_the_harrowing_counts_only_your_discard() {
    cr!("608.2c", "701.9a");
    assert_round_trips("Fanatic of the Harrowing");
    let mut t = TestGame::new(2);
    t.hand(P1, "Grizzly Bears");
    let lib = t.library_size(P0);
    t.enter(P0, "Fanatic of the Harrowing");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "the opponent discarded");
    assert_eq!(t.library_size(P0), lib, "you discarded nothing, so you don't draw");

    // With a card in your hand, you discard it and draw.
    let mut t = TestGame::new(2);
    t.hand(P0, "Grizzly Bears");
    let lib = t.library_size(P0);
    t.enter(P0, "Fanatic of the Harrowing");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.library_size(P0), lib - 1, "you discarded a card, so you draw");
}

/// "For each creature card exiled this way, each opponent loses 1 life and you gain 1
/// life." (Graveyard Glutton): both halves happen for each card. "You gain 1 life" used to
/// be read as a separate instruction, so two creature cards exiled gained only 1 life.
#[test]
fn graveyard_glutton_gains_life_for_each_creature_card_exiled() {
    cr!("608.2c", "702.145c");
    let mut t = TestGame::new(2);
    let id = t.battlefield(P0, "Graveyard Trespasser");
    // It becomes night: it transforms into Graveyard Glutton.
    t.g.set_day(false);
    let glutton = t.g.current(id);
    assert_eq!(t.obj_now(glutton).chars.name.as_str(), "Graveyard Glutton");
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P1, "Hill Giant");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    let power = t.pt(glutton).0;
    t.attack(&[(glutton, Entity::Player(P1))], &[]);
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Hill Giant"));
    assert_eq!(
        t.life(P1),
        20 - 2 - power,
        "each opponent loses 1 life for each creature card (and combat damage)"
    );
    assert_eq!(t.life(P0), 22, "you gain 1 life for each creature card");
}
