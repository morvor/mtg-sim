//! CR 102: players — the active player, opponents, teammates, and "your team".

use crate::r100_common::*;
use mtg_engine::game::GameConfig;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn the_active_player_is_the_player_whose_turn_it_is() {
    cr!("102.1");
    // Howling Mine: "At the beginning of each player's draw step, if this artifact is
    // untapped, that player draws an additional card." — "that player" is the active
    // player of the draw step.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Howling Mine");
    t.set_step(P0, Step::End);
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    go_to(&mut t, P1, Step::Draw);
    assert_eq!(t.g.active_player(), P1);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), h1 + 2, "draw step draw plus Howling Mine");
    assert_eq!(t.hand_size(P0), h0);
    // The nonactive players are the others, after the active player in APNAP order.
    assert_eq!(t.g.apnap(), vec![P1, P0]);
}

#[test]
fn in_a_two_player_game_the_opponent_is_the_other_player() {
    cr!("102.2");
    let mut t = TestGame::new(2);
    assert_eq!(t.g.opponents(P0), vec![P1]);
    assert_eq!(t.g.opponents(P1), vec![P0]);
    // Guttersnipe: "Whenever you cast an instant or sorcery spell, this creature deals 2
    // damage to each opponent."
    t.battlefield(P0, "Guttersnipe");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn in_a_team_game_opponents_are_the_players_not_on_your_team() {
    cr!("102.3");
    let mut t = team_game(&[0, 1, 0, 1], GameConfig::default());
    assert_eq!(t.g.teammates(P0), vec![P2]);
    assert_eq!(t.g.opponents(P0), vec![P1, P3]);
    t.battlefield(P0, "Guttersnipe");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.life(P3), 18);
    assert_eq!(t.life(P2), 20, "a teammate isn't an opponent");
}

#[test]
fn your_team_means_you_and_your_teammates() {
    cr!("102.4");
    let haste = |t: &TestGame, id: ObjectId| t.obj_now(id).has_keyword(KeywordKind::Haste);
    // "Warriors your team controls have haste."
    let mut t = team_game(&[0, 1, 0, 1], GameConfig::default());
    t.battlefield(P0, "Rushblade Commander");
    let own = t.battlefield(P0, "Fearless Halberdier");
    let mate = t.battlefield(P2, "Fearless Halberdier");
    let foe = t.battlefield(P1, "Fearless Halberdier");
    t.g.recompute();
    assert!(haste(&t, own) && haste(&t, mate));
    assert!(!haste(&t, foe));
    // Not a game between teams: "your team" means "you".
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Rushblade Commander");
    let own = t.battlefield(P0, "Fearless Halberdier");
    let other = t.battlefield(P2, "Fearless Halberdier");
    t.g.recompute();
    assert!(haste(&t, own));
    assert!(!haste(&t, other));
}
