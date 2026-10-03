//! CR 702.125 Undaunted.

use crate::common_k702_125_139::*;
use mtg_engine::game::GameConfig;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

const NORMAL: CastMethod = CastMethod::Normal;

#[test]
fn undaunted_reduces_the_cost_by_one_for_each_opponent() {
    cr!("702.125", "702.125a");
    ruling!(
        "Sublime Exhalation",
        "Effects that reduce what you pay to cast a spell don't affect its mana value."
    );
    assert_supported_card("Sublime Exhalation");
    // Two players: {6}{W} costs {5}{W}.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 5);
    let se = t.hand(P0, "Sublime Exhalation");
    assert!(!castable(&mut t, P0, se, NORMAL));
    t.lands(P0, "Plains", 1);
    assert!(castable(&mut t, P0, se, NORMAL));
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = t.cast(P0, se).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.g.mana_value_of(spell), 7);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    // Four players: it costs {3}{W}.
    let mut t = TestGame::new(4);
    t.lands(P0, "Plains", 3);
    let se = t.hand(P0, "Sublime Exhalation");
    assert!(!castable(&mut t, P0, se, NORMAL));
    t.lands(P0, "Plains", 1);
    assert!(castable(&mut t, P0, se, NORMAL));
    t.cast(P0, se).go();
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn only_opponents_count_not_teammates() {
    cr!("702.125a");
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            teams: Some(vec![0, 1, 0, 1]),
            ..Default::default()
        },
    );
    // P0 has two opponents (P1, P3): {5}{B} Curtains' Call costs {3}{B}.
    t.lands(P0, "Swamp", 3);
    let cc = t.hand(P0, "Curtains' Call");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P3, "Grizzly Bears");
    assert!(!castable(&mut t, P0, cc, NORMAL));
    t.lands(P0, "Swamp", 1);
    assert!(castable(&mut t, P0, cc, NORMAL));
}

#[test]
fn players_who_left_the_game_arent_counted() {
    cr!("702.125b");
    let mut t = TestGame::new(4);
    t.lands(P0, "Plains", 4);
    let se = t.hand(P0, "Sublime Exhalation");
    assert!(castable(&mut t, P0, se, NORMAL));
    // P3 concedes: P0 now has two opponents, and it costs {4}{W}.
    t.g.perform_action(P3, mtg_engine::decision::Action::Concede)
        .unwrap();
    t.settle();
    assert!(!t.g.player(P3).in_game());
    assert!(!castable(&mut t, P0, se, NORMAL));
    t.lands(P0, "Plains", 1);
    assert!(castable(&mut t, P0, se, NORMAL));
}

#[test]
fn each_instance_of_undaunted_applies() {
    cr!("702.125c");
    let def = custom_card(
        "Doubly Undaunted Exhalation",
        "Sorcery",
        None,
        "Undaunted\nUndaunted\nDestroy all creatures.",
    );
    let mut def = def;
    def.faces[0].chars.mana_cost = mtg_engine::mana::ManaCost::parse("{6}{W}");
    let mut t = TestGame::new(3);
    let se = t.custom(P0, def, Zone::Hand(P0));
    // Two opponents, two instances: {6}{W} costs {2}{W}.
    t.lands(P0, "Plains", 2);
    assert!(!castable(&mut t, P0, se, NORMAL));
    t.lands(P0, "Plains", 1);
    assert!(castable(&mut t, P0, se, NORMAL));
    t.cast(P0, se).go();
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn undaunted_reduces_only_generic_mana() {
    cr!("702.125a");
    // Eight players: seven opponents, but {6}{W} can't cost less than {W}.
    let mut t = TestGame::new(8);
    t.lands(P0, "Plains", 1);
    let se = t.hand(P0, "Sublime Exhalation");
    assert!(castable(&mut t, P0, se, NORMAL));
    let mut t = TestGame::new(8);
    t.lands(P0, "Island", 1);
    let se = t.hand(P0, "Sublime Exhalation");
    assert!(!castable(&mut t, P0, se, NORMAL));
}
