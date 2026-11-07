//! "... and you scry 1", "Then you scry 2.", "you surveil 2": scry and surveil with the
//! subject "you" (pattern in `src/oracle/patterns/a701_scry_surveil.rs`).

use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

/// The number of cards in each scry decision P0 was asked.
fn scries(t: &TestGame) -> Vec<usize> {
    t.asked()
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::Scry { cards } if *p == P0 => Some(cards.len()),
            _ => None,
        })
        .collect()
}

#[test]
fn you_scry_cards_compile() {
    assert_compiles(&[
        "Overwhelmed Apprentice",
        "Psychic Impetus",
        "The Scarab God",
        "Alibou, Ancient Witness",
        "Clockwork Droid",
    ]);
}

#[test]
fn overwhelmed_apprentice_mills_each_opponent_then_you_scry() {
    cr!("701.22a", "701.17a");
    // "When this creature enters, each opponent mills two cards. Then you scry 2."
    let mut t = TestGame::new(2);
    t.enter(P0, "Overwhelmed Apprentice");
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 2);
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(scries(&t), vec![2]);
}

#[test]
fn the_scarab_god_drains_and_scries_x() {
    cr!("701.22a", "503.1a");
    // "At the beginning of your upkeep, each opponent loses X life and you scry X, where X
    // is the number of Zombies you control."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Scarab God");
    t.battlefield(P0, "Walking Corpse");
    t.battlefield(P0, "Walking Corpse");
    t.set_step(P1, mtg_engine::turn::Step::End);
    t.advance_to(P0, mtg_engine::turn::Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(scries(&t), vec![2]);
}
