//! Modal headers (reflexive "When you do, choose one —", "choose up to X", "Choose one.
//! Activate only ...", "Choose one. If it was kicked, choose both instead") and choices
//! made one player at a time in turn order ("Starting with you, each player chooses ...").

use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

/// The players asked to choose objects, in the order they were asked, from `from` on.
fn choosers(t: &TestGame, from: usize) -> Vec<PlayerId> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect()
}

#[test]
fn gorbag_reflexive_mode_is_chosen_only_if_the_creature_was_sacrificed() {
    cr!("603.12", "700.2");
    ruling!(
        "Gorbag of Minas Morgul",
        "You choose a mode for that ability as it goes on the stack"
    );
    compiles("Gorbag of Minas Morgul");
    // "Whenever a Goblin or Orc you control deals combat damage to a player, you may
    // sacrifice it. When you do, choose one — • Draw a card. • Create a Treasure token."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gorbag of Minas Morgul");
    let goblin = t.battlefield(P0, "Raging Goblin");
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(goblin, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(goblin));
    assert_eq!(t.named_on_battlefield("Treasure Token").len(), 1);
    // Declining to sacrifice: no mode is chosen.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gorbag of Minas Morgul");
    let goblin = t.battlefield(P0, "Raging Goblin");
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(false));
    t.set_step(P0, Step::BeginningOfCombat);
    let from = t.asked().len();
    t.attack(&[(goblin, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(t.on_battlefield(goblin));
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseModes { .. })));
}

/// The most modes Bumi's enters ability let P0 choose, with `lessons` Lesson cards in the
/// graveyard.
fn bumi_max_modes(lessons: usize) -> Option<usize> {
    let mut t = TestGame::new(2);
    for _ in 0..lessons {
        t.graveyard(P0, "Containment Breach");
    }
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.enter(P0, "Bumi, King of Three Trials");
    t.g.flush_events();
    t.settle();
    t.asked()[from..].iter().find_map(|(_, d)| match d {
        Decision::ChooseModes { max, .. } => Some(*max as usize),
        _ => None,
    })
}

#[test]
fn bumi_chooses_up_to_x_modes_where_x_counts_lessons() {
    cr!("700.2", "603.3c");
    compiles("Bumi, King of Three Trials");
    // "When Bumi enters, choose up to X, where X is the number of Lesson cards in your
    // graveyard — • Put three +1/+1 counters on Bumi. • Target player scries 3. •
    // Earthbend 3."
    assert_eq!(bumi_max_modes(2), Some(2));
    assert_eq!(bumi_max_modes(5), Some(3));
    // No Lesson in the graveyard: no mode can be chosen.
    assert!(bumi_max_modes(0).is_none_or(|m| m == 0));
    // With one, the first mode: three counters on Bumi.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Containment Breach");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    let bumi = t.enter(P0, "Bumi, King of Three Trials");
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.counters(bumi, "+1/+1"), 3);
}

#[test]
fn skinshifter_modal_ability_once_each_turn() {
    cr!("602.5b", "700.2");
    ruling!(
        "Skinshifter",
        "You can activate Skinshifter's ability during any player's turn, but only once on each turn."
    );
    compiles("Skinshifter");
    // "{G}: Choose one. Activate only once each turn. • Until end of turn, Skinshifter
    // becomes a Rhino with base power and toughness 4/4 and gains trample. • ..."
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Skinshifter");
    t.lands(P0, "Forest", 2);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.activate(P0, s, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.pt(s), (4, 4));
    assert!(t.activate(P0, s, 0, &[]).is_err());
}

#[test]
fn depth_defiler_kicked_chooses_both_modes() {
    cr!("700.2", "702.33d");
    compiles("Depth Defiler");
    // "When you cast this spell, choose one. If it was kicked, choose both instead. •
    // Return target creature to its owner's hand. • Target player draws two cards, then
    // discards a card."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 5);
    t.lands(P0, "Wastes", 1);
    let defiler = t.hand(P0, "Depth Defiler");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0, 1]));
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Player(P0)]);
    let hand = t.hand_size(P0);
    t.cast(P0, defiler).kicked(true).go();
    t.resolve();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    // Cast Defiler (-1), drew two, discarded one.
    assert_eq!(t.hand_size(P0), hand - 1 + 2 - 1);
    // Not kicked: one mode only.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 5);
    let defiler = t.hand(P0, "Depth Defiler");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0, 1]));
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Player(P0)]);
    let from = t.asked().len();
    let hand = t.hand_size(P0);
    t.cast(P0, defiler).go();
    t.resolve();
    let max = t.asked()[from..].iter().find_map(|(_, d)| match d {
        Decision::ChooseModes { min, max, .. } => Some((*min, *max)),
        _ => None,
    });
    assert_eq!(max, Some((1, 1)));
    // The invalid two-mode answer fell back to a single mode: not both happened.
    let both = !t.on_battlefield(bears) && t.hand_size(P0) == hand - 1 + 1;
    assert!(!both);
}

#[test]
fn sadistic_shell_game_starts_with_the_next_opponent() {
    cr!("101.4b", "115.10");
    ruling!(
        "Sadistic Shell Game",
        "Each player will know what choices players earlier in the turn order made."
    );
    compiles("Sadistic Shell Game");
    // "Starting with the next opponent in turn order, each player chooses a creature you
    // don't control. Destroy the chosen creatures."
    let mut t = TestGame::new(3);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P2, "Hill Giant");
    t.lands(P0, "Swamp", 5);
    t.answer_choose(P1, &[Entity::Object(b)]);
    t.answer_choose(P2, &[Entity::Object(a)]);
    t.answer_choose(P0, &[Entity::Object(a)]);
    let spell = t.hand(P0, "Sadistic Shell Game");
    let from = t.asked().len();
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(choosers(&t, from), vec![P1, P2, P0]);
    assert!(t.on_battlefield(mine));
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
}

#[test]
fn druid_of_purification_each_player_may_choose_starting_with_you() {
    cr!("101.4b");
    ruling!(
        "Druid of Purification",
        "Players make their choices in turn order and get to know what choices were made before theirs"
    );
    compiles("Druid of Purification");
    // "When this creature enters, starting with you, each player may choose an artifact or
    // enchantment you don't control. Destroy each permanent chosen this way."
    let mut t = TestGame::new(3);
    let a = t.battlefield(P1, "Ornithopter");
    let b = t.battlefield(P2, "Ornithopter");
    let keep = t.battlefield(P2, "Glorious Anthem");
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.answer_choose(P1, &[]);
    t.answer_choose(P2, &[Entity::Object(b)]);
    let from = t.asked().len();
    t.enter(P0, "Druid of Purification");
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(choosers(&t, from), vec![P0, P1, P2]);
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
    assert!(t.on_battlefield(keep));
}

#[test]
fn rejoin_the_fight_opponents_choose_different_cards() {
    cr!("101.4b");
    compiles("Rejoin the Fight");
    // "Mill three cards. Then starting with the next opponent in turn order, each opponent
    // chooses a creature card in your graveyard that hasn't been chosen. Return each card
    // chosen this way to the battlefield under your control."
    let mut t = TestGame::new(3);
    let a = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    t.lands(P0, "Swamp", 6);
    // P1 chooses the Bears; P2 can't choose them again and gets the Giant.
    t.answer_choose(P1, &[Entity::Object(a)]);
    let spell = t.hand(P0, "Rejoin the Fight");
    let from = t.asked().len();
    t.cast(P0, spell).go();
    t.resolve();
    let asked: Vec<(PlayerId, Vec<Entity>)> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some((*p, candidates.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(asked.len(), 2);
    assert_eq!(asked[0].0, P1);
    assert!(!asked[1].1.contains(&Entity::Object(a)));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn minds_aglow_join_forces_totals_the_mana_paid() {
    cr!("101.4b", "107.3");
    ruling!(
        "Minds Aglow",
        "A player can’t choose to draw fewer than X cards."
    );
    compiles("Minds Aglow");
    // "Join forces — Starting with you, each player may pay any amount of mana. Each player
    // draws X cards, where X is the total amount of mana paid this way."
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    t.lands(P1, "Island", 2);
    let spell = t.hand(P0, "Minds Aglow");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer(P1, DecisionKind::X, Answer::Number(1));
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.cast(P0, spell).go();
    let from = t.asked().len();
    t.resolve();
    // P0 was asked how much to pay before P1.
    let order: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseX { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order, vec![P0, P1]);
    assert_eq!(t.hand_size(P0), h0 - 1 + 3);
    assert_eq!(t.hand_size(P1), h1 + 3);
}
