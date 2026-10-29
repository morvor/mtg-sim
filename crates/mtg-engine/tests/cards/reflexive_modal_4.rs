//! Modal triggered abilities with extra structure: "whenever you cast a spell with power,
//! toughness, or mana value 4, choose one that hasn't been chosen this turn" (CR 700.2).

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn fantastic_four_triggers_on_power_four_and_offers_only_unchosen_modes() {
    cr!("700.2", "208.1");
    supported("The Fantastic Four");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Fantastic Four");
    t.lands(P0, "Plains", 4);
    t.lands(P0, "Forest", 2);
    // Power 4, mana value 2: triggers.
    let a = t.hand(P0, "Loxodon Peacekeeper");
    let b = t.hand(P0, "Loxodon Peacekeeper");
    // Power 2, mana value 2: doesn't.
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![3]));
    let hand = t.hand_size(P0);
    t.cast(P0, a).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 1);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.cast(P0, b).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // The second time, "Draw a card" wasn't available.
    let offered: Vec<Vec<usize>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseModes { available, .. } => Some(available),
            _ => None,
        })
        .collect();
    assert_eq!(offered.len(), 2);
    assert!(offered[0].contains(&3));
    assert!(!offered[1].contains(&3));
    let stack_before = t.stack_len();
    t.cast(P0, bears).go();
    assert_eq!(t.stack_len(), stack_before + 1);
}

#[test]
fn glorfindel_chosen_mode_then_gets_plus_one() {
    cr!("700.2", "608.2c", "509.1b");
    supported("Glorfindel, Dauntless Rescuer");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Glorfindel, Dauntless Rescuer");
    t.lands(P0, "Island", 1);
    let opt = t.hand(P0, "Opt");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    assert_eq!(t.g.max_blocked_by(g), None);
    t.cast(P0, opt).go();
    t.resolve_all();
    assert_eq!(t.pt(g), (4, 3));
    assert_eq!(t.g.max_blocked_by(g), Some(1));
}

#[test]
fn zuko_unchosen_mode_and_you_lose_two_life() {
    cr!("700.2", "608.2c");
    supported("Zuko, Conflicted");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zuko, Conflicted");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.advance_to(P0, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P0), 18);
    // Next time, "Draw a card" has been chosen.
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.advance_to(P0, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(t.life(P0), 16);
    let offered: Vec<Vec<usize>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseModes { available, .. } => Some(available),
            _ => None,
        })
        .collect();
    assert_eq!(offered, vec![vec![0, 1, 2, 3], vec![1, 2, 3]]);
}

#[test]
fn tranquil_frillback_pays_twice_and_chooses_up_to_two_modes() {
    cr!("603.12", "603.12a", "700.2");
    ruling!(
        "Tranquil Frillback",
        "you may pay {G} up to three times. That is, you may pay {G}, {G}{G}, {G}{G}{G}, or choose not to pay"
    );
    supported("Tranquil Frillback");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    t.graveyard(P1, "Grizzly Bears");
    let f = t.hand(P0, "Tranquil Frillback");
    // Pay {G} twice, then stop.
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1, 2]));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, f).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.graveyard_size(P1), 0);
    let maxes: Vec<u32> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseModes { max, .. } => Some(max),
            _ => None,
        })
        .collect();
    assert_eq!(maxes, vec![2]);
}

#[test]
fn tranquil_frillback_without_paying_no_reflexive_ability() {
    cr!("603.12");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    let f = t.hand(P0, "Tranquil Frillback");
    t.answer_yes(P0, false);
    t.cast(P0, f).go();
    t.resolve_all();
    assert!(!t
        .asked()
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseModes { .. })));
}

#[test]
fn resplendent_marshal_dies_exiles_another_creature_card() {
    cr!("603.12", "400.7");
    supported("Resplendent Marshal");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Resplendent Marshal");
    let angel = t.battlefield(P0, "Serra Angel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.graveyard(P0, "Baneslayer Angel");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(other)]);
    t.g.destroy(m, None);
    t.resolve_all();
    assert!(t.in_exile("Baneslayer Angel"));
    assert!(t.in_graveyard(P0, "Resplendent Marshal"));
    assert_eq!(t.counters(angel, "+1/+1"), 1);
    assert_eq!(t.counters(bears, "+1/+1"), 0);
    // The Marshal itself (now a card in the graveyard) wasn't a choice.
    let cands: Vec<Vec<Entity>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .collect();
    assert_eq!(cands, vec![vec![Entity::Object(other)]]);
}

#[test]
fn cemetery_desecrator_exiles_another_card_and_shrinks_by_its_mana_value() {
    cr!("603.12", "700.2");
    supported("Cemetery Desecrator");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Cemetery Desecrator");
    let giant = t.graveyard(P1, "Hill Giant");
    let target = t.battlefield(P1, "Serra Angel");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer_targets(P0, &[Entity::Object(target)]);
    t.g.destroy(d, None);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    assert!(!t.on_battlefield(target));
    let cands: Vec<Vec<Entity>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .collect();
    assert_eq!(cands, vec![vec![Entity::Object(giant)]]);
}

#[test]
fn incinerator_of_the_guilty_collects_evidence_x_and_deals_x() {
    cr!("701.59a", "603.12");
    supported("Incinerator of the Guilty");
    let mut t = TestGame::new(2);
    let inc = t.battlefield(P0, "Incinerator of the Guilty");
    let giant = t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Grizzly Bears");
    let angel = t.battlefield(P1, "Serra Angel");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Number, Answer::Number(4));
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.attack(&[(inc, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(!t.on_battlefield(angel));
    assert!(t.on_battlefield(wall));
    // X could be at most the total mana value in the graveyard (6).
    let maxes: Vec<i64> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseNumber { max, .. } => Some(max),
            _ => None,
        })
        .collect();
    assert_eq!(maxes, vec![6]);
}

fn unleash(victim: &str) -> (TestGame, ObjectId, ObjectId) {
    supported("Unleash the Inferno");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Swamp", 1);
    let v = t.battlefield(P1, victim);
    let stone = t.battlefield(P1, "Mind Stone");
    let wurm = t.battlefield(P1, "Wurmcoil Engine");
    let spell = t.hand(P0, "Unleash the Inferno");
    t.cast(P0, spell).target(Entity::Object(v)).go();
    t.answer_targets(P0, &[Entity::Object(stone)]);
    t.resolve_all();
    (t, stone, wurm)
}

#[test]
fn unleash_the_inferno_excess_damage_destroys_an_artifact_up_to_the_excess() {
    cr!("120.10", "603.12");
    // 7 damage to a 2/2: 5 excess.
    let (t, stone, wurm) = unleash("Grizzly Bears");
    assert!(!t.on_battlefield(stone));
    assert!(t.on_battlefield(wurm));
    let cands: Vec<Vec<Entity>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates),
            _ => None,
        })
        .collect();
    // The reflexive ability's target (the last target choice).
    let last = cands.last().expect("a target choice");
    assert!(last.contains(&Entity::Object(stone)));
    assert!(!last.contains(&Entity::Object(wurm)));
    // 7 damage to a 0/8: no excess, no reflexive ability.
    let (t, stone, _) = unleash("Wall of Stone");
    assert!(t.on_battlefield(stone));
}

fn rampage(victim: &str) -> (TestGame, ObjectId) {
    supported("Rhino's Rampage");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let giant = t.battlefield(P0, "Hill Giant");
    let v = t.battlefield(P1, victim);
    let stone = t.battlefield(P1, "Mind Stone");
    let spell = t.hand(P0, "Rhino's Rampage");
    t.cast(P0, spell)
        .targets(&[Entity::Object(giant), Entity::Object(v)])
        .go();
    t.answer_targets(P0, &[Entity::Object(stone)]);
    t.resolve_all();
    (t, stone)
}

#[test]
fn rhinos_rampage_excess_fight_damage_destroys_an_artifact() {
    cr!("120.10", "701.14a", "603.12");
    // 4 damage to a 2/2: excess.
    let (t, stone) = rampage("Grizzly Bears");
    assert!(!t.on_battlefield(stone));
    // 4 damage to a 0/8: none.
    let (t, stone) = rampage("Wall of Stone");
    assert!(t.on_battlefield(stone));
}

#[test]
fn immard_removing_a_counter_lets_you_choose_a_mode() {
    cr!("603.12", "700.2");
    supported("Immard, the Stormcleaver");
    let mut t = TestGame::new(2);
    let im = t.battlefield(P0, "Immard, the Stormcleaver");
    t.g.add_counters(Entity::Object(im), "charge", 1, None);
    // Remove a counter, then 4 damage to P1.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.attack(&[(im, Entity::Player(P1))], &[]);
    assert_eq!(t.counters(im, "charge"), 0);
    let power = t.pt(im).0 as i32;
    assert_eq!(t.life(P1), 20 - 4 - power);
}

#[test]
fn immard_putting_a_counter_doesnt_trigger_the_reflexive_ability() {
    cr!("603.12");
    let mut t = TestGame::new(2);
    let im = t.battlefield(P0, "Immard, the Stormcleaver");
    t.g.add_counters(Entity::Object(im), "charge", 1, None);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.attack(&[(im, Entity::Player(P1))], &[]);
    assert_eq!(t.counters(im, "charge"), 2);
    assert!(!t
        .asked()
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseModes { .. })));
}

#[test]
fn ghostly_dancers_unlocks_a_locked_door() {
    cr!("608.2d", "709.5f");
    supported("Ghostly Dancers");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 5);
    let room = t.battlefield(P0, "Dazzling Theater // Prop Room");
    t.graveyard(P0, "Sigarda's Splendor");
    let gd = t.hand(P0, "Ghostly Dancers");
    // Unlock a door (the second instruction), the right one (Prop Room).
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, gd).go();
    t.resolve_all();
    assert_eq!(mtg_engine::rooms::unlocked(&t.g, room), [false, true]);
    assert!(t.in_graveyard(P0, "Sigarda's Splendor"));
}

#[test]
fn explosion_of_riches_five_damage_per_card_drawn_this_way() {
    cr!("603.12", "115.1");
    supported("Explosion of Riches");
    let mut t = TestGame::new(4);
    t.lands(P0, "Mountain", 6);
    let spell = t.hand(P0, "Explosion of Riches");
    // P1 and P3 draw, P2 doesn't: three cards drawn this way.
    t.answer_yes(P1, true);
    t.answer_yes(P2, false);
    t.answer_yes(P3, true);
    t.cast(P0, spell).go();
    t.resolve_all();
    let lost: Vec<i32> = [P1, P2, P3].iter().map(|p| 20 - t.life(*p)).collect();
    assert_eq!(lost.iter().sum::<i32>(), 15);
    assert!(lost.iter().all(|l| l % 5 == 0));
    assert_eq!(t.life(P0), 20);
    // Nobody chose the targets.
    assert!(!t
        .asked()
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
}
