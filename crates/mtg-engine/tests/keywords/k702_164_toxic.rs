//! CR 702.164 Toxic.

use crate::common_k702_153_167::*;
use mtg_engine::ability::*;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::*;

/// The N of each toxic ability the card has as printed.
fn printed_toxic(name: &str) -> Vec<i32> {
    card(name).faces[0]
        .chars
        .keywords()
        .filter(|k| k.kind == KeywordKind::Toxic)
        .map(|k| k.n.unwrap_or(0))
        .collect()
}

#[test]
fn toxic_is_written_toxic_n() {
    cr!("702.164", "702.164a");
    assert_supported("Tyrranax Atrocity");
    assert_eq!(printed_toxic("Tyrranax Atrocity"), vec![3]);
    // In a keyword list: "Lifelink, toxic 1".
    assert_eq!(printed_toxic("Venser, Corpse Puppet"), vec![1]);
    assert_eq!(printed_toxic("Sheoldred's Headcleaver"), vec![2]);
}

#[test]
fn combat_damage_to_a_player_also_gives_poison_counters() {
    cr!("702.164c");
    ruling!(
        "Tyrranax Atrocity",
        "The results of that damage are the player loses 2 life and gets a poison counter."
    );
    // Tyrranax Atrocity: 4/4 haste, toxic 3.
    let mut t = TestGame::new(2);
    let dino = t.battlefield(P0, "Tyrranax Atrocity");
    t.attack(&[(dino, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 16);
    assert_eq!(poison(&t, P1), 3);
    assert_eq!(poison(&t, P0), 0);
}

#[test]
fn toxic_does_nothing_for_damage_to_creatures_or_noncombat_damage() {
    cr!("702.164c");
    ruling!(
        "Tyrranax Atrocity",
        "If a creature with toxic deals combat damage to a creature or planeswalker, or if it deals noncombat damage, toxic has no effect and no player gets poison counters."
    );
    // Blocked: combat damage to a creature.
    let mut t = TestGame::new(2);
    let dino = t.battlefield(P0, "Tyrranax Atrocity");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.attack(&[(dino, Entity::Player(P1))], &[(wall, dino)]);
    assert_eq!(poison(&t, P1), 0);
    assert_eq!(t.life(P1), 20);
    // Noncombat damage dealt by the creature to a player.
    let mut t = TestGame::new(2);
    let dino = t.battlefield(P0, "Tyrranax Atrocity");
    run_effect(
        &mut t,
        Some(dino),
        P0,
        Effect::DealDamage {
            source: Sel::This,
            amount: Value::c(2),
            to: Sel::Target(0),
        },
        &[Entity::Player(P1)],
    );
    assert_eq!(t.life(P1), 18);
    assert_eq!(poison(&t, P1), 0);
}

#[test]
fn toxic_counts_once_per_combat_damage_regardless_of_amount() {
    cr!("702.164c");
    ruling!(
        "Tyrranax Atrocity",
        "Damage dealt by a creature with toxic grants the same number of counters regardless of how much damage is dealt."
    );
    ruling!(
        "Tyrranax Atrocity",
        "Any other effects of that damage, such as life gain from lifelink, still apply."
    );
    // Gratuitous Violence doubles the damage, not the poison counters.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gratuitous Violence");
    let dino = t.battlefield(P0, "Tyrranax Atrocity");
    t.attack(&[(dino, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 12);
    assert_eq!(poison(&t, P1), 3);
    // Venser, Corpse Puppet: lifelink, toxic 1: both results.
    let mut t = TestGame::new(2);
    let venser = t.battlefield(P0, "Venser, Corpse Puppet");
    t.attack(&[(venser, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 21);
    assert_eq!(poison(&t, P1), 1);
    // Double strike deals combat damage twice: toxic applies each time.
    let mut t = TestGame::new(2);
    let duelist = t.battlefield(P0, "Jawbone Duelist");
    t.attack(&[(duelist, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 18);
    assert_eq!(poison(&t, P1), 2);
}

#[test]
fn total_toxic_value_is_the_sum_of_its_toxic_abilities() {
    cr!("702.164b", "702.164c");
    ruling!(
        "Tyrranax Atrocity",
        "Multiple instances of toxic are cumulative."
    );
    assert_supported("Plague Nurse");
    // Plague Nurse (toxic 2) gains toxic 1: total toxic value 3.
    let mut t = TestGame::new(2);
    let nurse = t.battlefield(P0, "Plague Nurse");
    gain(&mut t, P0, nurse, Keyword::with_n(KeywordKind::Toxic, 1));
    assert_eq!(kw_count(&t, nurse, KeywordKind::Toxic), 2);
    t.attack(&[(nurse, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 17);
    assert_eq!(poison(&t, P1), 3);
    // Plague Nurse's ability: "{2}{G}: Each other creature you control with toxic gains
    // toxic 1 until end of turn."
    let mut t = TestGame::new(2);
    let nurse = t.battlefield(P0, "Plague Nurse");
    let dino = t.battlefield(P0, "Tyrranax Atrocity");
    t.lands(P0, "Forest", 3);
    t.activate(P0, nurse, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(kw_count(&t, dino, KeywordKind::Toxic), 2);
    t.attack(&[(dino, Entity::Player(P1))], &[]);
    assert_eq!(poison(&t, P1), 4);
}

#[test]
fn toxic_and_infect_both_give_poison_counters() {
    cr!("702.164c");
    // An infect creature that gains toxic 2: its damage to a player is given as poison
    // counters (CR 702.90b), plus its total toxic value.
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P0, "Glistener Elf");
    gain(&mut t, P0, elf, Keyword::with_n(KeywordKind::Toxic, 2));
    t.attack(&[(elf, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 20);
    assert_eq!(poison(&t, P1), 3);
}

#[test]
fn toxic_can_give_a_player_the_poison_counters_that_lose_the_game() {
    cr!("702.164c");
    ruling!(
        "Tyrranax Atrocity",
        "A player with ten or more poison counters loses the game. This is a state-based action and doesn't use the stack."
    );
    let mut t = TestGame::new(2);
    t.g.players[P1.idx()]
        .counters
        .insert(mtg_engine::types::counters::POISON.into(), 7);
    let dino = t.battlefield(P0, "Tyrranax Atrocity");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(dino, Entity::Player(P1))]),
    );
    t.set_step(P0, mtg_engine::turn::Step::BeginningOfCombat);
    // The game ends as soon as state-based actions are checked after combat damage.
    let ended = t.g.run_until(10_000, |g| g.player(P1).has_lost);
    assert!(ended);
    assert_eq!(poison(&t, P1), 10);
    assert_eq!(t.life(P1), 16);
}
