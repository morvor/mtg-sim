//! One coin flip for each object or player (CR 705), "skips their next X turns", and one
//! die roll for each player being attacked (CR 706.6).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn flip_a_coin_for_each_creature_destroy_those_whose_coin_came_up_tails() {
    cr!("705.2");
    compiles("Rakdos, the Showstopper");
    ruling!(
        "Rakdos, the Showstopper",
        "All of the creatures whose coins come up tails are destroyed at the same time."
    );
    // "When Rakdos enters, flip a coin for each creature that isn't a Demon, Devil, or
    // Imp. Destroy each creature whose coin comes up tails."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    // A Demon gets no flip.
    let demon = t.battlefield(P1, "Lord of the Pit");
    // Three flips, in some order: two tails and one heads.
    t.g.dice.loaded_coins.extend([false, true, false]);
    let rakdos = t.enter(P0, "Rakdos, the Showstopper");
    t.resolve_all();
    assert!(t.on_battlefield(rakdos));
    assert!(t.on_battlefield(demon));
    let survivors = [bears, giant, elves]
        .iter()
        .filter(|id| t.on_battlefield(**id))
        .count();
    assert_eq!(survivors, 1);
    assert!(t.g.dice.loaded_coins.is_empty());
    // The creatures were destroyed at the same time: one event batch.
    let destroyed: Vec<usize> = t
        .g
        .turn_events
        .iter()
        .enumerate()
        .filter(|(_, e)| matches!(e, mtg_engine::events::Event::Destroyed { .. }))
        .map(|(i, _)| i)
        .collect();
    assert_eq!(destroyed.len(), 2);
    let between = &t.g.turn_events[destroyed[0]..destroyed[1]];
    assert!(!between
        .iter()
        .any(|e| matches!(e, mtg_engine::events::Event::BatchBoundary)));
}

#[test]
fn for_each_blocking_creature_flip_a_coin() {
    cr!("705.2");
    compiles("Fighting Chance");
    // "For each blocking creature, flip a coin. If you win the flip, prevent all combat
    // damage that would be dealt by that creature this turn."
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let attacker2 = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Mountain", 1);
    t.set_step(P0, Step::PrecombatMain);
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![
            (giant, Entity::Player(P1)),
            (attacker2, Entity::Player(P1)),
        ]),
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(bears, giant), (elves, attacker2)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    // Win the first flip (call heads, heads), lose the second.
    t.g.dice.loaded_coins.extend([true, false]);
    let spell = t.hand(P0, "Fighting Chance");
    t.cast(P0, spell).go();
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    let damage = [giant, attacker2].map(|a| t.obj_now(a).damage);
    // Exactly one of the blockers' damage was prevented.
    let mut d = damage.to_vec();
    d.sort();
    assert_eq!(d, vec![0, 1], "{damage:?}");
}

#[test]
fn flip_a_coin_for_each_opponent() {
    cr!("705.2");
    compiles("Mutalith Vortex Beast");
    ruling!(
        "Mutalith Vortex Beast",
        "deals 3 damage to the appropriate player for each lost flip"
    );
    // "When this creature enters, flip a coin for each opponent you have. For each flip you
    // win, draw a card. For each flip you lose, this creature deals 3 damage to that
    // player."
    let mut t = TestGame::new(3);
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    // The flip for P1 wins, the flip for P2 loses.
    t.g.dice.loaded_coins.extend([true, false]);
    let before = t.hand_size(P0);
    t.enter(P0, "Mutalith Vortex Beast");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), before + 1);
    assert_eq!((t.life(P1), t.life(P2)), (20, 17));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn skips_their_next_x_turns() {
    cr!("705.2", "500.11");
    compiles("Ral Zarek, Guest Lecturer");
    // "−7: Flip five coins. Target opponent skips their next X turns, where X is the
    // number of coins that came up heads."
    let mut t = TestGame::new(2);
    let ral = t.battlefield(P0, "Ral Zarek, Guest Lecturer");
    t.g.objects[ral.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 7);
    t.set_step(P0, Step::PrecombatMain);
    t.g.dice
        .loaded_coins
        .extend([true, false, true, false, false]);
    t.activate(P0, ral, 3, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    // P1 skips two turns: P0 takes the next three turns.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.g.turn.number, 4);
}

#[test]
fn roll_a_d20_for_each_player_being_attacked_and_ignore_all_but_the_highest() {
    cr!("706.6");
    compiles("Iron Mastiff");
    // "Whenever this creature attacks, roll a d20 for each player being attacked and
    // ignore all but the highest roll. 1—9 | This creature deals damage equal to its power
    // to you. 10—19 | ... to defending player. 20 | ... to each opponent."
    let mut t = TestGame::new(3);
    let mastiff = t.battlefield(P0, "Iron Mastiff");
    let other = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::PrecombatMain);
    // Two players are attacked: two dice, the highest (12) counts.
    t.g.dice.loaded.extend([3, 12]);
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![
            (mastiff, Entity::Player(P1)),
            (other, Entity::Player(P2)),
        ]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 16);
    assert!(t.g.dice.loaded.is_empty());
}
