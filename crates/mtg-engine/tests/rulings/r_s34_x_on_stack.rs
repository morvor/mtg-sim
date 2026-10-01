//! Rulings batch S34 — a spell on the stack with {X} in its mana cost has the chosen
//! value of X in its mana value (CR 202.3e, 107.3a); everywhere else X is 0 (CR 107.3g).
//! Also each Phyrexian mana symbol contributes 1 (CR 202.3g).

use crate::r_s01_common::*;
use crate::r_s04_common::spell_targets;
use crate::r_s08_common::mana_value;
use crate::r_s28_common::energy;
use crate::r_s34_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P1 casts Blaze ({X}{R}, "Blaze deals X damage to any target.") at P0 with X = `x`, in
/// P1's main phase.
fn p1_blaze(t: &mut TestGame, x: usize) -> ObjectId {
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    t.lands(P1, "Wastes", x);
    let blaze = t.hand(P1, "Blaze");
    t.cast(P1, blaze)
        .x(x as i64)
        .target(Entity::Player(P0))
        .go()
}

/// P0 casts Blaze at P1 with X = `x`.
fn p0_blaze(t: &mut TestGame, x: usize) -> ObjectId {
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", x);
    let blaze = t.hand(P0, "Blaze");
    t.cast(P0, blaze)
        .x(x as i64)
        .target(Entity::Player(P1))
        .go()
}

#[test]
fn spell_swindle_creates_treasures_for_a_spell_s_x() {
    cr!("202.3e", "107.3a");
    ruling!(
        "Spell Swindle",
        "For spells with {X} in their mana costs, use the value chosen for X to determine the spell's mana value."
    );
    supported("Spell Swindle");
    // "Counter target spell. Create X Treasure tokens, where X is that spell's mana
    // value." Blaze with X = 3 has mana value 4.
    let mut t = TestGame::new(2);
    let blaze = p1_blaze(&mut t, 3);
    assert_eq!(mana_value(&t, blaze), 4);
    crate::r_s25_common::cast_new(&mut t, P0, "Spell Swindle", &[Entity::Object(blaze)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Blaze"));
    assert_eq!(t.life(P0), 20);
    assert_eq!(with_subtype(&t, P0, "Treasure").len(), 4);
}

#[test]
fn up_the_beanstalk_sees_x_in_a_spell_s_mana_value() {
    cr!("202.3e", "107.3a", "603.2");
    ruling!(
        "Up the Beanstalk",
        "If a spell has {X} in its mana cost, use the value chosen for that X to determine the mana value of that spell."
    );
    supported("Up the Beanstalk");
    // "... whenever you cast a spell with mana value 5 or greater, draw a card." Blaze
    // with X = 4 (mana value 5) triggers it; with X = 3 (mana value 4), it doesn't.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Up the Beanstalk");
    let hand = t.hand_size(P0);
    p0_blaze(&mut t, 4);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P1), 16);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Up the Beanstalk");
    let hand = t.hand_size(P0);
    p0_blaze(&mut t, 3);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn erratic_cyclops_gets_x_plus_one() {
    cr!("202.3e", "107.3a");
    ruling!(
        "Erratic Cyclops",
        "If a spell has {X} in its mana cost, include the value chosen for that X when determining the mana value of that spell."
    );
    supported("Erratic Cyclops");
    // "Whenever you cast an instant or sorcery spell, this creature gets +X/+0 until end
    // of turn, where X is that spell's mana value." Blaze with X = 3: +4/+0.
    let mut t = TestGame::new(2);
    let cyclops = t.battlefield(P0, "Erratic Cyclops");
    p0_blaze(&mut t, 3);
    t.resolve();
    assert_eq!(t.pt(cyclops), (4, 8));
}

#[test]
fn renegade_bull_gets_x_plus_one() {
    cr!("202.3e", "107.3a");
    ruling!(
        "Renegade Bull",
        "For spells on the stack with {X} in their mana costs, use the value chosen for X to determine the spell's mana value."
    );
    supported("Renegade Bull");
    // "Whenever you cast an instant or sorcery spell, this creature gets +X/+0 until end
    // of turn, where X is that spell's mana value." Blaze with X = 2: +3/+0.
    let mut t = TestGame::new(2);
    let bull = t.battlefield(P0, "Renegade Bull");
    p0_blaze(&mut t, 2);
    t.resolve();
    assert_eq!(t.pt(bull), (3, 5));
}

#[test]
fn electrosiphon_gets_energy_for_a_spell_s_x() {
    cr!("202.3e", "107.3a", "107.14");
    ruling!(
        "Electrosiphon",
        "For spells with {X} in their mana costs, use the value chosen for X to determine the spell’s mana value."
    );
    supported("Electrosiphon");
    // "Counter target spell. You get an amount of {E} equal to its mana value." Blaze
    // with X = 3 has mana value 4.
    let mut t = TestGame::new(2);
    let blaze = p1_blaze(&mut t, 3);
    crate::r_s25_common::cast_new(&mut t, P0, "Electrosiphon", &[Entity::Object(blaze)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Blaze"));
    assert_eq!(t.life(P0), 20);
    assert_eq!(energy(&t, P0), 4);
}

#[test]
fn lavabrink_venturer_uses_x_on_the_stack_and_0_elsewhere() {
    cr!("202.3e", "107.3g", "702.16b", "702.16f");
    ruling!(
        "Lavabrink Venturer",
        "For spells with {X} in their mana costs, use the value chosen for X to determine the spell's mana value. If a permanent or card in any other zone has {X} in its mana cost, X is considered to be 0."
    );
    supported("Lavabrink Venturer");
    // "As this creature enters, choose odd or even. (Zero is even.) This creature has
    // protection from each mana value of the chosen quality." P0 chooses odd: P1's Blaze
    // with X = 2 (mana value 3) can't target it; with X = 3 (mana value 4) it can.
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let venturer = t.enter(P0, "Lavabrink Venturer");
    t.settle();
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    t.lands(P1, "Wastes", 3);
    let blaze = t.hand(P1, "Blaze");
    t.answer(P1, DecisionKind::X, Answer::Number(2));
    let from = t.asked().len();
    t.cast(P1, blaze).target(Entity::Player(P0)).go();
    let candidates = crate::r_s02_common::target_candidates(&t, P1, from);
    assert!(!candidates[0].contains(&Entity::Object(venturer)));
    assert!(candidates[0].contains(&Entity::Player(P0)));
    t.resolve_all();
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let venturer = t.enter(P0, "Lavabrink Venturer");
    t.settle();
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    t.lands(P1, "Wastes", 3);
    let blaze = t.hand(P1, "Blaze");
    t.cast(P1, blaze).x(3).target(venturer).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Lavabrink Venturer"));
    // P0 chooses even: P1's Endless One cast with X = 3 (mana value 0 on the battlefield,
    // which is even) can't block it; Llanowar Elves (mana value 1) can.
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    let venturer = t.enter(P0, "Lavabrink Venturer");
    t.settle();
    t.g.objects[venturer.0 as usize].summoning_sick = false;
    t.set_step(P1, Step::PrecombatMain);
    let one = endless_one(&mut t, P1, 3);
    t.set_step(P0, Step::PrecombatMain);
    let elves = t.battlefield(P1, "Llanowar Elves");
    attack_with(&mut t, &[(venturer, Entity::Player(P1))]);
    assert!(!t.g.can_block(one, venturer));
    assert!(t.g.can_block(elves, venturer));
}

#[test]
fn phyrexian_symbols_count_1_each_toward_mana_value() {
    cr!("202.3g", "107.4f");
    ruling!(
        "Mental Misstep",
        "To calculate the mana value of a card with Phyrexian mana symbols in its cost, count each Phyrexian mana symbol as 1."
    );
    supported("Mental Misstep");
    supported("Gut Shot");
    supported("Tezzeret's Gambit");
    // Gut Shot ({R/P}) paid with 2 life is a spell with mana value 1: P1's Mental Misstep
    // ("Counter target spell with mana value 1.", also paid with 2 life) counters it.
    let mut t = TestGame::new(2);
    let shot = t.hand(P0, "Gut Shot");
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    let shot = t.cast(P0, shot).target(Entity::Player(P1)).go();
    assert_eq!(t.life(P0), 18);
    assert_eq!(mana_value(&t, shot), 1);
    let misstep = t.hand(P1, "Mental Misstep");
    t.answer(P1, DecisionKind::Option, Answer::Index(2));
    t.cast(P1, misstep).target(shot).go();
    assert_eq!(t.life(P1), 18);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Gut Shot"));
    assert_eq!(t.life(P1), 18);
    // Tezzeret's Gambit ({3}{U/P}) paid with {3} and 2 life: Kaervek deals 4.
    let mut t = TestGame::new(2);
    kaervek(&mut t);
    t.lands(P0, "Wastes", 3);
    let gambit = t.hand(P0, "Tezzeret's Gambit");
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    aim_kaervek_at_p0(&mut t);
    let spell = t.cast(P0, gambit).go();
    assert_eq!(t.life(P0), 18);
    assert_eq!(mana_value(&t, spell), 4);
    assert!(!spell_targets(&mut t, P1, "Mental Misstep").contains(&Entity::Object(spell)));
    t.resolve();
    assert_eq!(t.life(P0), 14);
}
