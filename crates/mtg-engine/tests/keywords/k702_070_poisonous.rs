//! CR 702.70 Poisonous.

use crate::common_k702_011_017::{assert_supported, attack_with};
use crate::common_k702_018_026::{declare_blocks, triggers_on_stack};
use crate::common_k702_052_066::run_effect;
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn poison(t: &TestGame, p: PlayerId) -> u32 {
    t.g.player(p).counter(counters::POISON)
}

#[test]
fn poisonous_gives_poison_counters_for_combat_damage_to_a_player() {
    cr!("702.70", "702.70a");
    assert_supported("Virulent Sliver");
    let mut t = TestGame::new(2);
    // Virulent Sliver: 1/1, "All Sliver creatures have poisonous 1."
    let sliver = t.battlefield(P0, "Virulent Sliver");
    attack_with(&mut t, &[(sliver, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 19);
    assert_eq!(poison(&t, P1), 1);
}

#[test]
fn poisonous_n_gives_n_counters_however_much_damage_was_dealt() {
    cr!("702.70a");
    ruling!(
        "Virulent Sliver",
        "Poisonous 1 causes the player to get just one poison counter when a Sliver deals combat damage to them, no matter how much damage that Sliver dealt."
    );
    assert_supported("Snake Cult Initiation");
    let mut t = TestGame::new(2);
    // Enchanted Grizzly Bears (2/2) has poisonous 3.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P0, "Snake Cult Initiation");
    assert!(t.g.attach(aura, Entity::Object(bears)));
    t.g.recompute();
    assert_eq!(t.obj_now(bears).chars.keyword_count(KeywordKind::Poisonous), 1);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
    assert_eq!(poison(&t, P1), 3);
}

#[test]
fn poisonous_doesnt_trigger_on_damage_to_a_creature() {
    cr!("702.70a");
    let mut t = TestGame::new(2);
    let sliver = t.battlefield(P0, "Virulent Sliver");
    let wall = t.battlefield(P1, "Wall of Wood");
    attack_with(&mut t, &[(sliver, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(wall, sliver)]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.obj_now(wall).damage, 1);
    assert_eq!(poison(&t, P1), 0);
}

#[test]
fn poisonous_doesnt_trigger_on_noncombat_damage() {
    cr!("702.70a");
    let mut t = TestGame::new(2);
    let sliver = t.battlefield(P0, "Virulent Sliver");
    run_effect(
        &mut t,
        None,
        P0,
        Effect::DealDamage {
            source: Sel::Target(0),
            amount: Value::c(1),
            to: Sel::Players(PlayerRef::Player(P1)),
        },
        &[Entity::Object(sliver)],
    );
    t.settle();
    assert_eq!(t.life(P1), 19);
    assert_eq!(triggers_on_stack(&t, "Poisonous 1"), 0);
    assert_eq!(poison(&t, P1), 0);
}

#[test]
fn each_instance_of_poisonous_triggers_separately() {
    cr!("702.70b");
    ruling!(
        "Virulent Sliver",
        "If a creature has multiple instances of poisonous, each triggers separately."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Virulent Sliver");
    t.battlefield(P0, "Virulent Sliver");
    assert_eq!(t.obj_now(a).chars.keyword_count(KeywordKind::Poisonous), 2);
    attack_with(&mut t, &[(a, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[]);
    t.advance_to_step(Step::CombatDamage);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Poisonous 1"), 2);
    t.resolve_all();
    assert_eq!(poison(&t, P1), 2);
}

#[test]
fn ten_poison_counters_from_poisonous_lose_the_game() {
    cr!("702.70a", "704.5c");
    ruling!(
        "Virulent Sliver",
        "A player with ten or more poison counters loses the game as a state-based action"
    );
    let mut t = TestGame::new(2);
    let sliver = t.battlefield(P0, "Virulent Sliver");
    t.g.players[1].counters.insert(counters::POISON.into(), 9);
    attack_with(&mut t, &[(sliver, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[]);
    t.advance_to_step(Step::CombatDamage);
    t.resolve_all();
    assert_eq!(poison(&t, P1), 10);
    assert!(t.has_lost(P1));
}

#[test]
fn a_virulent_sliver_that_isnt_a_sliver_doesnt_have_poisonous() {
    cr!("702.70a");
    ruling!(
        "Virulent Sliver",
        "If the creature type of a Sliver changes so it's no longer a Sliver, it will no longer be affected by its own ability."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Virulent Sliver");
    run_effect(
        &mut t,
        None,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveAllCreatureTypes],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(a)],
    );
    assert_eq!(t.obj_now(a).chars.keyword_count(KeywordKind::Poisonous), 0);
    attack_with(&mut t, &[(a, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 19);
    assert_eq!(poison(&t, P1), 0);
}
