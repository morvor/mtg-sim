//! Rulings batch P186 (review) — counts of what happened to opponents this turn include
//! opponents who have since left the game (CR 800.4i; interpretation
//! `history-counts-players-who-left`): the other wordings the interpretation rests on.

use crate::r_s01_common::supported;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn moonshae_pixie_counts_an_opponent_dealt_combat_damage_who_then_left() {
    cr!("800.4i", "603.2");
    interp!("history-counts-players-who-left");
    ruling!(
        "Moonshae Pixie // Pixie Dust",
        "If an opponent was dealt combat damage this turn and subsequently lost the game, Moonshae Pixie's second ability still counts that player when determining how many cards its controller should draw."
    );
    supported("Moonshae Pixie // Pixie Dust");
    let run = |leave: bool, combat: bool| {
        let mut t = TestGame::new(3);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.g.deal_damage(bears, Entity::Player(P1), 1, true);
        t.g.deal_damage(bears, Entity::Player(P2), 1, combat);
        t.g.flush_events();
        if leave {
            crate::r_p079_common::concede(&mut t, P2);
            assert!(t.has_lost(P2));
        }
        let hand = t.hand_size(P0);
        crate::r_s05_common::enter(&mut t, P0, "Moonshae Pixie // Pixie Dust");
        t.resolve_all();
        t.hand_size(P0) - hand
    };
    assert_eq!(run(false, true), 2);
    assert_eq!(run(true, true), 2, "the opponent who left still counts");
    // Control: noncombat damage to that opponent isn't counted.
    assert_eq!(run(true, false), 1);
}

#[test]
fn chandras_incinerator_counts_damage_to_an_opponent_who_lost() {
    cr!("800.4i", "601.2f");
    ruling!(
        "Chandra's Incinerator",
        "In a multiplayer game, Chandra's Incinerator counts damage dealt to opponents who have lost the game."
    );
    supported("Chandra's Incinerator");
    // {5}{R}, {X} less where X is the noncombat damage dealt to opponents this turn.
    let setup = |lands: usize| {
        let mut t = TestGame::new(3);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.g.deal_damage(bears, Entity::Player(P1), 1, false);
        t.g.deal_damage(bears, Entity::Player(P2), 3, false);
        t.g.flush_events();
        crate::r_p079_common::concede(&mut t, P2);
        assert!(t.has_lost(P2));
        t.lands(P0, "Mountain", lands);
        let ci = t.hand(P0, "Chandra's Incinerator");
        (t, ci)
    };
    // 4 damage counted: {1}{R}.
    let (mut t, ci) = setup(2);
    assert!(t.cast(P0, ci).try_go().is_ok());
    let (mut t, ci) = setup(1);
    assert!(t.cast(P0, ci).try_go().is_err());
}
