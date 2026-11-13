//! CR 702.105 Dethrone.

use crate::common_k702_011_017::{assert_supported, attack_with, bf, custom_card};
use crate::common_k702_018_026::{declare_blocks, triggers_on_stack};
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn dethrone_triggers_attacking_a_player_tied_for_most_life() {
    cr!("702.105", "702.105a");
    ruling!(
        "Marchesa's Emissary",
        "The +1/+1 counter is put on the creature before blockers are declared."
    );
    assert_supported("Marchesa's Emissary");
    let mut t = TestGame::new(2);
    // Marchesa's Emissary: 2/2 hexproof, dethrone. Both players have 20 life.
    let emissary = t.battlefield(P0, "Marchesa's Emissary");
    let wall = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(emissary, Entity::Player(P1))]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Dethrone"), 1);
    t.resolve_all();
    assert_eq!(t.counters(emissary, counters::PLUS1), 1);
    // Before blockers: the 3/3 survives a block by a 2/2.
    declare_blocks(&mut t, P1, &[(wall, emissary)]);
    t.advance_to(P0, turn::Step::EndOfCombat);
    assert!(t.on_battlefield(emissary));
    assert!(!t.on_battlefield(wall));
}

#[test]
fn dethrone_doesnt_trigger_attacking_a_player_with_less_life_than_another() {
    cr!("702.105a");
    let mut t = TestGame::new(3);
    let emissary = t.battlefield(P0, "Marchesa's Emissary");
    t.g.players[P1.idx()].life = 15;
    t.g.players[P2.idx()].life = 18;
    // The attacking player's own life total counts too: P0 has the most life, 20.
    attack_with(&mut t, &[(emissary, Entity::Player(P2))]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Dethrone"), 0);
    // Tied with P0 for the most life, P2 is dethroned.
    let mut t = TestGame::new(3);
    let emissary = t.battlefield(P0, "Marchesa's Emissary");
    t.g.players[P1.idx()].life = 15;
    t.g.players[P0.idx()].life = 18;
    t.g.players[P2.idx()].life = 18;
    attack_with(&mut t, &[(emissary, Entity::Player(P2))]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Dethrone"), 1);
}

#[test]
fn dethrone_doesnt_trigger_attacking_a_planeswalker() {
    cr!("702.105a");
    ruling!(
        "Marchesa's Emissary",
        "Dethrone doesn't trigger if the creature attacks a planeswalker, even if its controller has the most life."
    );
    let mut t = TestGame::new(2);
    let emissary = t.battlefield(P0, "Marchesa's Emissary");
    let jace = t.battlefield(P1, "Jace Beleren");
    t.g.players[P1.idx()].life = 30;
    attack_with(&mut t, &[(emissary, Entity::Object(jace))]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Dethrone"), 0);
}

#[test]
fn once_dethrone_triggers_life_totals_dont_matter() {
    cr!("702.105a");
    ruling!(
        "Marchesa's Emissary",
        "Once dethrone triggers, it doesn't matter what happens to the players' life totals before the ability resolves."
    );
    let mut t = TestGame::new(2);
    let emissary = t.battlefield(P0, "Marchesa's Emissary");
    attack_with(&mut t, &[(emissary, Entity::Player(P1))]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Dethrone"), 1);
    // In response, P1 loses life.
    t.g.lose_life(P1, 5);
    t.resolve_all();
    assert_eq!(t.counters(emissary, counters::PLUS1), 1);
}

#[test]
fn each_instance_of_dethrone_triggers_separately() {
    cr!("702.105b");
    let def = custom_card(
        "Twice-Crowned Rogue",
        "Creature — Human Rogue",
        Some((1, 1)),
        "Dethrone\nDethrone",
    );
    let mut t = TestGame::new(2);
    let rogue = bf(&mut t, P0, def);
    attack_with(&mut t, &[(rogue, Entity::Player(P1))]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Dethrone"), 2);
    t.resolve_all();
    assert_eq!(t.counters(rogue, counters::PLUS1), 2);
}

#[test]
fn in_two_headed_giant_attacking_either_player_of_the_leading_team_counts() {
    cr!("702.105a");
    ruling!(
        "Marchesa's Emissary",
        "In a Two-Headed Giant game, dethrone will trigger if the creature attacks either player on the team with the most life or tied for the most life."
    );
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    // The opposing team has the most life.
    for p in [P0, P1] {
        t.g.players[p.idx()].life = 25;
    }
    for p in [P2, P3] {
        t.g.players[p.idx()].life = 30;
    }
    let a = t.battlefield(P0, "Marchesa's Emissary");
    let b = t.battlefield(P0, "Marchesa's Emissary");
    attack_with(&mut t, &[(a, Entity::Player(P2)), (b, Entity::Player(P3))]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Dethrone"), 2);
}

#[test]
fn scourge_of_the_throne_checks_the_same_condition() {
    cr!("702.105a");
    assert_supported("Scourge of the Throne");
    let mut t = TestGame::new(3);
    // "Whenever this creature attacks for the first time each turn, if it's attacking the
    // player with the most life or tied for most life, untap all attacking creatures.
    // After this phase, there is an additional combat phase."
    let scourge = t.battlefield(P0, "Scourge of the Throne");
    t.g.players[P1.idx()].life = 10;
    attack_with(&mut t, &[(scourge, Entity::Player(P1))]);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    let mut t = TestGame::new(3);
    let scourge = t.battlefield(P0, "Scourge of the Throne");
    attack_with(&mut t, &[(scourge, Entity::Player(P1))]);
    t.settle();
    // Dethrone and the untap ability.
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(!t.obj_now(scourge).tapped);
    assert_eq!(t.counters(scourge, counters::PLUS1), 1);
}
