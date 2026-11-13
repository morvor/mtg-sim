//! Cards whose die rolls and coin flips compile with `oracle/patterns/r705_706_dice.rs`,
//! and "those creatures don't untap" after tapping them
//! (`oracle/patterns/those_dont_untap.rs`).

use mtg_engine::decision::Decision;
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
fn that_much_after_a_roll_is_the_result() {
    cr!("706.4");
    compiles("Dissatisfied Customer");
    compiles("Non-Human Cannonball");
    // "When this creature enters, roll a six-sided die. If the result is 3 or less, you
    // lose that much life."
    let mut t = TestGame::new(2);
    t.g.dice.loaded.push_back(2);
    t.enter(P0, "Dissatisfied Customer");
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    t.g.dice.loaded.push_back(5);
    t.enter(P0, "Dissatisfied Customer");
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    // "When this creature dies, roll a six-sided die. If the result is 4 or less, this
    // creature deals that much damage to you."
    let cannonball = t.battlefield(P0, "Non-Human Cannonball");
    t.g.dice.loaded.push_back(3);
    t.g.destroy(cannonball, None);
    t.resolve_all();
    assert_eq!(t.life(P0), 15);
}

#[test]
fn when_you_do_after_a_roll_triggers_with_the_result() {
    cr!("706.4", "603.12");
    compiles("Slight Malfunction");
    // "Roll a six-sided die. When you do, Slight Malfunction deals 1 damage to each of up
    // to X target creatures, where X is the result."
    for natural in [3, 1] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P1, "Grizzly Bears");
        let b = t.battlefield(P1, "Hill Giant");
        t.lands(P0, "Mountain", 2);
        t.g.dice.loaded.push_back(natural);
        let chosen: Vec<Entity> = [a, b]
            .iter()
            .take(natural as usize)
            .map(|x| Entity::Object(*x))
            .collect();
        t.answer_targets(P0, &chosen);
        let spell = t.hand(P0, "Slight Malfunction");
        t.cast(P0, spell).modes(&[1]).go();
        // Nothing is rolled (and nothing targeted) until the spell resolves.
        assert!(t.asked().iter().all(|(_, d)| !matches!(d, Decision::ChooseTargets { .. })));
        t.resolve_all();
        // The reflexive trigger's targets were chosen after the roll: up to the result.
        let max = t
            .asked()
            .into_iter()
            .find_map(|(_, d)| match d {
                Decision::ChooseTargets { min, max, .. } => Some((min, max)),
                _ => None,
            })
            .expect("targets chosen");
        assert_eq!(max, (0, natural.min(2)));
        let hit = [a, b]
            .iter()
            .filter(|x| t.obj_now(**x).damage == 1)
            .count();
        assert_eq!(hit, chosen.len(), "a roll of {natural}");
    }
}

#[test]
fn a_flip_that_counts_heads_has_no_call_and_no_winner() {
    cr!("705.2");
    compiles("Ral Zarek");
    compiles("Tavern Scoundrel");
    let mut t = TestGame::new(2);
    // "Whenever you win a coin flip, create two Treasure tokens."
    t.battlefield(P0, "Tavern Scoundrel");
    let ral = t.battlefield(P0, "Ral Zarek");
    t.g.objects[ral.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 7);
    t.set_step(P0, Step::PrecombatMain);
    // "−7: Flip five coins. Take an extra turn after this one for each coin that comes up
    // heads."
    t.g.dice
        .loaded_coins
        .extend([true, false, true, true, false]);
    t.activate(P0, ral, 2, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.g.extra_turns, vec![P0, P0, P0]);
    // Nobody called the flips, so nobody won one.
    assert!(t.asked().iter().all(|(_, d)| !matches!(
        d,
        Decision::ChooseOption { prompt, .. } if prompt == "Call the coin flip"
    )));
    assert!(t
        .permanents()
        .all(|o| !o.chars.has_subtype("Treasure")));
}

#[test]
fn those_creatures_a_results_table_row_tapped_dont_untap() {
    cr!("706.3a", "502.3");
    compiles("Cone of Cold");
    for (natural, stays_tapped) in [(15, true), (5, false)] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let mine = t.battlefield(P0, "Hill Giant");
        t.set_step(P0, Step::PrecombatMain);
        t.lands(P0, "Island", 4);
        t.g.dice.loaded.push_back(natural);
        let cone = t.hand(P0, "Cone of Cold");
        t.cast(P0, cone).go();
        t.resolve_all();
        assert!(t.obj_now(bears).tapped);
        assert!(!t.obj_now(mine).tapped);
        // 10—19: "Tap all creatures your opponents control. Those creatures don't untap
        // during their controllers' next untap steps."
        t.advance_to(P1, Step::Upkeep);
        assert_eq!(t.obj_now(bears).tapped, stays_tapped, "a roll of {natural}");
    }
}

#[test]
fn tapped_targets_dont_untap_during_their_controllers_next_untap_step() {
    cr!("502.3");
    compiles("Frost Breath");
    compiles("Breaching Leviathan");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let other = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(other);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 3);
    // "Tap up to two target creatures. Those creatures don't untap during their
    // controller's next untap step."
    let breath = t.hand(P0, "Frost Breath");
    t.cast(P0, breath)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(a).tapped && t.obj_now(b).tapped);
    assert!(!t.obj_now(other).tapped);
    // Only the next one.
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(a).tapped && !t.obj_now(b).tapped);
}
