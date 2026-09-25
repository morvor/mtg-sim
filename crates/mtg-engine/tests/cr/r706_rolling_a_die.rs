//! CR 706: rolling a die.

use crate::r107_planechase::{add_planar_deck, planechase_game, roll as roll_planar};
use crate::r703_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::dice::DieRoll;
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::planechase::{self, PlanarFace};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// A die roll of this turn.
#[derive(Debug, PartialEq, Eq)]
struct Roll {
    player: PlayerId,
    sides: u32,
    natural: u32,
    result: u32,
    planar: bool,
}

fn rolls(t: &TestGame) -> Vec<Roll> {
    t.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::DieRolled {
                player,
                sides,
                result,
                natural,
                planar,
            } => Some(Roll {
                player: *player,
                sides: *sides,
                natural: *natural,
                result: *result,
                planar: *planar,
            }),
            _ => None,
        })
        .collect()
}

fn results(t: &TestGame) -> Vec<u32> {
    rolls(t).iter().map(|r| r.result).collect()
}

fn load(t: &mut TestGame, naturals: &[u32]) {
    t.g.dice.loaded.extend(naturals.iter().copied());
}

fn spell(name: &str, text: &str) -> CardDef {
    oracle_card(name, "Sorcery", "{0}", None, text)
}

/// Adorable Kitten enters: "When this creature enters, roll a six-sided die. You gain life
/// equal to the result."
fn kitten(t: &mut TestGame, p: PlayerId) {
    t.enter(p, "Adorable Kitten");
    t.resolve_all();
}

#[test]
fn an_n_sided_die_has_n_equally_likely_outcomes() {
    cr!("706.1", "706.1a", "706.1b");
    ruling!(
        "Netherese Puzzle-Ward",
        "digital substitutes are allowed, provided they have the same number of equally likely outcomes"
    );
    let mut t = TestGame::new(2);
    let mut d6 = DieRoll::new(6);
    d6.count = Value::c(600);
    run_effect(&mut t, P0, None, Effect::RollDice(Box::new(d6)), &[]);
    let r = rolls(&t);
    assert_eq!(r.len(), 600);
    assert!(r.iter().all(|x| x.sides == 6 && x.player == P0));
    for face in 1..=6 {
        let n = r.iter().filter(|x| x.result == face).count();
        // Expected 100 each; ±40 is more than four standard deviations.
        assert!((60..=140).contains(&n), "{face}: {n}");
    }
    // "Roll two d4": two four-sided dice; results from 1 to 4.
    let mut t = TestGame::new(2);
    cast_and_resolve(&mut t, P0, spell("Two Dice", "Roll two d4."), &[]);
    let r = rolls(&t);
    assert_eq!(r.len(), 2);
    assert!(r
        .iter()
        .all(|x| x.sides == 4 && (1..=4).contains(&x.result)));
}

#[test]
fn the_result_is_the_natural_result_after_modifiers() {
    cr!("706.2");
    ruling!(
        "Diviner's Portent",
        "is looking for the result after these modifications. Anything that is looking for the"
    );
    ruling!(
        "Vexing Puzzlebox",
        "If an effect instructs you to roll one or more dice and add some number to that roll, the result is the total after adding that number."
    );
    supported("Diviner's Portent");
    supported("Netherese Puzzle-Ward");
    let mut t = TestGame::new(2);
    // "Roll a d20 and add the number of cards in your hand. 1—14 | Draw X cards. 15+ |
    // Scry X, then draw X cards." A natural 12 with five cards in hand is a 17.
    for _ in 0..5 {
        t.hand(P0, "Grizzly Bears");
    }
    t.lands(P0, "Island", 4);
    load(&mut t, &[12]);
    let portent = t.hand(P0, "Diviner's Portent");
    t.cast(P0, portent).x(1).go();
    t.resolve_all();
    assert_eq!(
        rolls(&t),
        vec![Roll {
            player: P0,
            sides: 20,
            natural: 12,
            result: 17,
            planar: false
        }]
    );
    assert!(t
        .asked()
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::Scry { .. })));
    // "Whenever you roll a die's highest natural result" looks at the natural result: a
    // natural 3 plus 1 doesn't count; a natural 4 plus 1 does.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Netherese Puzzle-Ward");
    let hand = t.hand_size(P0);
    load(&mut t, &[3, 4]);
    cast_and_resolve(&mut t, P0, spell("Nudge", "Roll a d4 and add 1."), &[]);
    assert_eq!(results(&t), vec![4]);
    assert_eq!(t.hand_size(P0), hand);
    cast_and_resolve(&mut t, P0, spell("Nudge", "Roll a d4 and add 1."), &[]);
    assert_eq!(results(&t), vec![4, 5]);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn optional_modifiers_can_have_costs() {
    cr!("706.2a");
    ruling!(
        "Snickering Squirrel",
        "You decide whether to use Snickering Squirrel right after you see the result of the die roll."
    );
    ruling!(
        "Monitor Monitor",
        "If you roll multiple dice at the same time, paying {1} will let you reroll any number of those dice."
    );
    supported("Snickering Squirrel");
    supported("Adorable Kitten");
    let mut t = TestGame::new(2);
    // P1's Squirrel can increase a die P0 rolled: P1 decides and taps it.
    let squirrel = t.battlefield(P1, "Snickering Squirrel");
    t.answer_yes(P1, true);
    load(&mut t, &[5]);
    kitten(&mut t, P0);
    assert_eq!(rolls(&t)[0].natural, 5);
    assert_eq!(results(&t), vec![6]);
    assert_eq!(t.life(P0), 26);
    assert!(t.obj(squirrel).tapped);
    // Once tapped, it can't pay its cost again.
    load(&mut t, &[2]);
    let asked = t.asked().len();
    kitten(&mut t, P0);
    assert_eq!(results(&t)[1], 2);
    assert!(t.asked()[asked..]
        .iter()
        .all(|(p, d)| !(*p == P1 && matches!(d, Decision::YesNo { .. }))));

    // Monitor Monitor's reroll costs {1}: P0 activates a mana ability to pay it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Monitor Monitor");
    let forest = t.battlefield(P0, "Forest");
    t.answer_yes(P0, true);
    load(&mut t, &[1, 6]);
    kitten(&mut t, P0);
    assert!(t.obj(forest).tapped);
    let r = rolls(&t);
    assert_eq!(r.len(), 1, "the rerolled roll never happened");
    assert_eq!(r[0].natural, 6);
    assert_eq!(t.life(P0), 26);
    // Once each turn.
    t.g.untap(forest);
    load(&mut t, &[1]);
    kitten(&mut t, P0);
    assert_eq!(results(&t)[1], 1);
    assert!(!t.obj(forest).tapped);
}

#[test]
fn rerolls_are_considered_before_increases() {
    cr!("706.2b");
    ruling!(
        "Celebr-8000",
        "If a die is rerolled, the original roll essentially never happened"
    );
    supported("Clam-I-Am");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Snickering Squirrel");
    t.battlefield(P0, "Clam-I-Am");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    // A natural 3 on a six-sided die: Clam-I-Am's reroll first (a 4), then the Squirrel's
    // +1. (Adding first would have been undone by the reroll.)
    load(&mut t, &[3, 4]);
    kitten(&mut t, P0);
    assert_eq!(results(&t), vec![5]);
    assert_eq!(rolls(&t)[0].natural, 4);
    assert_eq!(t.life(P0), 25);
    let order: Vec<String> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::YesNo { prompt, .. } => Some(prompt),
            _ => None,
        })
        .collect();
    assert!(order[0].contains("reroll"), "{order:?}");
    assert!(order[1].contains("increase"), "{order:?}");
}

#[test]
fn a_results_table_says_what_happens_for_each_result() {
    cr!("706.3", "706.3a");
    supported("Herald of Hadar");
    let mut t = TestGame::new(2);
    let herald = t.battlefield(P0, "Herald of Hadar");
    let activate = |t: &mut TestGame, natural: u32| {
        t.lands(P0, "Swamp", 6);
        load(t, &[natural]);
        t.activate(P0, herald, 0, &[]).unwrap();
        t.resolve_all();
    };
    // 1—9: each opponent loses 2 life.
    activate(&mut t, 5);
    assert_eq!((t.life(P0), t.life(P1)), (20, 18));
    // 10—19: ... and you gain 2 life.
    activate(&mut t, 10);
    assert_eq!((t.life(P0), t.life(P1)), (22, 16));
    // 20: ... and create two Treasure tokens.
    activate(&mut t, 20);
    assert_eq!((t.life(P0), t.life(P1)), (24, 14));
    let treasures = t
        .permanents()
        .filter(|o| o.chars.has_subtype("Treasure"))
        .count();
    assert_eq!(treasures, 2);
}

#[test]
fn the_roll_and_its_results_table_are_one_ability() {
    cr!("706.3b");
    ruling!(
        "Herald of Hadar",
        "The instruction to roll a die and the effect that occurs because of the result are all part of the same ability. Players do not get the chance to respond to the ability after knowing the result of the roll."
    );
    let herald = card("Herald of Hadar");
    let abilities: Vec<_> = herald.faces[0]
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .collect();
    assert_eq!(abilities.len(), 1);
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Herald of Hadar");
    t.lands(P0, "Swamp", 6);
    load(&mut t, &[3]);
    t.activate(P0, h, 0, &[]).unwrap();
    // Nothing is rolled until the ability resolves...
    assert_eq!(t.stack_len(), 1);
    assert!(rolls(&t).is_empty());
    t.resolve();
    // ...and the table's effect happens as part of that same resolution.
    assert_eq!(results(&t), vec![3]);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn rolling_again_uses_the_same_dice_and_modifiers() {
    cr!("706.3c");
    ruling!(
        "Herald of Hadar",
        "Some effects instruct you to roll again. This uses the same number and type of dice as the original roll"
    );
    let mut t = TestGame::new(2);
    let again = spell(
        "Encore",
        "Roll a d6 and add 1.\n2—4 | You gain 1 life.\n5—7 | You gain 2 life. You may roll again.",
    );
    t.answer_yes(P0, true);
    load(&mut t, &[5, 2]);
    cast_and_resolve(&mut t, P0, again, &[]);
    // 5 + 1 = 6: gain 2 and roll again (a d6, still adding 1): 2 + 1 = 3: gain 1.
    let r = rolls(&t);
    assert_eq!(r.len(), 2);
    assert!(r.iter().all(|x| x.sides == 6));
    assert_eq!(results(&t), vec![6, 3]);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn without_a_results_table_the_text_says_how_to_use_the_result() {
    cr!("706.4");
    let mut t = TestGame::new(2);
    load(&mut t, &[4]);
    kitten(&mut t, P0);
    assert_eq!(t.life(P0), 24);
    supported("Ancient Gold Dragon");
    let mut t = TestGame::new(2);
    load(&mut t, &[3]);
    cast_and_resolve(
        &mut t,
        P0,
        spell(
            "Summon Faeries",
            "Roll a d20. You create a number of 1/1 blue Faerie Dragon creature tokens with flying equal to the result.",
        ),
        &[],
    );
    assert_eq!(t.named_on_battlefield("Faerie Dragon Token").len(), 3);
}

#[test]
fn rolling_doubles() {
    cr!("706.5");
    let doubles = || {
        spell(
            "Lucky Pair",
            "Roll two six-sided dice. If you rolled doubles, you gain 5 life.",
        )
    };
    let mut t = TestGame::new(2);
    load(&mut t, &[3, 3]);
    cast_and_resolve(&mut t, P0, doubles(), &[]);
    assert_eq!(t.life(P0), 25);
    load(&mut t, &[3, 4]);
    cast_and_resolve(&mut t, P0, doubles(), &[]);
    assert_eq!(t.life(P0), 25);
}

#[test]
fn an_ignored_roll_never_happened() {
    cr!("706.6");
    ruling!(
        "Netherese Puzzle-Ward",
        "Any die roll that is ignored, such as from Pixie Guide's effect, will not cause the Perfect Illumination ability to trigger."
    );
    ruling!(
        "Vexing Puzzlebox",
        "If an effect instructs you to roll more than one die and ignore one or more of them, the result is only what wasn't ignored."
    );
    supported("Pixie Guide");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pixie Guide");
    let counter = oracle_card(
        "Die Counter",
        "Enchantment",
        "{0}",
        None,
        "Whenever you roll a die, you gain 1 life.",
    );
    t.custom(P0, counter, Zone::Battlefield);
    // Pixie Guide: roll two, ignore the lowest. The ignored die triggers nothing.
    load(&mut t, &[2, 5]);
    kitten(&mut t, P0);
    assert_eq!(results(&t), vec![5]);
    assert_eq!(t.life(P0), 20 + 5 + 1);
    // A tie for the lowest: the player chooses which one to ignore.
    load(&mut t, &[4, 4]);
    let asked = t.asked().len();
    kitten(&mut t, P0);
    assert!(t.asked()[asked..].iter().any(|(p, d)| *p == P0
        && matches!(d, Decision::ChooseOption { prompt, .. } if prompt.contains("ignore"))));
    assert_eq!(results(&t), vec![5, 4]);
    // "Roll two d20 and ignore the lower roll."
    let mut t = TestGame::new(2);
    load(&mut t, &[16, 3]);
    cast_and_resolve(
        &mut t,
        P0,
        spell(
            "Advantage",
            "Roll two d20 and ignore the lower roll.\n1—14 | You gain 1 life.\n15—20 | You gain 5 life.",
        ),
        &[],
    );
    assert_eq!(results(&t), vec![16]);
    assert_eq!(t.life(P0), 25);
}

#[test]
fn the_planar_die_triggers_die_roll_abilities_without_a_numerical_result() {
    cr!("706.7");
    ruling!(
        "Vexing Puzzlebox",
        "In a game of Planechase, the result of rolling the planar die is not a number and does not cause Vexing Puzzlebox's first ability to put charge counters on it."
    );
    ruling!(
        "Component Pouch",
        "While playing Planechase, rolling the planar die will cause any ability that triggers whenever a player rolls one or more dice to trigger."
    );
    supported("Brazen Dwarf");
    let mut t = planechase_game(2, false);
    add_planar_deck(&mut t, P0, &["Goldmeadow", "The Fourth Sphere"]);
    planechase::set_starting_plane(&mut t.g);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Brazen Dwarf");
    let box_ = t.battlefield(P0, "Vexing Puzzlebox");
    // "Whenever you roll a die's highest natural result": the planar die has none.
    t.battlefield(P0, "Netherese Puzzle-Ward");
    let hand = t.hand_size(P0);
    roll_planar(&mut t, P0, PlanarFace::Blank);
    t.resolve_all();
    assert!(rolls(&t).iter().any(|r| r.planar));
    // Brazen Dwarf's ability triggered; the Puzzlebox's did too, but put no counters.
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.counters(box_, counters::CHARGE), 0);
    assert_eq!(t.hand_size(P0), hand);
    // A die with a numerical result puts counters on it.
    load(&mut t, &[7]);
    cast_and_resolve(&mut t, P0, spell("D20", "Roll a d20."), &[]);
    assert_eq!(t.counters(box_, counters::CHARGE), 7);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn results_stored_on_a_permanent_and_rerolled() {
    cr!("706.8", "706.8a", "706.8b");
    ruling!(
        "Centaur of Attention",
        "The value of X is the largest number of dice that match, not what number they're showing."
    );
    supported("Centaur of Attention");
    let mut t = TestGame::new(2);
    load(&mut t, &[1, 1, 3, 3, 4]);
    let centaur = t.enter(P0, "Centaur of Attention");
    t.resolve_all();
    assert_eq!(
        t.g.dice.stored.get(&centaur).cloned().unwrap_or_default(),
        vec![(6, 1), (6, 1), (6, 3), (6, 3), (6, 4)]
    );
    t.g.recompute();
    assert_eq!(t.pt(centaur), (5, 5));
    // At the beginning of combat: reroll the 3s and the 4 (three six-sided dice). The
    // rerolled results stop being stored; the new ones are stored.
    t.answer_yes(P0, true); // "you may"
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    load(&mut t, &[1, 1, 6]);
    t.set_step(P0, Step::PrecombatMain);
    to_step_start(&mut t, P0, Step::BeginningOfCombat);
    t.resolve_all();
    let mut stored = t.g.dice.stored.get(&centaur).cloned().unwrap_or_default();
    stored.sort();
    assert_eq!(stored, vec![(6, 1), (6, 1), (6, 1), (6, 1), (6, 6)]);
    // Rerolling is rolling: three six-sided dice were rolled.
    assert_eq!(results(&t)[5..], [1, 1, 6]);
    t.g.recompute();
    assert_eq!(t.pt(centaur), (7, 7));
}

#[test]
fn abilities_that_store_and_refer_to_stored_results_are_linked() {
    cr!("706.8c");
    ruling!(
        "Centaur of Attention",
        "If Centaur of Attention's enters-the-battlefield ability is copied or triggers multiple times somehow, it's possible to store more than five results on it."
    );
    let mut t = TestGame::new(2);
    load(&mut t, &[1, 2, 3, 4, 5]);
    let centaur = t.enter(P0, "Centaur of Attention");
    t.resolve_all();
    // A Clone copying it stores results on itself, and its +X/+X refers to those.
    t.answer_choose(P0, &[Entity::Object(centaur)]);
    load(&mut t, &[6, 6, 6, 2, 2]);
    let clone = t.enter(P0, "Clone");
    t.resolve_all();
    t.g.recompute();
    assert_eq!(t.obj_now(clone).chars.name.as_str(), "Centaur of Attention");
    assert_eq!(t.pt(centaur), (4, 4));
    assert_eq!(t.pt(clone), (6, 6));
    assert_eq!(
        t.g.dice.stored.get(&t.g.current(clone)).map(Vec::len),
        Some(5)
    );
    assert_eq!(t.g.dice.stored.get(&centaur).map(Vec::len), Some(5));
}
