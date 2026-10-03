//! CR 900: casual variants in general — optional rules that use supplemental zones,
//! rules, cards and game implements.

use crate::r100_common::{fillers, pregame};
use crate::r900_common::*;
use mtg_engine::card::{card, CardDef};
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::object::Zone;
use mtg_engine::planechase::{self, PLANAR_DIE_ACTION};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::variants;
use mtg_engine::*;
use std::sync::Arc;

/// A 40-card deck that also lists a plane, a vanguard, and a scheme.
fn deck_with_supplements() -> Vec<Arc<CardDef>> {
    let mut d = fillers(40);
    d.extend(cards(&["Krosa", "Titania", "Roots of All Evil"]));
    d
}

fn started(config: GameConfig, teams: Option<Vec<u8>>) -> TestGame {
    let mut t = pregame(
        GameConfig {
            skip_mulligans: true,
            starting_player: Some(P0),
            teams,
            ..config
        },
        vec![deck_with_supplements(), fillers(40), fillers(40)],
    );
    t.g.start();
    t
}

fn can_roll(t: &mut TestGame, p: PlayerId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p).iter().any(|a| {
        matches!(a, Action::Special(SpecialAction::Other { name, .. }) if name.as_str() == PLANAR_DIE_ACTION)
    })
}

#[test]
fn casual_variants_are_optional_additional_rules() {
    cr!("900.1");
    // Without a casual variant, none of their rules apply: no starting plane, no vanguard
    // modifiers, no scheme set in motion.
    let mut t = started(GameConfig::default(), Some(vec![0, 1, 1]));
    assert!(face_up_names(&t).is_empty());
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.hand_size(P0), 7);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(variants::face_up_schemes(&t.g).is_empty());
    // Each variant, when used, adds its rules to the normal ones.
    let t = started(GameConfig::planechase_game(), None);
    assert_eq!(face_up_names(&t), vec!["Krosa"]);
    let t = started(GameConfig::vanguard_game(), None);
    assert_eq!((t.life(P0), t.hand_size(P0)), (15, 9));
    let mut t = started(
        GameConfig {
            variant: Variant::Archenemy,
            ..Default::default()
        },
        Some(vec![0, 1, 1]),
    );
    assert_eq!(t.life(P0), 40);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(
        t.g.permanents()
            .filter(|o| o.chars.has_subtype("Saproling"))
            .count(),
        5
    );
}

#[test]
fn casual_variants_use_supplemental_zones_cards_and_implements() {
    cr!("900.2");
    // Supplemental cards listed with a deck aren't part of it: they start in the command
    // zone — a plane and a scheme face down (in supplementary decks), a vanguard face up.
    let t = pregame(
        GameConfig::planechase_game(),
        vec![deck_with_supplements(), fillers(40)],
    );
    assert_eq!(t.g.player(P0).library.len(), 40);
    let supplements: Vec<(String, bool)> = t
        .g
        .command
        .iter()
        .map(|id| (name_of(&t, *id), t.obj(*id).face_down))
        .collect();
    assert_eq!(
        supplements,
        vec![
            ("Krosa".to_string(), true),
            ("Titania".to_string(), false),
            ("Roots of All Evil".to_string(), true),
        ]
    );
    for id in t.g.command.clone() {
        assert_eq!(t.zone(id), Zone::Command);
    }
    // A game implement: the planar die, used only in Planechase.
    let mut t = planechase_game(2, false);
    add_planar_deck(&mut t, P0, &["Krosa"]);
    planechase::set_starting_plane(&mut t.g);
    assert!(can_roll(&mut t, P0));
    let mut t = TestGame::new(2);
    assert!(!can_roll(&mut t, P0));
    let _ = card("Krosa");
}
