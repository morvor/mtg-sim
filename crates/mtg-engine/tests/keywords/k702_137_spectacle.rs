//! CR 702.137 Spectacle.

use crate::common_k702_125_139::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const SPECTACLE: CastMethod = CastMethod::Keyword(KeywordKind::Spectacle);

#[test]
fn spectacle_cost_can_be_paid_after_an_opponent_lost_life() {
    cr!("702.137", "702.137a");
    ruling!(
        "Light Up the Stage",
        "The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    assert_supported_card("Light Up the Stage");
    let mut t = TestGame::new(2);
    // Light Up the Stage: {2}{R} sorcery, spectacle {R}.
    t.lands(P0, "Mountain", 1);
    let lus = t.hand(P0, "Light Up the Stage");
    assert!(!castable(&mut t, P0, lus, SPECTACLE));
    t.g.lose_life(P1, 1);
    assert!(castable(&mut t, P0, lus, SPECTACLE));
    let spell = t.cast(P0, lus).method(SPECTACLE).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.g.mana_value_of(spell), 3);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Light Up the Stage"));
}

#[test]
fn damage_to_an_opponent_counts_and_regaining_the_life_doesnt_matter() {
    cr!("702.137a");
    ruling!(
        "Light Up the Stage",
        "Spectacle cares only that an opponent lost life during the turn, not that the opponent's life total is currently lower than it was."
    );
    ruling!(
        "Light Up the Stage",
        "Damage dealt to a player causes that player to lose that much life."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let bolt = t.hand(P0, "Lightning Bolt");
    let skewer = t.hand(P0, "Skewer the Critics");
    assert!(!castable(&mut t, P0, skewer, SPECTACLE));
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    t.g.gain_life(P1, 10);
    assert_eq!(t.life(P1), 27);
    assert!(castable(&mut t, P0, skewer, SPECTACLE));
    t.cast(P0, skewer)
        .method(SPECTACLE)
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    assert_eq!(t.life(P1), 24);
}

#[test]
fn only_an_opponents_life_loss_counts_and_only_this_turn() {
    cr!("702.137a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let lus = t.hand(P0, "Light Up the Stage");
    t.g.lose_life(P0, 3);
    assert!(!castable(&mut t, P0, lus, SPECTACLE));
    t.g.lose_life(P1, 1);
    assert!(castable(&mut t, P0, lus, SPECTACLE));
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(!castable(&mut t, P0, lus, SPECTACLE));
}

#[test]
fn an_opponent_who_lost_life_and_left_the_game_still_counts() {
    cr!("702.137a");
    ruling!(
        "Light Up the Stage",
        "In a multiplayer game, if an opponent loses life and later that turn leaves the game, you can cast a spell for its spectacle cost."
    );
    let mut t = TestGame::new(3);
    t.lands(P0, "Mountain", 1);
    let lus = t.hand(P0, "Light Up the Stage");
    t.g.lose_life(P2, 2);
    t.g.perform_action(P2, mtg_engine::decision::Action::Concede)
        .unwrap();
    t.settle();
    assert!(!t.g.player(P2).in_game());
    assert!(castable(&mut t, P0, lus, SPECTACLE));
}

#[test]
fn spectacle_doesnt_change_when_the_spell_can_be_cast() {
    cr!("702.137a");
    ruling!(
        "Light Up the Stage",
        "Spectacle doesn't change when you can cast the spell."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let lus = t.hand(P0, "Light Up the Stage");
    t.set_step(P1, Step::PrecombatMain);
    t.g.lose_life(P1, 1);
    assert!(!castable(&mut t, P0, lus, SPECTACLE));
}

#[test]
fn if_its_spectacle_cost_was_paid() {
    cr!("702.137a");
    assert_supported_card("Rafter Demon");
    // Rafter Demon: "When this creature enters, if its spectacle cost was paid, each
    // opponent discards a card."
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Mountain", 3);
    t.hand(P1, "Grizzly Bears");
    let demon = t.hand(P0, "Rafter Demon");
    t.cast(P0, demon).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 1);
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Mountain", 3);
    t.hand(P1, "Grizzly Bears");
    t.g.lose_life(P1, 1);
    let demon = t.hand(P0, "Rafter Demon");
    t.cast(P0, demon).method(SPECTACLE).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
}

#[test]
fn a_spell_cast_for_its_spectacle_cost_keeps_its_colors() {
    cr!("702.137a");
    ruling!(
        "Rix Maadi Reveler",
        "Rix Maadi Reveler is only red. It's not black, even if you cast it for its spectacle cost."
    );
    assert_supported_card("Rix Maadi Reveler");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 3);
    t.g.lose_life(P1, 1);
    for _ in 0..2 {
        t.hand(P0, "Grizzly Bears");
    }
    let rix = t.hand(P0, "Rix Maadi Reveler");
    let spell = t.cast(P0, rix).method(SPECTACLE).go();
    let colors = t.g.obj(spell).chars.colors;
    assert!(colors.contains(Color::Red) && !colors.contains(Color::Black));
    t.resolve_all();
    // Its spectacle cost was paid: discard your hand, then draw three cards.
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(
        graveyard(&t, P0)
            .iter()
            .filter(|n| *n == "Grizzly Bears")
            .count(),
        2
    );
}
