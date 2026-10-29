//! Choosing cards from another player's revealed or looked-at hand and graveyard (CR
//! 701.9b, 701.20a, 608.2d), and conditions about a chosen player.

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

/// The candidates of the first choice `p` was asked to make since `from`.
fn first_choice(t: &TestGame, from: usize, p: PlayerId) -> Vec<Entity> {
    t.asked()[from..]
        .iter()
        .find_map(|(q, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *q == p => Some(candidates.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

#[test]
fn never_happened_a_nonland_card_from_graveyard_or_hand() {
    cr!("608.2d", "701.20a");
    compiles("Never Happened");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let bears = t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Forest");
    let bolt = t.graveyard(P1, "Lightning Bolt");
    t.graveyard(P1, "Mountain");
    let spell = t.hand(P0, "Never Happened");
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    let from = t.asked().len();
    t.cast(P0, spell).target(P1).go();
    t.resolve();
    let mut cands = first_choice(&t, from, P0);
    cands.sort_by_key(|e| format!("{e:?}"));
    let mut want = vec![Entity::Object(bears), Entity::Object(bolt)];
    want.sort_by_key(|e| format!("{e:?}"));
    assert_eq!(cands, want, "nonland cards in that player's hand and graveyard");
    assert!(t.in_exile("Lightning Bolt"));
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn agonizing_remorse_a_nonland_card_from_it_or_a_card_from_their_graveyard() {
    cr!("608.2d", "701.20a");
    compiles("Agonizing Remorse");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let bears = t.hand(P1, "Grizzly Bears");
    let forest = t.hand(P1, "Forest");
    let mountain = t.graveyard(P1, "Mountain");
    let spell = t.hand(P0, "Agonizing Remorse");
    t.answer_choose(P0, &[Entity::Object(mountain)]);
    let from = t.asked().len();
    t.cast(P0, spell).target(P1).go();
    t.resolve();
    let cands = first_choice(&t, from, P0);
    assert!(cands.contains(&Entity::Object(bears)));
    assert!(cands.contains(&Entity::Object(mountain)), "any card from the graveyard");
    assert!(!cands.contains(&Entity::Object(forest)), "a nonland card from the hand");
    assert!(t.in_exile("Mountain"));
    assert_eq!(t.life(P0), 19);
}

#[test]
fn dreams_of_steel_and_oil_one_from_the_hand_then_one_from_the_graveyard() {
    cr!("608.2d", "701.20a");
    compiles("Dreams of Steel and Oil");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    t.hand(P1, "Grizzly Bears");
    t.graveyard(P1, "Ornithopter");
    t.graveyard(P1, "Lightning Bolt");
    let spell = t.hand(P0, "Dreams of Steel and Oil");
    t.cast(P0, spell).target(P1).go();
    t.resolve();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_exile("Ornithopter"));
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
}

#[test]
fn extortion_look_at_a_hand_and_choose_up_to_two_cards() {
    cr!("701.20a", "701.9b");
    compiles("Extortion");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    let a = t.hand(P1, "Grizzly Bears");
    let b = t.hand(P1, "Forest");
    t.hand(P1, "Lightning Bolt");
    let spell = t.hand(P0, "Extortion");
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.cast(P0, spell).target(P1).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Forest"));
    assert!(t.in_hand(P1, "Lightning Bolt"));
}

#[test]
fn thought_stalker_warlock_if_they_lost_life_this_turn() {
    cr!("608.2d", "701.9b");
    compiles("Thought-Stalker Warlock");
    // P1 lost life this turn: P0 chooses the card.
    let mut t = TestGame::new(2);
    t.g.players[1].life -= 1;
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve();
    let bears = t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Forest");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let from = t.asked().len();
    t.enter(P0, "Thought-Stalker Warlock");
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(first_choice(&t, from, P0), vec![Entity::Object(bears)]);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_hand(P1, "Forest"));
    // Otherwise, they discard a card of their choice.
    let mut t = TestGame::new(2);
    t.hand(P1, "Grizzly Bears");
    let forest = t.hand(P1, "Forest");
    t.answer_choose(P1, &[Entity::Object(forest)]);
    t.enter(P0, "Thought-Stalker Warlock");
    t.g.flush_events();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Forest"));
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn biting_palm_ninja_reflexive_choice_then_exile_that_card() {
    cr!("603.12", "608.2d");
    compiles("Biting-Palm Ninja");
    let mut t = TestGame::new(2);
    let ninja = t.battlefield(P0, "Biting-Palm Ninja");
    t.g.objects[ninja.0 as usize]
        .counters
        .insert("menace".into(), 1);
    let bears = t.hand(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.set_step(P0, Step::PrecombatMain);
    t.attack(&[(ninja, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.counters(ninja, "menace"), 0);
}
