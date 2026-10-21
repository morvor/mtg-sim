//! Trigger grammar: "a counter is put on" (any kind, Hollowmurk Siege), "one or more of
//! your opponents are attacked" (Party Dude), "When you cast a spell with the chosen name
//! for the first time this turn" (Medomai's Prophecy), "Whenever that creature is dealt
//! damage by an attacking creature this turn" (Glyph of Life).

use crate::combat2_common::*;
use crate::td_next_turn_durations::mana;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn hollowmurk_siege_sultai_draws_once_each_turn_for_any_counter() {
    cr!("122.6", "603.2");
    supported("Hollowmurk Siege");
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.enter(P0, "Hollowmurk Siege");
    t.resolve_all();
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let hand = t.hand_size(P0);
    // A counter on an opponent's creature: nothing.
    t.g.add_counters(Entity::Object(theirs), "+1/+1", 1, None);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // A stun counter (any kind) on your creature: draw.
    t.g.add_counters(Entity::Object(bears), "stun", 1, None);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Only once each turn.
    t.g.add_counters(Entity::Object(bears), "+1/+1", 1, None);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn party_dude_pumps_once_when_opponents_are_attacked() {
    cr!("508.3e", "603.2c");
    supported("Party Dude");
    let mut t = TestGame::new(3);
    let dude = t.battlefield(P0, "Party Dude");
    mtg_engine::classes::set_level(&mut t.g, dude, 3);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.hand(P0, "Island");
    t.hand(P0, "Island");
    // Attacking both opponents at once: one trigger.
    t.answer_targets(P0, &[Entity::Object(a)]);
    t.answer_targets(P0, &[Entity::Object(b)]);
    declare(&mut t, P0, &[(a, Entity::Player(P1)), (b, Entity::Player(P2))]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(a), (4, 4));
    assert_eq!(t.pt(b), (2, 2));
}

#[test]
fn medomais_prophecy_draws_for_the_first_spell_with_the_chosen_name_this_turn() {
    cr!("603.7b", "607.2a");
    supported("Medomai's Prophecy");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Medomai's Prophecy");
    // Chapter I (scry), chapter II: choose "Lightning Bolt"; chapter III sets up the
    // delayed ability.
    t.answer(P0, DecisionKind::Name, Answer::Text("Lightning Bolt".into()));
    for _ in 0..3 {
        t.g.add_counters(Entity::Object(saga), "lore", 1, None);
        t.resolve_all();
    }
    let hand = t.hand_size(P0);
    // Another name: nothing.
    let growth = t.hand(P0, "Giant Growth");
    let bears = t.battlefield(P0, "Grizzly Bears");
    mana(&mut t, P0);
    t.cast_with(P0, growth, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // The first Lightning Bolt: draw two; the second: nothing.
    for (i, draws) in [(0, 2), (1, 0)] {
        let bolt = t.hand(P0, "Lightning Bolt");
        let before = t.hand_size(P0);
        mana(&mut t, P0);
        t.cast_with(P0, bolt, &[Entity::Player(P1)]).unwrap();
        t.resolve_all();
        assert_eq!(t.hand_size(P0), before - 1 + draws, "bolt {i}");
    }
    // Next turn: over.
    t.advance_to(P1, Step::Upkeep);
    let bolt = t.hand(P0, "Lightning Bolt");
    let before = t.hand_size(P0);
    mana(&mut t, P0);
    t.cast_with(P0, bolt, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), before - 1);
}

#[test]
fn glyph_of_life_gains_life_for_damage_attacking_creatures_deal_to_the_wall() {
    cr!("603.7c", "120.3");
    ruling!(
        "Glyph of Life",
        "This is not redirection of damage. The wall still takes the damage."
    );
    supported("Glyph of Life");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Stone");
    let bears = t.battlefield(P1, "Grizzly Bears");
    declare(&mut t, P1, &[(bears, Entity::Player(P0))]);
    let glyph = t.hand(P0, "Glyph of Life");
    mana(&mut t, P0);
    t.cast_with(P0, glyph, &[Entity::Object(wall)]).unwrap();
    t.resolve_all();
    // Damage from a source that isn't an attacking creature: no life.
    let bolt = t.hand(P0, "Lightning Bolt");
    mana(&mut t, P0);
    t.cast_with(P0, bolt, &[Entity::Object(wall)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    declare_blocks(&mut t, P0, &[(wall, bears)]);
    t.advance_to(P1, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.obj_now(wall).damage, 5);
    // This turn only: the next attack blocked by the wall gains nothing.
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    declare(&mut t, P1, &[(bears, Entity::Player(P0))]);
    declare_blocks(&mut t, P0, &[(wall, bears)]);
    t.advance_to(P1, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.obj_now(wall).damage, 2);
    assert_eq!(t.life(P0), 22);
}
