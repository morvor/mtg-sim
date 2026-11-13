//! Variable, mixed and compound targets (`oracle/patterns/targets_grammar_2.rs`,
//! `per_player_targets.rs`, `phrases::parse_target`): a target inside a comparison or an
//! "among" group, "each of X targets", players among the kinds a target may be, several
//! target phrases as one subject, per-opponent targets, up to one target per counter, and
//! restrictions on an Aura's target checked only as it's cast.

use mtg_engine::testing::*;
use mtg_engine::types::*;
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
fn target_inside_a_comparison_fell_the_mighty() {
    cr!("601.2c", "608.2b");
    ruling!("Fell the Mighty", "Fell the Mighty resolves");
    compiles("Fell the Mighty");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 5);
    let small = t.battlefield(P0, "Grizzly Bears"); // 2/2
    let same = t.battlefield(P1, "Grizzly Bears");
    let big = t.battlefield(P1, "Hill Giant"); // 3/3
    let fell = t.hand(P0, "Fell the Mighty");
    t.cast(P0, fell).target(small).go();
    // The target's power as the spell resolves: 1.
    t.g.add_counters(Entity::Object(small), "-1/-1", 1, None);
    t.resolve();
    assert!(t.on_battlefield(small));
    assert!(!t.on_battlefield(same));
    assert!(!t.on_battlefield(big));
}

#[test]
fn target_player_of_an_among_group_fall_of_the_impostor() {
    cr!("601.2c");
    compiles("Fall of the Impostor");
    let mut t = TestGame::new(3);
    let fall = t.battlefield(P0, "Fall of the Impostor");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // A bigger creature another opponent controls doesn't matter.
    let wurm = t.battlefield(P2, "Craw Wurm");
    // Chapters I and II put their counters on the Wurm.
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    t.g.add_counters(Entity::Object(fall), "lore", 2, None);
    t.resolve_all();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.add_counters(Entity::Object(fall), "lore", 1, None);
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert!(t.on_battlefield(bears));
    assert!(t.on_battlefield(wurm));
}

#[test]
fn each_of_x_targets_spinning_wheel_kick() {
    cr!("601.2c", "115.3");
    compiles("Spinning Wheel Kick");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 6);
    let giant = t.battlefield(P0, "Hill Giant");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let c = t.battlefield(P1, "Grizzly Bears");
    let kick = t.hand(P0, "Spinning Wheel Kick");
    t.cast(P0, kick)
        .x(2)
        .target(giant)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve();
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
    assert!(t.on_battlefield(c));
}

#[test]
fn creature_token_player_or_planeswalker_coalborn_entity() {
    cr!("115.1", "115.4");
    compiles("Coalborn Entity");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Coalborn Entity");
    t.lands(P0, "Mountain", 6);
    t.activate(P0, e, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert_eq!(t.life(P1), 19);
    // A nontoken creature isn't a legal target.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.clear_answers();
    let r = t.activate(P0, e, 0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(r.is_err() || t.obj_now(bears).damage == 0);
}

#[test]
fn up_to_one_hundred_targets_baldin() {
    compiles("Baldin, Century Herdmaster");
    let mut t = TestGame::new(2);
    let baldin = t.battlefield(P0, "Baldin, Century Herdmaster");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.set_step(P0, mtg_engine::turn::Step::BeginningOfCombat);
    t.attack(&[(baldin, Entity::Player(P1))], &[]);
    assert_eq!(t.pt(a), (2, 4));
    assert_eq!(t.pt(b), (2, 4));
}

#[test]
fn any_number_of_them_phase_out_change_of_plans() {
    compiles("Change of Plans");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let cop = t.hand(P0, "Change of Plans");
    t.cast(P0, cop)
        .x(2)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.answer_yes(P0, true); // you may (any number)
    t.answer_yes(P0, true); // the first one phases out
    t.answer_yes(P0, false); // the second doesn't
    t.resolve_all();
    let phased = [a, b].iter().filter(|o| t.obj_now(**o).phased_out).count();
    assert_eq!(phased, 1);
}

#[test]
fn two_target_phrases_each_deal_damage_friendly_rivalry() {
    cr!("120.3");
    compiles("Friendly Rivalry");
    compiles("Graceful Takedown");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let legend = t.battlefield(P0, "Isamaru, Hound of Konda"); // legendary 2/2
    let wurm = t.battlefield(P1, "Craw Wurm"); // 6/4
    let fr = t.hand(P0, "Friendly Rivalry");
    t.lands(P0, "Mountain", 2);
    t.cast(P0, fr).target(a).target(legend).target(wurm).go();
    t.resolve();
    assert!(!t.on_battlefield(wurm));
}

#[test]
fn for_any_number_of_opponents_windgraces_judgment() {
    cr!("601.2c");
    compiles("Windgrace's Judgment");
    compiles("Geth's Summons");
    let mut t = TestGame::new(3);
    t.lands(P0, "Swamp", 3);
    t.lands(P0, "Forest", 2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P2, "Grizzly Bears");
    let wj = t.hand(P0, "Windgrace's Judgment");
    // A target for one opponent only.
    t.cast(P0, wj).targets(&[Entity::Object(a)]).go();
    t.resolve();
    assert!(!t.on_battlefield(a));
    assert!(t.on_battlefield(b));
}

#[test]
fn up_to_one_target_for_each_counter_quarantine_field() {
    compiles("Quarantine Field");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 6);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let c = t.battlefield(P1, "Craw Wurm");
    let qf = t.hand(P0, "Quarantine Field");
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.cast(P0, qf).x(2).go();
    t.resolve_all();
    let exiled = [a, b, c].iter().filter(|o| !t.on_battlefield(**o)).count();
    assert_eq!(exiled, 2);
}

#[test]
fn aura_target_restriction_only_as_cast_enthralling_hold() {
    cr!("601.2c", "608.2b", "303.4a");
    ruling!(
        "Enthralling Hold",
        "affects only the choice of target as the spell is cast"
    );
    compiles("Enthralling Hold");
    compiles("Dream Leash");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    let untapped = t.battlefield(P1, "Grizzly Bears");
    let hold = t.hand(P0, "Enthralling Hold");
    assert!(t.cast(P0, hold).target(untapped).try_go().is_err());
    let tapped = t.battlefield(P1, "Hill Giant");
    t.g.tap(tapped);
    let hold = t.hand(P0, "Enthralling Hold");
    t.cast(P0, hold).target(tapped).go();
    // It untaps before the spell resolves: it still resolves.
    t.g.objects[tapped.0 as usize].tapped = false;
    t.resolve();
    assert_eq!(t.obj_now(tapped).controller, P0);
}
