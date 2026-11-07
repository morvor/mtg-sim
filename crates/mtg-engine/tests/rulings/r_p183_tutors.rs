//! Rulings batch P183 — searching libraries: what a search finds (CR 701.23), cards
//! exiled face up (CR 406.3), and a Two-Headed Giant drain (CR 810.9).

use crate::r_p009_common::{obj, pl};
use crate::r_p189_common::set_library;
use crate::r_s01_common::{attack_with, supported};
use mtg_engine::decision::Decision;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::player_control::can_see;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The candidates of the last card choice `p` was asked.
fn last_choice(t: &TestGame, p: PlayerId) -> Vec<Entity> {
    t.asked()
        .iter()
        .rev()
        .find_map(|(q, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *q == p => Some(candidates.clone()),
            _ => None,
        })
        .expect("no card choice")
}

/// P0's ETB search of `searcher` over a library of `library` (top first): the cards
/// (by name) it may find.
fn findable(searcher: &str, library: &[&str]) -> Vec<String> {
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, library);
    t.answer_yes(P0, true);
    t.enter(P0, searcher);
    t.resolve_all();
    let mut names: Vec<String> = last_choice(&t, P0)
        .iter()
        .filter_map(|e| match e {
            Entity::Object(o) if lib.contains(o) => Some(t.g.obj(*o).chars.name.to_string()),
            _ => None,
        })
        .collect();
    names.sort();
    names
}

#[test]
fn search_filters_find_exactly_the_described_cards() {
    cr!("701.23a");
    ruling!(
        "Sphinx Summoner",
        "The card you find must be both an artifact and a creature."
    );
    ruling!(
        "Trophy Mage",
        "Trophy Mage's ability finds an artifact card with mana value exactly 3."
    );
    ruling!(
        "Mwonvuli Beast Tracker",
        "The creature card must have one of the listed abilities, not an ability that grants one of the listed abilities to the creature."
    );
    for c in [
        "Sphinx Summoner",
        "Trophy Mage",
        "Mwonvuli Beast Tracker",
        "Prized Elephant",
        "Coalition Relic",
    ] {
        supported(c);
    }
    assert_eq!(
        findable(
            "Sphinx Summoner",
            &["Mind Stone", "Grizzly Bears", "Ornithopter"]
        ),
        vec!["Ornithopter"]
    );
    assert_eq!(
        findable(
            "Trophy Mage",
            &["Mind Stone", "Coalition Relic", "Icy Manipulator", "Hill Giant"]
        ),
        vec!["Coalition Relic"]
    );
    // Prized Elephant only has "{G}: ~ gains trample until end of turn".
    assert_eq!(
        findable(
            "Mwonvuli Beast Tracker",
            &["Prized Elephant", "Giant Spider", "Craw Wurm"]
        ),
        vec!["Giant Spider"]
    );
}

#[test]
fn hoarding_dragon_exiles_the_artifact_face_up() {
    cr!("406.3", "701.23a");
    ruling!(
        "Hoarding Dragon",
        "The artifact card you find is exiled face up. All players can see what it is."
    );
    supported("Hoarding Dragon");
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Grizzly Bears", "Mind Stone"]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(lib[1])]);
    t.enter(P0, "Hoarding Dragon");
    t.resolve_all();
    let stone = t.g.find_in_zone(Zone::Exile, "Mind Stone");
    assert_eq!(stone.len(), 1);
    assert!(!t.g.obj(stone[0]).face_down);
    assert!(can_see(&t.g, P0, stone[0]));
    assert!(can_see(&t.g, P1, stone[0]));
}

#[test]
fn earwig_squad_exiles_the_cards_face_up() {
    cr!("406.3", "702.76a");
    ruling!(
        "Earwig Squad",
        "The cards are exiled face up. All players will know what they are."
    );
    supported("Earwig Squad");
    let mut t = TestGame::new(2);
    let lib = set_library(
        &mut t,
        P1,
        &["Grizzly Bears", "Lightning Bolt", "Forest", "Mind Stone"],
    );
    let rogue = t.battlefield(P0, "Bane Alley Blackguard");
    attack_with(&mut t, &[(rogue, pl(P1))]);
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.life(P1), 19);
    t.lands(P0, "Swamp", 3);
    let squad = t.hand(P0, "Earwig Squad");
    t.cast(P0, squad)
        .method(CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Prowl))
        .go();
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_choose(P0, &[obj(lib[0]), obj(lib[1]), obj(lib[3])]);
    t.resolve_all();
    for i in [0, 1, 3] {
        let c = t.g.current(lib[i]);
        assert_eq!(t.g.obj(c).zone, Zone::Exile);
        assert_eq!(t.g.obj(c).owner, P1);
        assert!(!t.g.obj(c).face_down);
        assert!(can_see(&t.g, P0, c) && can_see(&t.g, P1, c));
    }
}

#[test]
fn forerunner_drains_the_opposing_team_for_2_in_two_headed_giant() {
    cr!("810.9");
    ruling!(
        "Forerunner of the Coalition",
        "In a Two-Headed Giant game, the last ability of Forerunner of the Coalition causes the opposing team to lose 2 life."
    );
    supported("Forerunner of the Coalition");
    let mut t = TestGame::with_config(
        4,
        mtg_engine::game::GameConfig {
            variant: mtg_engine::game::Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    assert_eq!(t.life(P2), 30);
    t.battlefield(P0, "Forerunner of the Coalition");
    t.enter(P0, "Talas Merchant");
    t.resolve_all();
    assert_eq!((t.life(P2), t.life(P3)), (28, 28));
    assert_eq!(t.life(P0), 30);
}
